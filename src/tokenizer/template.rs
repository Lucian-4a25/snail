use super::{
    js_token::TokenLabel,
    string::read_escape_char,
    util::{get_token_from_map, is_new_line, move_to_next_line, next_code_is},
    TokenResult,
};
use crate::parser::Parser;

pub fn read_template_token(ctx: &mut Parser) -> TokenResult {
    let mut result = String::new();
    let start_pos = ctx.cur_token_start;
    let cur_is_template = ctx.cur_token_test(|t| {
        t.label == TokenLabel::Template || t.label == TokenLabel::Invalidtemplate
    });

    loop {
        let ch_op = ctx.codes.get(ctx.cursor);
        if ch_op.is_none() {
            panic!("Unterminated template");
        }
        let ch = *ch_op.unwrap();
        let maybe_substitution = ctx.cursor == start_pos && cur_is_template;
        // '`'
        if ch == 96 {
            if maybe_substitution {
                ctx.cursor += 1;
                return get_token_from_map(TokenLabel::BackQuote);
            } else {
                return get_token_from_map(TokenLabel::Template).map(|mut token| {
                    token.value = Some(result);
                    token
                });
            }
        }
        // '${'
        else if ch == 36 && next_code_is(ctx, 123) {
            if maybe_substitution {
                ctx.cursor += 2;
                return get_token_from_map(TokenLabel::DollarBraceL);
            } else {
                return get_token_from_map(TokenLabel::Template).map(|mut token| {
                    token.value = Some(result);
                    token
                });
            }
        }
        // '\'
        else if ch == 92 {
            ctx.cursor += 1;
            if let Some(c) = read_escape_char(ctx) {
                result.push(c);
            }
        } else if is_new_line(ch as u32) {
            let v = if ch == 13
                && ctx
                    .codes
                    .get(ctx.cursor + 1)
                    .map_or(false, |next_c| *next_c == 10)
            {
                ctx.cursor += 1;
                '\n'
            } else {
                char::from_u32(ch as u32).unwrap()
            };
            result.push(v);
            move_to_next_line(ctx);
        } else {
            result.push(char::from_u32(ch as u32).unwrap());
            ctx.cursor += 1;
        }
    }
}
