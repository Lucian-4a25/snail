use crate::ast::expression::{Expression, Identifier, LiteralValue};
use crate::ast::statement::Statement;
use crate::{
    parser::{
        AccessorKind, Label, LabelKind, Parser, PrivateNameInfo, PrivateNameProp, StatementContext,
    },
    tokenizer::{
        js_token::{Token, TokenLabel},
        util::{has_break_in_range, is_identifier_char, is_identifier_start, is_keyword_token},
    },
};
use regex::Regex;

lazy_static! {
    static ref skip_space_commnet: Regex = Regex::new(r"(\s|//.*|(?s:/\*.*?\*/))*").unwrap();
    static ref keyword_relation_operator: Regex = Regex::new(r"^in(stanceof)?$").unwrap();
}

pub fn check_used_private_name(ctx: &mut Parser, private_name_info: PrivateNameInfo) {
    let last_private_stack_op = ctx.private_name_stack.last_mut();
    let has_last_stack = last_private_stack_op.is_some();
    let mut unchecked = vec![];
    for name in private_name_info.used {
        if private_name_info.declared.contains_key(&name) {
            continue;
        }
        if has_last_stack {
            unchecked.push(name);
        } else {
            println!(
                "Private field '#${}' must be declared in an enclosing class",
                name
            );
        }
    }
    if unchecked.len() > 0 {
        last_private_stack_op.unwrap().used.append(&mut unchecked);
    }
}

pub fn check_private_name_conflicts(
    private_info: &mut PrivateNameInfo,
    name: &str,
    is_static: bool,
    kind: Option<AccessorKind>,
) {
    let v_op = private_info.declared.get_mut(name);
    if v_op.is_none() {
        private_info.declared.insert(
            name.to_string(),
            PrivateNameProp {
                is_static,
                accessor: kind,
            },
        );
    } else {
        let v = v_op.unwrap();
        // static private key conflicts with non-static private key that has the same name
        if is_static && !v.is_static {
            println!("Identifier '#${}' has already been declared", name);
            return;
        }
        if v.accessor.is_none()
            || kind.is_none()
            || matches!(&v.accessor, Some(AccessorKind::GetSet))
            || v.accessor == kind
        {
            println!("Identifier '#${}' has already been declared", name);
            return;
        }
        v.accessor = Some(AccessorKind::GetSet);
    }
}

pub fn check_unreserved(ctx: &Parser, name: &str, start: usize, end: usize) {
    if ctx.in_generator_scope() && name == "yield" {
        println!("Cannot use 'yield' as identifier inside a generator");
    }
    if ctx.in_async_scope() && name == "await" {
        println!("Cannot use 'await' as identifier inside an async function");
    }
    if ctx.in_class_field_init() && name == "arguments" {
        println!("Cannot use 'arguments' in class field initializer");
    }
    if ctx.in_class_static_block() && (name == "arguments" || name == "await") {
        panic!("Cannot use ${} in class static initialization block", name);
    }
    if is_keyword_token(name).is_some() {
        panic!("Unexpected keyword {}", name);
    }
    // TODO: if ecamversion < 6, can return directly
    // if has_break_in_range(ctx, (start, end)) {
    //      return;
    // }
    // TODO: check reserved word list
}

pub fn unexpected(token: Token) -> ! {
    panic!("Unexpected token {:?}", token);
}

pub fn can_insert_semicolon(ctx: &Parser) -> bool {
    ctx.cur_token.as_ref().map_or(false, |t| {
        t.label == TokenLabel::Eof || t.label == TokenLabel::BraceR
    }) || has_break_in_range(ctx, (ctx.last_token_end, ctx.cur_token_start))
}

// check if trailing comma is followed by label
pub fn after_trailing_comma(ctx: &mut Parser, label: TokenLabel, auto_next: bool) -> bool {
    if ctx.cur_token_is(label) {
        if auto_next {
            ctx.next_unwrap();
        }
        return true;
    }
    false
}

pub fn is_import_expr(ctx: &Parser) -> bool {
    let skip_word_count = skip_space_comment_at_char_idx(&ctx.content, ctx.cursor, true);
    let mut iter = ctx.content.chars().skip(ctx.cursor + skip_word_count);
    let next_char_op = iter.next().map(|c| c as usize);
    if let Some(next_char) = next_char_op {
        // '(' or '.'
        return next_char == 40 || next_char == 46;
    }

    false
}

