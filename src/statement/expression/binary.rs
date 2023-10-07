use crate::ast::{
    _LocationNode,
    expression::{
        BinaryExpression, BinaryOpeartorLeft, BinaryOperator, Expression, LogicalExpression,
        LogicalOperator, PrivateIdentifier,
    },
};
use crate::{
    parser::Parser,
    statement::util::unexpected,
    tokenizer::{js_token::TokenLabel, util::get_token_from_map},
};

use super::{parse_private_ident, unary::parse_maybe_unary};

// parse expression by operator
pub fn parse_expr_ops(ctx: &mut Parser) -> Expression {
    let start_loc = ctx.start_location_node();
    if ctx.cur_token_is(TokenLabel::PrivateId) {
        if ctx.private_name_stack.len() == 0 {
            unexpected(ctx.cur_token.clone().unwrap());
        }
        let private_ident = parse_private_ident(ctx);
        // Only 'IN' operator can be used after private identifier before parsing subscript expression.
        if !ctx.cur_token_is(TokenLabel::_In) {
            unexpected(ctx.cur_token.clone().unwrap());
        }
        return parse_expr_op(
            ctx,
            ExpressionOperatorLeft::PrivateIdentifier(private_ident),
            start_loc,
            -1,
        );
    }

    let expr = parse_maybe_unary(ctx, false, false);
    // TODO: check expression errors
    // TODO: check if expr is ArrowFunctionExpression
    // if expr.start == start_loc.pos &&
    parse_expr_op(ctx, ExpressionOperatorLeft::Expression(expr), start_loc, -1)
}

pub enum ExpressionOperatorLeft {
    Expression(Expression),
    PrivateIdentifier(PrivateIdentifier),
}

pub fn parse_expr_op(
    ctx: &mut Parser,
    left: ExpressionOperatorLeft,
    left_loc_node: _LocationNode,
    min_prec: i8,
) -> Expression {
    let cur_token = ctx.cur_token.as_ref().unwrap();
    if cur_token.binop.is_some() && !(ctx.for_init.is_some() && ctx.cur_token_is(TokenLabel::_In)) {
        let mut prec = cur_token.binop.unwrap() as i8;
        // most operators are left-associative, but for exponentiation operator(**) it's right-associative
        if prec > min_prec || prec == min_prec && cur_token.label == TokenLabel::StarStar {
            let logical = ctx.cur_token_test(|t| {
                t.label == TokenLabel::LogicalOr || t.label == TokenLabel::LogicalAnd
            });
            let coalesce = ctx.cur_token_is(TokenLabel::Coalesce);
            // change the precendence of coalesce to be equal with TokenLabel::Coalesce,
            // to check the mixed usage coalesce and logical error.
            if coalesce {
                prec = get_token_from_map(TokenLabel::LogicalAnd)
                    .unwrap()
                    .binop
                    .unwrap() as i8;
            }
            let op_val = ctx.get_cur_token_value();

            ctx.next_unwrap();

            let cur_left_loc_node = ctx.start_location_node();
            let next_left = if ctx.cur_token_is(TokenLabel::PrivateId) {
                ExpressionOperatorLeft::PrivateIdentifier(parse_private_ident(ctx))
            } else {
                ExpressionOperatorLeft::Expression(parse_maybe_unary(ctx, false, false))
            };
            let right = parse_expr_op(ctx, next_left, cur_left_loc_node, prec);
            let node = build_binary(
                ctx,
                left_loc_node.clone(),
                left,
                right,
                op_val,
                logical || coalesce,
            );
            if logical && ctx.cur_token_is(TokenLabel::Coalesce)
                || coalesce
                    && ctx.cur_token_test(|t| {
                        t.label == TokenLabel::LogicalAnd || t.label == TokenLabel::LogicalOr
                    })
            {
                println!("Logical expressions and coalesce expressions cannot be mixed. Wrap either by parentheses");
            }

            return parse_expr_op(
                ctx,
                ExpressionOperatorLeft::Expression(node),
                left_loc_node.clone(),
                min_prec,
            );
        }
    }

    // PrivateIdenfier could only be at the left of relational expression
    match left {
        ExpressionOperatorLeft::PrivateIdentifier(_) => {
            panic!("Private identifier can only be left side of binary expression");
        }
        ExpressionOperatorLeft::Expression(expr) => expr,
    }
}

pub fn build_binary(
    ctx: &Parser,
    start_loc_node: _LocationNode,
    left: ExpressionOperatorLeft,
    right: Expression,
    op: String,
    logical: bool,
) -> Expression {
    // TODO: check if private identifier was placed in the right side of expression
    match left {
        ExpressionOperatorLeft::PrivateIdentifier(pri_ident) => {
            if op != "in" {
                unexpected(ctx.cur_token.clone().unwrap());
            }
            BinaryExpression::new(
                BinaryOpeartorLeft::PrivateIdentifier(pri_ident),
                BinaryOperator::from(op),
                Box::new(right),
                ctx.compose_loc_info(start_loc_node),
            )
            .into()
        }
        ExpressionOperatorLeft::Expression(expr) => {
            if logical {
                LogicalExpression::new(
                    Box::new(expr),
                    LogicalOperator::from(op),
                    Box::new(right),
                    ctx.compose_loc_info(start_loc_node),
                )
                .into()
            } else {
                BinaryExpression::new(
                    BinaryOpeartorLeft::Expression(Box::new(expr)),
                    BinaryOperator::from(op),
                    Box::new(right),
                    ctx.compose_loc_info(start_loc_node),
                )
                .into()
            }
        }
    }
}
