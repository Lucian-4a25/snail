use super::js_token::TokenLabel;
use super::number::{read_fixed_int, read_range_int};
use super::util::{get_token_from_map, is_new_line, move_to_next_line};
use super::{get_cur_code_from_ctx, TokenResult};
use crate::parser::Parser;

pub fn read_string_token(ctx: &mut Parser, code: usize) -> TokenResult {
    let mut value = String::new();
    loop {
        let c = get_cur_code_from_ctx(ctx);
        match c {
            c if c == code => {
                ctx.cursor += 1;
                break;
            }
            // '\'
            92 => {
                ctx.cursor += 1;
                if let Some(e) = read_escape_char(ctx) {
                    value.push(e);
                }
            }
            0x2028 | 0x209 => {
                ctx.cursor += 1;
                move_to_next_line(ctx);
            }
            10 | 13 => {
                panic!("Unterminated string constant");
            }
            _ => {
                ctx.cursor += 1;
                unsafe {
                    value.push(char::from_u32_unchecked(c as u32));
                }
            }
        }
    }

    get_token_from_map(TokenLabel::String).map(|mut r| {
        r.value = Some(value);
        r
    })
}

/// 当前的光标位置为 \ 的下个元素，从光标所在位置开始读取 escape char 的值
pub fn read_escape_char(ctx: &mut Parser) -> Option<char> {
    let mut iter = ctx.content.chars().skip(ctx.cursor);
    let code: usize = iter
        .next()
        .map(|r| r as usize)
        .expect("Unterminated string constant");
    ctx.cursor += 1;
    match code {
        110 => Some('\n'),
        114 => Some('\r'),
        // 'x'，后面是两位十六进制的数
        120 => Some(read_hex_char(ctx, 2)),
        // 'u'
        117 => Some(read_unicode_code_point_char(ctx)),
        116 => Some('\t'),
        // '\b' -> unicode value 8
        98 => Some('\u{08}'),
        118 => Some('\u{0b}'),
        102 => Some('\u{0c}'),
        // 'a\
        //    b' 形式的字符串，需要忽略后面的结果
        13 => {
            // 这是因为在 windows 中，换行符会被读取为 \r\n 两个字符, 所以需要判断一下此处的情况
            if get_cur_code_from_ctx(ctx) == 10 {
                ctx.cursor += 1;
            }
            move_to_next_line(ctx);
            None
        }
        // '\n'
        10 => {
            move_to_next_line(ctx);
            None
        }
        // 0-9
        n @ 48..=57 => {
            if n == 56 || n == 57 {
                panic!("Invalid escape sequence");
            }
            let mut oct_str = char::from_u32(n as u32).unwrap().to_string();
            let mut times = 2;
            while times > 0 {
                let code = get_cur_code_from_ctx(ctx);
                if code >= 48 && code <= 55 {
                    oct_str.push(char::from_u32(code as u32).unwrap());
                } else {
                    break;
                }
                times -= 1;
            }
            ctx.cursor += oct_str.len() - 1;

            let ch = usize::from_str_radix(&oct_str, 8).unwrap();
            if ch == 56 || ch == 57 {
                panic!("Invalid Octal literal")
            }
            if is_new_line(ch as u32) {
                return None;
            }

            Some(char::from_u32(ch as u32).unwrap())
        }
        _ => Some(char::from_u32(code as u32).unwrap()),
    }
}

/// 读取长度为 len 的十六进制字符
pub fn read_hex_char(ctx: &mut Parser, len: u8) -> char {
    let code = read_fixed_int(ctx, 16, len as u32, false);
    char::from_u32(code).expect("Not a valid Hex Value")
}

/// 读取 \u{xxxxxx} 格式的字符
pub fn read_unicode_code_point_char(ctx: &mut Parser) -> char {
    let code = get_cur_code_from_ctx(ctx);
    if code == 123 {
        ctx.cursor += 1;
        // '{'
        let v = read_range_int(ctx, 16, (1, 6), false);
        if get_cur_code_from_ctx(ctx) != 123 {
            panic!("Expected {{");
        }
        ctx.cursor += 1;
        return char::from_u32(v).expect("Not a valid Hex Value");
    }

    read_hex_char(ctx, 4)
}
