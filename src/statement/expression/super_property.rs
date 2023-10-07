use super::{parse_epxr_list, parse_expression, parse_ident};
use crate::{
    ast::expression::{CallExpression, Expression, MemberExprProperty, MemberExpression, Super},
    parser::Parser,
    statement::util::unexpected,
    tokenizer::js_token::TokenLabel,
};

// Here we choose not to reuse the parse_sub_script code instead to parse what's next after Super first,
// the purpuse is to make the return value of 'parse_atom' become a Expression type instead of a SuperElement.
// Thus it could make others function which is based 'parse_atom' more common.
pub fn parse_super(ctx: &mut Parser) -> Expression {
    if !ctx.allow_super() {
        panic!("'super' keyword outside a method")
    }
    let start_loc = ctx.start_location_node();
    let next_token = ctx.next_unwrap();
    let next_token_label = next_token.label;
    // check if super keyword is in valid place
    if next_token_label == TokenLabel::ParenL && !ctx.allow_direct_super() {
        panic!("super() call outside constructor of a subclass");
    }

    let super_el = Super::new(ctx.compose_loc_info(start_loc.clone()));

    match next_token_label {
        TokenLabel::Dot | TokenLabel::BracketL => {
            let computed = next_token_label == TokenLabel::BracketL;
            ctx.next_unwrap();
            let property: Expression;
            if computed {
                property = parse_expression(ctx);
                ctx.expect(TokenLabel::BracketR);
            } else {
                property = parse_ident(ctx, true).into();
            };
            MemberExpression::new(
                super_el.into(),
                MemberExprProperty::Expression(Box::new(property)),
                computed,
                false,
                ctx.compose_loc_info(start_loc),
            )
            .into()
        }

        TokenLabel::ParenL => {
            let expr_list = parse_epxr_list(ctx, TokenLabel::ParenR, true, false);
            CallExpression::new(
                super_el.into(),
                expr_list.into_iter().map(|e| e.into()).collect(),
                false,
                ctx.compose_loc_info(start_loc),
            )
            .into()
        }
        _ => unexpected(ctx.cur_token.clone().unwrap()),
    }
}
