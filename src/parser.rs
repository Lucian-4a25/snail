use crate::ast::{Position, _LocationNode, get_location_at, AstNodePos};
use crate::statement::scope::{
    Scope, SCOPE_ARROW, SCOPE_ASYNC, SCOPE_CLASS_STATIC_BLOCK, SCOPE_DIRECT_SUPER, SCOPE_FUNCTION,
    SCOPE_GENERATOR, SCOPE_SUPER, SCOPE_TOP, SCOPE_VAR,
};
use crate::statement::util::{can_insert_semicolon, unexpected};
use crate::tokenizer::context::{get_context_by_label, TokenContext, TokenContextLabel};
use crate::tokenizer::js_token::Token;
use crate::tokenizer::js_token::TokenLabel;
use crate::tokenizer::{next_token, TokenResult};
use std::collections::HashMap;
use std::string::String;
use std::vec;

#[derive(Debug, Eq, PartialEq)]
pub enum StatementContext {
    TopLevel,
    For,
    If,
    While,
    DoWhile,
    With,
    // for custom label statment
    LabelStmt,
}

#[derive(Debug)]
pub struct Parser {
    pub content: String,
    pub chars: Vec<char>,
    pub codes: Vec<u32>,
    /// 当前光标所在的字符位置
    pub cursor: usize,
    /// 是否允许解析正则表达式
    pub expr_allowed: bool,
    /// 当前光标所在行号
    pub line: usize,
    /// 当前行开始的位置
    pub line_start: usize,
    /// 上个 token 开始位置
    pub last_token_start: usize,
    /// 上个 token 结束位置
    pub last_token_end: usize,
    /// 上个 token 开始 loc
    pub last_token_start_loc: Option<Position>,
    /// 上个 token 结束 loc
    pub last_token_end_loc: Option<Position>,
    /// 当前解析得到的 token
    pub cur_token: Option<Token>,
    /// 上个解析得到的 token
    pub prev_token: Option<Token>,
    /// 当前 token 的位置
    pub cur_token_start: usize,
    pub cur_token_end: usize,
    pub cur_token_start_loc: Option<Position>,
    pub cur_token_end_loc: Option<Position>,
    /// 是否是严格模式
    pub strict_mode: bool,
    /// 解析的 token context
    pub token_context: Vec<TokenContext>,
    // TODO: 补充检查是否包含 escape 格式的情况
    /// 是否当前解析的 token 包含 escape 字符
    pub contains_esc: bool,
    /// 当前包含的 statement 上下文栈
    pub stmt_context: Vec<StatementContext>,
    /// 可能存在的箭头函数位置
    pub potential_arrow_pos: usize,
    /// 可能存在的 for await 箭头函数
    pub potential_arrow_in_for_await: bool,
    /// 作用域栈
    pub scope_stack: Vec<Scope>,
    // 最近的 name 为 await token 的开始位置
    pub await_ident_pos: usize,
    pub yield_pos: usize,
    pub await_pos: usize,
    /// 当前解析的源文件
    pub source_file: Option<String>,
    /// 用于 new 操作符后不允许解析 call 表达式中的情况
    pub disable_call_expr: bool,
    /// 用于禁止解析表达式在 forin 中可能出现的 in operator
    pub disable_in_op: bool,
    /// 用于判断是否当前解析的表达式位于 for 初始范围内
    pub for_init: Option<ForInitType>,
    pub private_name_stack: Vec<PrivateNameInfo>,
    /// statment labels
    pub labels: Vec<Label>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum AccessorKind {
    Get,
    Set,
    GetSet,
}

#[derive(Debug)]
pub struct PrivateNameProp {
    pub is_static: bool,
    pub accessor: Option<AccessorKind>,
}

#[derive(Debug)]
pub struct PrivateNameInfo {
    pub declared: HashMap<String, PrivateNameProp>,
    pub used: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Label {
    pub kind: LabelKind,
    pub name: Option<String>,
}

#[derive(Debug, Clone)]
pub enum LabelKind {
    Switch,
    Loop,
    // A custom label means there was no related switch or loop statment.
    Custom,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ForInitType {
    Normal,
    Await,
}

impl Parser {
    pub fn new(content: String) -> Parser {
        let mut chars = vec![];
        let mut codes = vec![];
        for char in content.chars() {
            chars.push(char);
            codes.push(char as u32);
        }
        Parser {
            content,
            chars,
            codes,
            cursor: 0,
            expr_allowed: false,
            line: 1,
            line_start: 0,
            last_token_end: 0,
            last_token_start: 0,
            last_token_start_loc: None,
            last_token_end_loc: None,
            cur_token: None,
            prev_token: None,
            cur_token_start: 0,
            cur_token_end: 0,
            cur_token_start_loc: None,
            cur_token_end_loc: None,
            token_context: vec![],
            strict_mode: false,
            contains_esc: false,
            stmt_context: vec![StatementContext::TopLevel],
            potential_arrow_pos: 0,
            potential_arrow_in_for_await: false,
            scope_stack: vec![Scope::new(SCOPE_TOP)],
            await_ident_pos: 0,
            yield_pos: 0,
            await_pos: 0,
            source_file: None,
            disable_call_expr: false,
            disable_in_op: false,
            for_init: None,
            private_name_stack: vec![],
            labels: vec![],
        }
    }

