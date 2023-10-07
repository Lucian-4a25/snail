pub mod array;
pub mod assignment;
pub mod binary;
pub mod class;
pub mod conditional;
pub mod function;
pub mod import;
pub mod literal;
pub mod new;
pub mod object;
pub mod paren;
pub mod postfix;
pub mod subscript;
pub mod super_property;
pub mod template;
pub mod unary;
pub mod util;
use self::array::parse_arr_expr_or_pattern;
use self::assignment::parse_maybe_assign;
use self::class::parse_class_expr;
use self::function::{parse_arrow_expr, parse_func_expr};
use self::import::parse_import_expr;
use self::literal::parse_literal;
use self::new::parse_new;
use self::object::parse_obj_expr_or_pattern;
use self::paren::parse_parenl;
use self::super_property::parse_super;
use self::template::parse_template;
use self::unary::parse_maybe_unary;
use super::util::{
    after_trailing_comma, can_insert_semicolon, check_unreserved, is_async_func, unexpected,
};
use crate::ast::{
    expression::{
        ArrayExprEle, AwaitExpression, CallExprArgs, CallExpression, Expression, Identifier,
        PrivateIdentifier, SequenceExpression, SpreadElement, ThisExpression, YieldExpression,
    },
    pattern::Pattern,
};
use crate::parser::Parser;
use crate::tokenizer::{context::TokenContextLabel, js_token::TokenLabel, util::get_code_from_idx};
use core::panic;
use std::vec;

pub fn parse_expression(ctx: &mut Parser) -> Expression {
    let start_loc = ctx.start_location_node();
    let expr = parse_maybe_assign(ctx);
    if ctx.cur_token_is(TokenLabel::Comma) {
        let mut expressions = vec![expr];
        while ctx.eat(TokenLabel::Comma) {
            expressions.push(parse_maybe_assign(ctx));
        }
        return SequenceExpression::new(expressions, ctx.compose_loc_info(start_loc)).into();
    }

    expr
}

// TODO: Figure out a good way to predicate expression and pattern.
pub fn parse_expr_atom(ctx: &mut Parser) -> Expression {
    // TODO:
    // In accorn, here need to judge if this is a slash token, and if it's, then to read regex token.
    // I think we should keep tokinize implemention decoupling with parse process,
    // and fix weird regex action in token context. So We here just ignore that.

    let cur_token = ctx.cur_token.as_ref().unwrap();
    match cur_token.label {
        TokenLabel::_Super => parse_super(ctx),
        TokenLabel::_This => {
            let start_loc = ctx.start_location_node();
            ctx.next_unwrap();
            ThisExpression::new(ctx.compose_loc_info(start_loc)).into()
        }
        TokenLabel::Name => parse_atom_name(ctx),
        TokenLabel::Regexp
        | TokenLabel::Number
        | TokenLabel::String
        | TokenLabel::_True
        | TokenLabel::_False
        | TokenLabel::_Null => parse_literal(ctx).into(),
        TokenLabel::ParenL => parse_parenl(ctx),
        TokenLabel::BracketL => parse_arr_expr_or_pattern(ctx),
        TokenLabel::BraceL => {
            ctx.overrid_token_context(TokenContextLabel::BraceExpr);
            parse_obj_expr_or_pattern(ctx)
        }
        TokenLabel::_Function => parse_func_expr(ctx, false).into(),
        TokenLabel::_Class => parse_class_expr(ctx).into(),
        TokenLabel::_New => parse_new(ctx),
        TokenLabel::BackQuote => parse_template(ctx, false).into(),
        TokenLabel::_Import => parse_import_expr(ctx),
        _ => unexpected(ctx.cur_token.clone().unwrap()),
    }
}

