use super::{assignment::parse_maybe_assign, parse_ident};
use crate::ast::{
    _LocationNode,
    expression::{ArrowFunctionBody, ArrowFunctionExpression, Expression, FunctionExpression},
    pattern::Pattern,
    statement::{ExpressionStatement, FunctionBody, FunctionBodyContent, Statement},
};
use crate::{
    parser::Parser,
    statement::{
        lval::parse_binding_list,
        parse_statement,
        scope::{get_func_flags, SCOPE_ARROW},
        util::is_directive_candidate,
    },
    tokenizer::js_token::TokenLabel,
};

pub fn parse_arrow_expr(
    ctx: &mut Parser,
    start_loc: _LocationNode,
    params: Vec<Pattern>,
    is_async: bool,
) -> ArrowFunctionExpression {
    ctx.enter_scope(get_func_flags(is_async, false) | SCOPE_ARROW);
    let is_blk = ctx.cur_token_is(TokenLabel::BraceL);
    let body = if is_blk {
        ArrowFunctionBody::FunctionBoby(parse_function_body(ctx))
    } else {
        ArrowFunctionBody::Expression(Box::new(parse_maybe_assign(ctx)))
    };
    ctx.exit_scope();

    ArrowFunctionExpression::new(
        params,
        body,
        !is_blk,
        is_async,
        ctx.compose_loc_info(start_loc),
    )
}

pub fn parse_func_expr(ctx: &mut Parser, is_async: bool) -> FunctionExpression {
    let start_loc = ctx.start_location_node();
    if is_async {
        ctx.expect_contexual("async");
    }
    ctx.expect(TokenLabel::_Function);
    let is_generator = ctx.eat(TokenLabel::Star);
    let id = if ctx.cur_token_is(TokenLabel::Name) {
        Some(parse_ident(ctx, true))
    } else {
        None
    };
    ctx.enter_scope(get_func_flags(is_async, is_generator));
    ctx.expect(TokenLabel::ParenL);
    let params = parse_binding_list(ctx, TokenLabel::ParenR, false, true);
    let body = parse_function_body(ctx);
    ctx.exit_scope();

    FunctionExpression::new(
        id,
        params.into_iter().map(|p| p.unwrap()).collect(),
        body,
        is_generator,
        is_async,
        ctx.compose_loc_info(start_loc),
    )
}

pub fn parse_function_body(ctx: &mut Parser) -> FunctionBody {
    let start_loc = ctx.start_location_node();
    let old_labels = ctx.labels.clone();
    let mut body: Vec<FunctionBodyContent> = vec![];
    let mut allow_dirctive = true;
    ctx.labels = vec![];
    // TODO: check simple params in strict mode
    ctx.expect(TokenLabel::BraceL);
    while !ctx.eat(TokenLabel::BraceR) {
        let stmt = parse_statement(ctx);
        if !allow_dirctive {
            body.push(FunctionBodyContent::Statement(stmt.into()));
        } else if is_directive_candidate(&stmt) {
            if let Statement::ExpressionStatement(ExpressionStatement {
                expression: Expression::Literal(literal),
                ..
            }) = stmt
            {
                body.push(FunctionBodyContent::Directive(literal.into()));
            }
        } else {
            allow_dirctive = false;
            body.push(FunctionBodyContent::Statement(stmt.into()));
        }
    }

    ctx.labels = old_labels;

    FunctionBody::new(body, ctx.compose_loc_info(start_loc))
}
