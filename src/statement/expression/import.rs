use crate::{
    ast::expression::{Expression, ImportExpression, MetaProperty},
    parser::Parser,
    statement::util::unexpected,
    tokenizer::js_token::TokenLabel,
};

use super::{assignment::parse_maybe_assign, parse_ident};

// parse import.meta or a dynamic import expression
pub fn parse_import_expr(ctx: &mut Parser) -> Expression {
    let start_loc = ctx.start_location_node();
    if ctx.contains_esc {
        println!("Escape sequence in keyword import");
    }
    let meta = parse_ident(ctx, true);

    // import("")
    if ctx.cur_token_is(TokenLabel::ParenR) && !ctx.disable_call_expr {
        ctx.next_unwrap();
        //    TODO: complete parse assign
        let source = parse_maybe_assign(ctx);
        if !ctx.eat(TokenLabel::ParenR) {
            if ctx.eat(TokenLabel::Comma) && ctx.eat(TokenLabel::ParenR) {
                println!("Trailing comma is not allowed in import()");
            } else {
                unexpected(ctx.cur_token.clone().unwrap());
            }
        }

        return ImportExpression::new(Box::new(source), ctx.compose_loc_info(start_loc)).into();
    }

    if ctx.cur_token_is(TokenLabel::Dot) {
        ctx.next_unwrap();
        let contains_esc = ctx.contains_esc;
        let property = parse_ident(ctx, true);
        if property.name != "meta" {
            println!("The only valid meta property for import is 'import.meta'");
        }
        if contains_esc {
            println!("'import.meta' must not contain escaped characters");
        }
        if ctx.source_file.as_ref().map_or(true, |s| s != "module") {
            println!("Cannot use 'import.meta' outside a module");
        }

        return MetaProperty::new(meta, property, ctx.compose_loc_info(start_loc)).into();
    }

    unexpected(ctx.cur_token.clone().unwrap());
}
