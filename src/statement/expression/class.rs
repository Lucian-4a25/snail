use crate::ast::expression::ClassExpression;
use crate::statement::parse_class_body;
use crate::{parser::Parser, tokenizer::js_token::TokenLabel};

use super::{parse_ident, subscript::parse_expr_subscripts};

pub fn parse_class_expr(ctx: &mut Parser) -> ClassExpression {
    let start_loc = ctx.start_location_node();
    let old_strict = ctx.strict_mode;
    // A class definition is always strict mode code.
    ctx.strict_mode = true;
    ctx.next_unwrap();

    let id = if ctx.cur_token_is(TokenLabel::Name) {
        Some(parse_ident(ctx, true))
    } else {
        None
    };

    let super_class = if ctx.eat(TokenLabel::_Extends) {
        Some(Box::new(parse_expr_subscripts(ctx)))
    } else {
        None
    };

    let class_body = parse_class_body(ctx, super_class.is_some());
    ctx.strict_mode = old_strict;

    ClassExpression::new(id, super_class, class_body, ctx.compose_loc_info(start_loc))
}
