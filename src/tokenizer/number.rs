use super::{
    js_token::TokenLabel,
    util::{get_cur_code_from_ctx, get_token_from_map},
    TokenResult,
};
use crate::parser::Parser;

const MAX: u32 = std::u32::MAX;

pub fn read_number_token(
    ctx: &mut Parser,
    starts_with_dot: bool,
    starts_with_zero: bool,
) -> TokenResult {
    let start = ctx.cursor;
    let mut result: String = String::new();
    let mut octal = starts_with_zero && ctx.cursor - start >= 2;
    if !starts_with_dot {
        result.push_str(&read_int(ctx, 10, true).to_string());
    }
    let mut next = get_cur_code_from_ctx(ctx);
    // The BigInt
    if !octal && !starts_with_dot && next == 110 {
        // 'n'
        ctx.cursor += 1;
        return get_token_from_map(TokenLabel::Number).map(|mut r| {
            let mut s = String::new();
            let mut iter = ctx.content.chars().skip(start);
            for _ in start..ctx.cursor {
                s.push(iter.next().unwrap());
            }
            r.value = Some(s);
            r
        });
    }

    if octal && (result.contains("8") || result.contains("9")) {
        octal = false;
    }

    // '.'
    if next == 46 && !octal {
        ctx.cursor += 1;
        result.push('.');
        result.push_str(&read_int(ctx, 10, true).to_string());
        next = get_cur_code_from_ctx(ctx);
    }

    if next == 69 || next == 101 {
        // 'e' or 'E'
        ctx.cursor += 1;
        result.push(if next == 69 { 'e' } else { 'E' });
        next = get_cur_code_from_ctx(ctx);
        if next == 43 || next == 45 {
            ctx.cursor += 1;
            result.push(if next == 43 { '+' } else { '-' });
        }
        result.push_str(&read_int(ctx, 10, true).to_string());
    }

    get_token_from_map(TokenLabel::Number).map(|mut r| {
        r.value = Some(result);
        r
    })
}

pub fn read_radix_int(ctx: &mut Parser, radix: u32) -> u32 {
    let (l, v) = eat_int(ctx, radix, MAX, true);
    if l == 0 {
        panic!("Expected number in radix {}", radix);
    }

    v
}

pub fn read_int(ctx: &mut Parser, radix: u32, allow_separators: bool) -> u32 {
    let (_, v) = eat_int(ctx, radix, MAX, allow_separators);

    v
}

/// 读取对应进制长度最大为 len 的数字
pub fn read_fixed_int(ctx: &mut Parser, radix: u32, len: u32, allow_separators: bool) -> u32 {
    let (l, v) = eat_int(ctx, radix, len, allow_separators);
    if l != len {
        panic!("Expected number in radix");
    }

    v
}

pub fn read_range_int(
    ctx: &mut Parser,
    radix: u32,
    len_range: (u32, u32),
    allow_separators: bool,
) -> u32 {
    let (min, max) = len_range;
    if min > max {
        panic!("The range left must be less than right");
    }
    let (l, v) = eat_int(ctx, radix, max, allow_separators);
    if l < min || l > max {
        panic!("Expected {} to {} numbers, got {} numbers.", min, max, l);
    }

    v
}

fn eat_int(ctx: &mut Parser, radix: u32, maxlen: u32, allow_separators: bool) -> (u32, u32) {
    let start = ctx.cursor;
    let mut couter = 0;
    let mut value: u32 = 0;
    let mut last_code = 0;
    let mut iter = ctx.content.chars().skip(ctx.cursor);
    while couter < maxlen {
        let code = iter.next().map(|r| r as u32).expect("Bad end of Number");
        let val: u32;
        if allow_separators && code == 95 {
            // _ 数字分割符
            if couter == 0 {
                panic!("Numeric separator is not allowed at the first of digits");
            }
            if last_code == 95 {
                panic!("Numeric separator must be exactly one underscore");
            }
            last_code = 95;
            ctx.cursor += 1;
            continue;
        }
        val = match code {
            48..=57 => code - 48,       // 0-9
            65..=90 => code - 65 + 10,  // 'A'
            97..=122 => code - 97 + 10, // 'a'
            _ => MAX,
        };
        if val >= radix {
            break;
        }
        couter += 1;
        ctx.cursor += 1;
        last_code = code;
        value = value * radix + val;
    }

    ((ctx.cursor - start) as u32, value)
}
