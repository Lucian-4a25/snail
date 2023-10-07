use super::{parse_await, postfix::parse_maybe_postfix};
use crate::{
    ast::expression::{
        Expression, UnaryExpression, UnaryOperator, UpdateExpression, UpdateOperator,
    },
    parser::Parser,
    statement::util::unexpected,
    tokenizer::js_token::TokenLabel,
};

// parse unary operator
pub fn parse_maybe_unary(ctx: &mut Parser, mut saw_unary: bool, is_inc_dec: bool) -> Expression {
    let start_loc = ctx.start_location_node();
    let expr;

    if ctx.is_contextual("await") && ctx.can_await() {
        expr = parse_await(ctx).into();
        saw_unary = true;
    } else if ctx.cur_token_test(|t| t.prefix) {
        let is_update = ctx.cur_token_test(|t| t.label == TokenLabel::IncDec);
        let operator_val = ctx.get_cur_token_value();
        ctx.next_unwrap();
        let argument = parse_maybe_unary(ctx, saw_unary, is_update);
        // TODO: check expression errors

        if is_update {
            // TODO: check LValue
        } else {
            saw_unary = true;
        }

        expr = if is_update {
            UpdateExpression::new(
                UpdateOperator::from(operator_val),
                Box::new(argument),
                true,
                ctx.compose_loc_info(start_loc),
            )
            .into()
        } else {
            UnaryExpression::new(
                UnaryOperator::from(operator_val),
                Box::new(argument),
                true,
                ctx.compose_loc_info(start_loc),
            )
            .into()
        };
    } else {
        expr = parse_maybe_postfix(ctx);
    }

    if !is_inc_dec && ctx.eat(TokenLabel::StarStar) {
        if saw_unary {
            unexpected(ctx.cur_token.clone().unwrap());
        }
    }

    expr
}