// For a single token which starts with Name, there is a few possible parsing result:
// - async function expression, eg. let fn = async function() {}
// - async arrow function expression, eg. let fn = async () => {}; and, let f = async ident => {}
// - an arrow function expression, eg. let fn = param => {}
// - an ordinary identifier
pub fn parse_atom_name(ctx: &mut Parser) -> Expression {
    let start_loc = ctx.start_location_node();
    let may_be_arrow_func = ctx.potential_arrow_pos == ctx.cur_token_start;

    // maybe async function expression.
    if is_async_func(ctx) {
        ctx.overrid_token_context(TokenContextLabel::FnExpr);
        return parse_func_expr(ctx, true).into();
    }

    let mut maybe_async_arrow_func = may_be_arrow_func && ctx.is_contextual("async");
    let mut ident_node = parse_ident(ctx, false);

    // for case: ident => {}
    if may_be_arrow_func && !can_insert_semicolon(ctx) && ctx.eat(TokenLabel::Arrow) {
        return parse_arrow_expr(ctx, start_loc, vec![Pattern::Identifier(ident_node)], false)
            .into();
    }

    // for case: async [no LineTerminator here] ident [no LineTerminator here] => {}
    if maybe_async_arrow_func
        && !can_insert_semicolon(ctx)
        && ctx.cur_token_is(TokenLabel::Name)
        && (!ctx.potential_arrow_in_for_await || !ctx.cur_token_value_is("of") || ctx.contains_esc)
    {
        ident_node = parse_ident(ctx, false);
        if can_insert_semicolon(ctx) && !ctx.eat(TokenLabel::Arrow) {
            unexpected(ctx.cur_token.clone().unwrap());
        }
        return parse_arrow_expr(ctx, start_loc, vec![Pattern::Identifier(ident_node)], true)
            .into();
    }

    if !ctx.disable_call_expr && ctx.cur_token_is(TokenLabel::ParenL) {
        maybe_async_arrow_func = maybe_async_arrow_func && !can_insert_semicolon(ctx);
        ctx.eat(TokenLabel::ParenL);
        let expr_list = parse_epxr_list(ctx, TokenLabel::ParenR, true, false);
        // TODO: check ecam >= 8
        // for case: async (..) => {}
        if maybe_async_arrow_func && !can_insert_semicolon(ctx) && ctx.eat(TokenLabel::Arrow) {
            // TODO: check pattern errors for arrow functions
            return parse_arrow_expr(ctx, start_loc, vec![Pattern::Identifier(ident_node)], true)
                .into();
        }
        // TODO: check expression errors.
        return CallExpression::new(
            Expression::from(ident_node).into(),
            expr_list.into_iter().map(|e| e.into()).collect(),
            false,
            ctx.compose_loc_info(start_loc),
        )
        .into();
    }

    ident_node.into()
}

pub enum ExprListElement {
    Expression(Expression),
    SpreadElement(SpreadElement),
    Null,
}

impl From<ExprListElement> for ArrayExprEle {
    fn from(value: ExprListElement) -> Self {
        match value {
            ExprListElement::Expression(e) => Self::Expression(e),
            ExprListElement::SpreadElement(s) => Self::SpreadElement(s),
            ExprListElement::Null => Self::Null,
        }
    }
}

impl From<ExprListElement> for CallExprArgs {
    fn from(value: ExprListElement) -> Self {
        match value {
            ExprListElement::Expression(e) => Self::Expression(e),
            ExprListElement::SpreadElement(s) => Self::SpreadElement(s),
            ExprListElement::Null => {
                panic!("Call expression arguments disallow empty value.")
            }
        }
    }
}

// parse a list of expression, for case:
// - call expression params list
pub fn parse_epxr_list(
    ctx: &mut Parser,
    close: TokenLabel,
    allow_trailing_comma: bool,
    allow_empty: bool,
) -> Vec<ExprListElement> {
    let mut eles: Vec<ExprListElement> = vec![];
    let mut first = true;
    while !ctx.eat(close) {
        if first {
            first = false;
        } else {
            ctx.expect(TokenLabel::Comma);
        }

        if allow_trailing_comma && after_trailing_comma(ctx, close, true) {
            break;
        }

        if allow_empty && ctx.cur_token_is(TokenLabel::Comma) {
            eles.push(ExprListElement::Null);
        } else if ctx.cur_token_is(TokenLabel::Ellipsis) {
            eles.push(ExprListElement::SpreadElement(parse_spread_el(ctx)));
        } else {
            eles.push(ExprListElement::Expression(parse_maybe_assign(ctx)));
        }
    }

    eles
}

