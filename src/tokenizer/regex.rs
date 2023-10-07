use super::js_token::TokenLabel;
use super::util::is_new_line;
use super::{get_token_from_map, TokenResult};
use crate::parser::Parser;
use regex::Regex;

pub fn read_regex_token(ctx: &mut Parser) -> TokenResult {
    let mut escaped = false;
    let mut in_class = false;
    let mut result = "/".to_string();
    let mut iter = ctx.content.chars().skip(ctx.cursor);
    loop {
        let n = iter.next().map(|c| c as usize);
        if n.is_none() {
            panic!("Unterminated regular expression")
        }
        let next = n.unwrap();
        if is_new_line(next as u32) {
            panic!("Unterminated regular expression")
        }
        if !escaped {
            match next {
                // '['
                91 => {
                    in_class = true;
                }
                // ']'
                n if n == 93 && in_class => {
                    in_class = false;
                }
                // '/'
                n if n == 47 && !in_class => {
                    ctx.cursor += 1;
                    result.push('/');
                    break;
                }
                // '\'
                92 => {
                    escaped = true;
                }
                _ => {}
            }
            if next != 92 {
                result.push(char::from_u32(next as u32).unwrap());
            }
        } else {
            result.push(char::from_u32(next as u32).unwrap());
        }
        ctx.cursor += 1;
    }

    let re = Regex::new(r"[a-z]").unwrap();
    for c in iter {
        if re.is_match(&c.to_string()) {
            ctx.cursor += 1;
            result.push(c);
        } else {
            break;
        }
    }

    // TODO: 忽略检验正则是否合法的检查，因为检查一个正则的处理过于复杂，所以暂时不处理

    get_token_from_map(TokenLabel::Regexp).map(|mut r| {
        r.value = Some(result);
        r
    })
}
