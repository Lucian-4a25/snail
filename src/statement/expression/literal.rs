use crate::{
    ast::expression::{Literal, LiteralValue},
    parser::Parser,
    tokenizer::js_token::TokenLabel,
};
use regex::Regex;

pub fn parse_literal(ctx: &mut Parser) -> Literal {
    let start_loc = ctx.start_location_node();
    let cur_token = ctx.cur_token.as_ref().unwrap();
    // TODO: ignore raw property for now

    let literal_val = match cur_token.label {
        TokenLabel::String => LiteralValue::String(cur_token.value.clone().unwrap()),
        TokenLabel::Number => {
            LiteralValue::Number(cur_token.value.clone().unwrap().parse::<i64>().unwrap())
        }
        TokenLabel::_True | TokenLabel::_False => {
            LiteralValue::Boolean(cur_token.value.clone().unwrap().parse::<bool>().unwrap())
        }
        TokenLabel::Regexp => {
            let reg = Regex::new(&cur_token.value.clone().unwrap());
            LiteralValue::Regx(reg.ok())
        }
        TokenLabel::_Null => LiteralValue::Null,
        _ => {
            panic!("Unexpected token");
        }
    };

    ctx.next_unwrap();

    // TODO: 补充 Regex 的详细字段 { flag, pattern }
    Literal::new(literal_val, None, None, ctx.compose_loc_info(start_loc))
}
