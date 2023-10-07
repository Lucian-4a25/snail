use super::{conditional::parse_maybe_conditional, parse_yield};
use crate::ast::expression::{
    AssignmentExpression, AssignmentExpressionLeft, AssignmentOperator, Expression,
};
use crate::{
    parser::{ForInitType, Parser},
    tokenizer::js_token::TokenLabel,
};

pub fn parse_maybe_assign(ctx: &mut Parser) -> Expression {
    if ctx.is_contextual("yield") {
        if ctx.in_generator_scope() {
            parse_yield(ctx);
        } else {
            ctx.expr_allowed = true;
        }
    }

    let start_loc = ctx.start_location_node();
    if ctx.cur_token_is(TokenLabel::Name) || ctx.cur_token_is(TokenLabel::ParenL) {
        ctx.potential_arrow_pos = ctx.cur_token_start;
        ctx.potential_arrow_in_for_await = ctx
            .for_init
            .as_ref()
            .map_or(false, |t| *t == ForInitType::Await);
    }

    let left = parse_maybe_conditional(ctx);

    if ctx.cur_token_test(|t| t.is_assign) {
        let operator = AssignmentOperator::from(ctx.get_cur_token_value());

        // TODO: skip assignable check for now.
        ctx.next_unwrap();
        let right = parse_maybe_assign(ctx);

        return Expression::AssignmentExpression(AssignmentExpression::new(
            AssignmentExpressionLeft::Expression(Box::new(left)),
            operator,
            Box::new(right),
            ctx.compose_loc_info(start_loc),
        ));
    }

    left
}
