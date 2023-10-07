use super::{
    parse_epxr_list, parse_expr_atom, parse_expression, parse_ident, parse_private_ident,
    template::parse_template,
};
use crate::{
    ast::{
        _LocationNode,
        expression::{
            CallExpression, ChainExpression, ChainExpressionElement, Expression,
            MemberExprProperty, MemberExpression, TaggedTemplateExpression,
        },
    },
    parser::Parser,
    statement::util::unexpected,
    tokenizer::js_token::TokenLabel,
};

// parse []、.、?.、() subscript
pub fn parse_expr_subscripts(ctx: &mut Parser) -> Expression {
    let start_loc = ctx.start_location_node();
    let expr = parse_expr_atom(ctx);
    // TODO: check if expr is Arrow Expression

    parse_subscripts(ctx, expr, start_loc.clone())
}

// parse subscript based exp
pub fn parse_subscripts(
    ctx: &mut Parser,
    base: Expression,
    start_loc_node: _LocationNode,
) -> Expression {
    let mut base_node = base;
    let mut optional_chained = false;

    loop {
        let optional = ctx.eat(TokenLabel::QuestionDot);
        let computed = ctx.eat(TokenLabel::BracketL);
        if optional {
            optional_chained = true;
        }
        if ctx.disable_call_expr && optional {
            panic!("Optional chaining cannot appear in the callee of new expressions");
        }
        if computed
            || (optional
                && !ctx.cur_token_test(|t| {
                    t.label != TokenLabel::ParenL && t.label != TokenLabel::BackQuote
                }))
            || ctx.eat(TokenLabel::Dot)
        {
            let property: MemberExprProperty;
            if computed {
                property = parse_expression(ctx).into();
                ctx.expect(TokenLabel::BracketR);
            }
            // TODO: alse to check to token's type
            else if ctx.cur_token_is(TokenLabel::PrivateId) {
                property = parse_private_ident(ctx).into();
            } else {
                property = Expression::from(parse_ident(ctx, true)).into();
            }

            base_node = MemberExpression::new(
                base_node.into(),
                property,
                computed,
                optional,
                ctx.compose_loc_info(start_loc_node.clone()),
            )
            .into();
        } else if !ctx.disable_call_expr && ctx.eat(TokenLabel::ParenL) {
            let expr_list = parse_epxr_list(ctx, TokenLabel::ParenR, true, false);
            // TODO: check expression errors
            base_node = CallExpression::new(
                base_node.into(),
                expr_list.into_iter().map(|e| e.into()).collect(),
                optional,
                ctx.compose_loc_info(start_loc_node.clone()),
            )
            .into();
        } else if ctx.cur_token_is(TokenLabel::BackQuote) {
            if optional || optional_chained {
                panic!("Optional chaining cannot appear in the tag of tagged template expressions");
            }
            let quasis = parse_template(ctx, true);
            base_node = TaggedTemplateExpression::new(
                Box::new(base_node),
                quasis,
                ctx.compose_loc_info(start_loc_node.clone()),
            )
            .into();
        } else {
            break;
        }
    }

    if optional_chained {
        let chain_expr = match base_node {
            Expression::CallExpression(expr) => ChainExpressionElement::CallExpression(expr),
            Expression::MemberExpression(expr) => ChainExpressionElement::MemberExpression(expr),
            _ => {
                unexpected(ctx.cur_token.clone().unwrap());
            }
        };
        base_node = ChainExpression::new(chain_expr, ctx.compose_loc_info(start_loc_node)).into();
    }

    base_node
}
