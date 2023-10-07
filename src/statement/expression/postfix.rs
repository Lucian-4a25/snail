use super::{subscript::parse_expr_subscripts, util::is_lhs_expr_simple};
use crate::{
    ast::expression::{Expression, UpdateExpression, UpdateOperator},
    parser::Parser,
    statement::util::can_insert_semicolon,
};

// UpdateExpression[Yield, Await] :
// LeftHandSideExpression[?Yield, ?Await] [no LineTerminator here] ++
// LeftHandSideExpression[?Yield, ?Await] [no LineTerminator here] --
pub fn parse_maybe_postfix(ctx: &mut Parser) -> Expression {
    let start_loc = ctx.start_location_node();
    let expr = parse_expr_subscripts(ctx);
    if ctx.cur_token_test(|t| t.postfix) && !can_insert_semicolon(ctx) {
        if !is_lhs_expr_simple(&expr, ctx.strict_mode) {
            panic!("Invalid left-hand side expression in postfix operation");
        }
        let operator = UpdateOperator::from(ctx.get_cur_token_value());
        ctx.next_unwrap();
        return UpdateExpression::new(
            operator,
            Box::new(expr),
            false,
            ctx.compose_loc_info(start_loc.clone()),
        )
        .into();
    }

    expr
}
