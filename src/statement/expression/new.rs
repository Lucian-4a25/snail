use super::{parse_epxr_list, parse_ident, subscript::parse_expr_subscripts, ExprListElement};
use crate::{
    ast::expression::{Expression, MetaProperty, NewExpression},
    parser::Parser,
    tokenizer::js_token::TokenLabel,
};

pub fn parse_new(ctx: &mut Parser) -> Expression {
    if ctx.contains_esc {
        println!("Escape sequence in keyword new");
    }
    let start_loc = ctx.start_location_node();
    let meta = parse_ident(ctx, true);
    // TODO: check ecam version
    if ctx.eat(TokenLabel::Dot) {
        let meta_contains_esc = ctx.contains_esc;
        let property = parse_ident(ctx, false);
        if property.name != "target" {
            println!("The only valid meta property for new is 'new.target'");
        }
        if meta_contains_esc {
            println!("'new.target' must not contain escaped characters");
        }
        if ctx.allow_new_dot_target() {
            println!("'new.target' can only be used in functions and class static block");
        }

        return MetaProperty::new(meta, property, ctx.compose_loc_info(start_loc)).into();
    }

    let mut new_args = vec![];
    let old_disable_call_expr = ctx.disable_call_expr;
    ctx.disable_call_expr = true;
    let callee = parse_expr_subscripts(ctx);
    ctx.disable_call_expr = old_disable_call_expr;

    if ctx.eat(TokenLabel::ParenL) {
        // TODO: allow trailing comma when ecam version >= 8
        let expr_list = parse_epxr_list(ctx, TokenLabel::ParenR, true, false);
        for e in expr_list {
            match e {
                ExprListElement::Expression(expr) => {
                    new_args.push(expr.into());
                }
                ExprListElement::SpreadElement(spr) => {
                    new_args.push(spr.into());
                }
                _ => {}
            }
        }
    }

    NewExpression::new(Box::new(callee), new_args, ctx.compose_loc_info(start_loc)).into()
}