    pub fn next(&mut self) -> TokenResult {
        self.last_token_start = self.cur_token_start;
        self.last_token_end = self.cur_token_end;
        self.last_token_start_loc = self.cur_token_start_loc.clone();
        self.last_token_end_loc = self.cur_token_end_loc.clone();

        next_token(self)
    }

    pub fn next_unwrap(&mut self) -> Token {
        let next_token_res = self.next();
        if next_token_res.is_err() {
            panic!("read token error: {:?}", next_token_res.unwrap_err());
        }

        next_token_res.unwrap()
    }

    pub fn cur_token_test<F>(&self, f: F) -> bool
    where
        F: FnOnce(&Token) -> bool,
    {
        self.cur_token.as_ref().map_or(false, f)
    }

    pub fn cur_token_is(&self, label: TokenLabel) -> bool {
        self.cur_token.as_ref().map_or(false, |t| t.label == label)
    }

    pub fn get_cur_token_value(&self) -> String {
        let default_val = "".to_string();
        self.cur_token
            .as_ref()
            .map_or(default_val, |t| t.value.clone().unwrap_or("".to_string()))
    }

    pub fn cur_token_value_is(&self, val: &str) -> bool {
        self.cur_token
            .as_ref()
            .map_or(false, |t| t.value.clone().map_or(false, |v| v == val))
    }

    pub fn get_cursor_position(&self) -> Position {
        Position {
            line: self.line,
            col: self.cursor - self.line_start,
        }
    }

    // token context
    pub fn cur_token_ctx(&self) -> Option<&TokenContext> {
        self.token_context.last()
    }

    pub fn in_generator_ctx(&self) -> bool {
        for c in self.token_context.iter().rev() {
            if c.label == TokenContextLabel::FnStat
                || c.label == TokenContextLabel::FnExpr
                || c.label == TokenContextLabel::FnGen
                || c.label == TokenContextLabel::FnExprGen
            {
                return c.generator;
            }
        }
        return false;
    }

    pub fn overrid_token_context(&mut self, ctx_label: TokenContextLabel) {
        if self
            .cur_token_ctx()
            .map_or(true, |ctx| ctx.label != ctx_label)
        {
            self.token_context.pop();
            self.token_context.push(get_context_by_label(ctx_label));
        }
    }

    pub fn eat_contextual(&mut self, n: &str) -> bool {
        if self.is_contextual(n) {
            self.next_unwrap();
            return true;
        }

        false
    }

    pub fn expect_contexual(&mut self, n: &str) {
        if !self.eat_contextual(n) {
            unexpected(self.cur_token.clone().unwrap())
        }
    }

    pub fn is_contextual(&self, n: &str) -> bool {
        self.cur_token.as_ref().map_or(false, |t| {
            t.label == TokenLabel::Name && t.value.as_ref().map_or(false, |v| v == n)
        }) && !self.contains_esc
    }

    // parsing scope
    pub fn enter_scope(&mut self, flags: u32) {
        self.scope_stack.push(Scope::new(flags));
    }

    pub fn exit_scope(&mut self) {
        self.scope_stack.pop();
    }

    pub fn cur_this_scope(&self) -> Option<&Scope> {
        for sc in self.scope_stack.iter().rev() {
            if sc.flags & SCOPE_VAR > 0 && !(sc.flags & SCOPE_ARROW) > 0 {
                return Some(sc);
            }
        }
        None
    }

    pub fn cur_var_scope(&self) -> Option<&Scope> {
        for sc in self.scope_stack.iter().rev() {
            if sc.flags & SCOPE_VAR > 0 {
                return Some(sc);
            }
        }
        None
    }

    pub fn cur_scope(&self) -> Option<&Scope> {
        self.scope_stack.last()
    }

