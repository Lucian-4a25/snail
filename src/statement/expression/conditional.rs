use crate::ast::expression::{ConditionalExpression, Expression};
use crate::{parser::Parser, tokenizer::js_token::TokenLabel};

use super::{assignment::parse_maybe_assign, binary::parse_expr_ops};

// parse a ternary operator
pub fn parse_maybe_conditional(ctx: &mut Parser) -> Expression {
    let start_loc = ctx.start_location_node();
    let expr = parse_expr_ops(ctx);
    // TODO: check expression errors

    if ctx.cur_token_is(TokenLabel::Question) {
        let consequent = parse_maybe_assign(ctx);
        ctx.expect(TokenLabel::Colon);
        let alternate = parse_maybe_assign(ctx);
        return ConditionalExpression::new(
            Box::new(expr),
            Box::new(consequent),
            Box::new(alternate),
            ctx.compose_loc_info(start_loc),
        )
        .into();
    }

    expr
}
