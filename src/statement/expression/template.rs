use super::parse_expression;
use crate::{
    ast::expression::{Expression, TemplateElement, TemplateLiteral, TemplateValue},
    parser::Parser,
    statement::util::unexpected,
    tokenizer::js_token::TokenLabel,
};

pub fn parse_template(ctx: &mut Parser, tagged: bool) -> TemplateLiteral {
    let start_loc = ctx.start_location_node();
    let mut expressions: Vec<Expression> = vec![];
    let mut quasis: Vec<TemplateElement> = vec![];

    ctx.next_unwrap();

    loop {
        if ctx.cur_token_is(TokenLabel::Eof) {
            panic!("Unterminated template literal");
        }
        let tmp_el = parse_template_ele(ctx, tagged);
        if tmp_el.tail {
            quasis.push(tmp_el);
            ctx.next_unwrap();
            break;
        }
        quasis.push(tmp_el);
        ctx.expect(TokenLabel::DollarBraceL);
        expressions.push(parse_expression(ctx));
        ctx.expect(TokenLabel::BraceR);
    }

    TemplateLiteral::new(quasis, expressions, ctx.compose_loc_info(start_loc))
}

pub fn parse_template_ele(ctx: &mut Parser, tagged: bool) -> TemplateElement {
    let start_loc = ctx.start_location_node();
    let value = if ctx.cur_token_is(TokenLabel::Invalidtemplate) {
        if !tagged {
            println!("Bad escape sequence in untagged template literal");
        }
        TemplateValue {
            raw: ctx.get_cur_token_value(),
            cooked: None,
        }
    } else if ctx.cur_token_is(TokenLabel::Template) {
        let v = ctx.get_cur_token_value();
        TemplateValue {
            raw: v.clone(),
            cooked: Some(v),
        }
    } else {
        unexpected(ctx.cur_token.clone().unwrap())
    };

    ctx.next_unwrap();

    TemplateElement::new(
        value,
        ctx.cur_token_is(TokenLabel::BackQuote),
        ctx.compose_loc_info(start_loc),
    )
}