    pub fn allow_super(&self) -> bool {
        let cur_scope_op = self.cur_scope();
        if let Some(Scope {
            flags,
            in_class_field_init,
            ..
        }) = cur_scope_op
        {
            if flags & SCOPE_SUPER > 0 || *in_class_field_init {
                return true;
            }
        }
        false
    }

    pub fn allow_direct_super(&self) -> bool {
        let cur_scp_op = self.cur_this_scope();
        if let Some(Scope { flags, .. }) = cur_scp_op {
            if flags & SCOPE_DIRECT_SUPER > 0 {
                return true;
            }
        }
        false
    }

    pub fn allow_new_dot_target(&self) -> bool {
        let cur_scp_op = self.cur_this_scope();
        if let Some(Scope {
            flags,
            in_class_field_init,
            ..
        }) = cur_scp_op
        {
            if flags & (SCOPE_FUNCTION | SCOPE_CLASS_STATIC_BLOCK) > 0 || *in_class_field_init {
                return true;
            }
        }
        false
    }

    pub fn in_function_scope(&self) -> bool {
        if let Some(scp) = self.cur_scope() {
            return scp.flags & SCOPE_FUNCTION > 0;
        }
        false
    }

    pub fn in_generator_scope(&self) -> bool {
        if let Some(scp) = self.cur_scope() {
            return scp.flags & SCOPE_GENERATOR > 0 && !scp.in_class_field_init;
        }
        false
    }

    pub fn in_async_scope(&self) -> bool {
        if let Some(scp) = self.cur_scope() {
            return scp.flags & SCOPE_ASYNC > 0 && !scp.in_class_field_init;
        }
        false
    }

    pub fn can_await(&self) -> bool {
        for sp in self.scope_stack.iter().rev() {
            if sp.in_class_field_init || sp.flags & SCOPE_CLASS_STATIC_BLOCK > 0 {
                return false;
            }
            if sp.flags & SCOPE_FUNCTION > 0 {
                return sp.flags & SCOPE_ASYNC > 0;
            }
        }

        return false;
    }

    pub fn in_class_field_init(&self) -> bool {
        if let Some(scp) = self.cur_scope() {
            return scp.in_class_field_init;
        }
        false
    }

    pub fn in_class_static_block(&self) -> bool {
        if let Some(scp) = self.cur_scope() {
            return scp.flags & SCOPE_CLASS_STATIC_BLOCK > 0;
        }
        false
    }

    // ast node positioin related
    pub fn start_location_node(&self) -> _LocationNode {
        get_location_at(
            self.cur_token_start,
            self.cur_token_start_loc.clone().unwrap(),
        )
    }

    pub fn end_location_node(&self) -> _LocationNode {
        get_location_at(
            self.last_token_end,
            self.last_token_end_loc.clone().unwrap(),
        )
    }

    pub fn compose_loc_info(&self, start_loc: _LocationNode) -> AstNodePos {
        (
            start_loc,
            self.end_location_node(),
            self.source_file.clone(),
        )
    }

    /// if current token is label, consume it.
    pub fn eat(&mut self, label: TokenLabel) -> bool {
        if self.cur_token_is(label) {
            self.next_unwrap();
            true
        } else {
            false
        }
    }

    /// assert the current token label
    pub fn expect(&mut self, label: TokenLabel) -> bool {
        if self.eat(label) {
            return true;
        }

        panic!("Unexpected token");
    }

    // consume a semicolon or check if a semicolon could appear in that position
    pub fn semicolon(&mut self) {
        if !self.eat(TokenLabel::Semi) && !can_insert_semicolon(self) {
            unexpected(self.cur_token.clone().unwrap())
        }
    }

    // statement context
    pub fn enter_stmt_ctx(&mut self, stmt_ctx: StatementContext) {
        self.stmt_context.push(stmt_ctx);
    }

    pub fn exit_stmt_ctx(&mut self) {
        self.stmt_context.pop();
    }

    pub fn cur_stmt_ctx_is(&self, stmt_ctx: StatementContext) -> bool {
        self.stmt_context.last().map_or(false, |c| c == &stmt_ctx)
    }

    // class private name stack
    pub fn enter_private_name_stack(&mut self) {
        self.private_name_stack.push(PrivateNameInfo {
            declared: HashMap::new(),
            used: vec![],
        });
    }

    pub fn exit_private_name_stack(&mut self) -> Option<PrivateNameInfo> {
        self.private_name_stack.pop()
    }
}