pub fn parse_spread_el(ctx: &mut Parser) -> SpreadElement {
    let start_loc = ctx.start_location_node();
    ctx.next_unwrap();

    let argument = parse_maybe_assign(ctx);

    SpreadElement::new(argument, ctx.compose_loc_info(start_loc))
}

/// 解析一个标识符，如果 liberal 为 true 表示是一个对象属性，忽略关键字检查
pub fn parse_ident(ctx: &mut Parser, liberal: bool) -> Identifier {
    let mut name = String::new();
    let start_loc = ctx.start_location_node();
    let cur_token = ctx.cur_token.as_ref().expect("Unexpected Token");
    if ctx.cur_token_is(TokenLabel::Name) {
        name.push_str(cur_token.value.as_ref().unwrap());
    } else if cur_token.keyword {
        name.push_str(cur_token.label.as_str());
        // To fix: https://github.com/acornjs/acorn/issues/575
        if (name == TokenLabel::_Class.as_str() || name == TokenLabel::_Function.as_str())
            && (ctx.last_token_end != ctx.last_token_start + 1
                || get_code_from_idx(ctx, ctx.last_token_start) != 46)
        {
            ctx.token_context.pop();
        }
    } else {
        unexpected(ctx.cur_token.clone().unwrap());
    }

    if liberal && cur_token.keyword && ctx.contains_esc {
        println!("Escape sequence in keyword {}", cur_token.label.as_str());
    }

    ctx.next_unwrap();

    let ident_node = Identifier::new(name, ctx.compose_loc_info(start_loc));

    if !liberal {
        check_unreserved(ctx, &ident_node.name, ident_node.start, ident_node.end);
        if &ident_node.name == "await" && ctx.await_ident_pos == 0 {
            ctx.await_ident_pos = ident_node.start;
        }
    }

    ident_node
}

// 解析一个 yield 表达式
pub fn parse_yield(ctx: &mut Parser) -> YieldExpression {
    let start_loc = ctx.start_location_node();
    if ctx.yield_pos == 0 {
        ctx.yield_pos = ctx.cur_token_start;
    }

    ctx.next_unwrap();

    let mut delegate = false;
    let mut argument = None;
    if !ctx.cur_token_is(TokenLabel::Semi)
        && !can_insert_semicolon(ctx)
        && !ctx.cur_token_test(|t| t.label != TokenLabel::Star && !t.starts_expr)
    {
        if ctx.eat(TokenLabel::Star) {
            delegate = true;
        }
        argument = Some(Box::new(parse_maybe_assign(ctx)));
    }

    YieldExpression::new(delegate, argument, ctx.compose_loc_info(start_loc))
}

// parse await expression
pub fn parse_await(ctx: &mut Parser) -> AwaitExpression {
    let start_loc = ctx.start_location_node();
    if ctx.await_pos == 0 {
        ctx.await_pos = ctx.cur_token_start;
    }

    ctx.next_unwrap();

    let argument = parse_maybe_unary(ctx, false, false);

    AwaitExpression::new(Box::new(argument), ctx.compose_loc_info(start_loc))
}

pub fn parse_private_ident(ctx: &mut Parser) -> PrivateIdentifier {
    let start_loc = ctx.start_location_node();

    if !ctx.cur_token_is(TokenLabel::PrivateId) {
        unexpected(ctx.cur_token.clone().unwrap());
    }
    let name = ctx.get_cur_token_value();
    ctx.next_unwrap();

    let last_private_name_op = ctx.private_name_stack.last_mut();
    if last_private_name_op.is_none() {
        println!(
            "Private field '#${}' must be declared in an enclosing class",
            name
        );
    } else {
        let last_private_name = last_private_name_op.unwrap();
        last_private_name.used.push(name.clone());
    }

    PrivateIdentifier::new(name, ctx.compose_loc_info(start_loc))
}

pub fn parse_paren_expression(ctx: &mut Parser) -> Expression {
    ctx.expect(TokenLabel::ParenL);
    let expr = parse_expression(ctx);
    ctx.expect(TokenLabel::ParenR);

    expr
}