// check if there is break charactor between 'async' and 'function'
pub fn is_async_func(ctx: &Parser) -> bool {
    if !ctx.is_contextual("async") {
        return false;
    }
    let skip_word_count = skip_space_comment_at_char_idx(&ctx.content, ctx.cursor, true);
    if has_break_in_range(ctx, (ctx.cursor, ctx.cursor + skip_word_count)) {
        return false;
    }

    if ctx.content.starts_with("function") {
        let byte_idx =
            get_byte_idx_by_char_idx(&ctx.content, ctx.cursor + skip_word_count).unwrap();
        if byte_idx + 8 == ctx.content.chars().count() {
            return true;
        }
        let next_func_ch = ctx
            .content
            .chars()
            .skip(ctx.cursor + skip_word_count + 8)
            .next()
            .map_or(0, |ch| ch as usize);
        if !(is_identifier_char(next_func_ch) || next_func_ch > 0xffff) {
            return true;
        }
    }

    false
}

pub fn is_let(ctx: &Parser) -> bool {
    if !ctx.is_contextual("let") {
        return false;
    }

    let skip_word_count = skip_space_comment_at_char_idx(&ctx.content, ctx.cursor, true);
    let mut iter = ctx.content.chars().skip(ctx.cursor + skip_word_count);
    let next_char_op = iter.next().map(|c| c as usize);
    if next_char_op.is_none() {
        return false;
    }
    let next_char = next_char_op.unwrap();
    // '[' '/'
    if next_char == 91 || next_char == 92 {
        return true;
    }
    if !ctx.cur_stmt_ctx_is(StatementContext::TopLevel) {
        return false;
    }
    // '{', astral
    if next_char == 123 || next_char > 0xffff {
        return true;
    }
    if is_identifier_start(next_char) {
        let mut identifier = String::from(char::from_u32(next_char as u32).unwrap());
        while let Some(c) = iter.next().map(|ch| ch as usize) {
            if is_identifier_char(c) {
                identifier.push(char::from_u32(c as u32).unwrap());
            } else if c == 92 || c > 0xffff {
                return true;
            } else {
                break;
            }
        }

        if !keyword_relation_operator.is_match(&identifier) {
            return false;
        }
    }

    false
}

/// TODO: 优化这种使用正则的方式，其实可以复用 skip_space_comment 这个方法的，而且读取字符的效率理论上性能会更好，
/// 只要暂时记录当前的 cursor、line、line_start 几个值，之后再写回去就行了
/// 读取从 start_idx 开始的空白字符以及注释字符数目
/// 支持 字节索引和字符索引两种
/// 返回值: 跳过的字符长度
pub fn skip_space_comment_at_char_idx(target: &str, start_idx: usize, is_char_idx: bool) -> usize {
    let mut s_idx = start_idx;
    if is_char_idx {
        if let Some(idx) = get_byte_idx_by_char_idx(target, start_idx) {
            s_idx = idx;
        } else {
            return 0;
        }
    }
    let match_op = skip_space_commnet.find_at(target, s_idx);
    if let Some(mat) = match_op {
        return mat.as_str().chars().count();
    }

    0
}

/// 根据字符的索引位置获取对应的字节索引位置
pub fn get_byte_idx_by_char_idx(target: &str, char_idx: usize) -> Option<usize> {
    let mut iter = target.char_indices().skip(char_idx);
    if let Some((idx, _)) = iter.next() {
        return Some(idx);
    }

    None
}

// check if label exists or there is valid destination to go.
pub fn check_label_destination(
    labels: &Vec<Label>,
    cur_label: &Option<Identifier>,
    is_break: bool,
) -> bool {
    let mut valid = false;
    for label in labels.iter() {
        if cur_label.is_none() {
            if matches!(label.kind, LabelKind::Custom) {
                continue;
            }
            if is_break {
                valid = true;
                break;
            }
            // continue could only be used to loop label
            if matches!(label.kind, LabelKind::Loop) {
                valid = true;
                break;
            }
        } else {
            let ident = cur_label.as_ref().unwrap();
            if label.name.as_ref().map_or(false, |n| n == &ident.name) {
                if is_break {
                    valid = true;
                    break;
                }
                if matches!(label.kind, LabelKind::Loop) {
                    valid = true;
                    break;
                }
            }
        }
    }

    valid
}

pub fn is_directive_candidate(stmt: &Statement) -> bool {
    if let Statement::ExpressionStatement(expr) = stmt {
        if let Expression::Literal(literal) = &expr.expression {
            if let LiteralValue::String(..) = &literal.value {
                // ensure there is not paren surrounded with string
                return expr.start == literal.start;
            }
        }
    }

    false
}
