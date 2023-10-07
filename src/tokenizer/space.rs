use super::{
    get_cur_code_from_ctx, get_next_code_from_ctx,
    util::{is_new_line, is_non_ascii_white_space, move_to_next_line},
};
use crate::parser::Parser;

pub fn skip_space_comment(ctx: &mut Parser) {
    loop {
        let code = get_cur_code_from_ctx(ctx);
        match code {
            // ' '
            32 | 160 => {
                ctx.cursor += 1;
            }
            13 => {
                ctx.cursor += 1;
                if get_next_code_from_ctx(ctx) == 10 {
                    ctx.cursor += 1;
                }
                move_to_next_line(ctx);
            }
            // '\r' '\n'
            10 | 8232 | 8233 => {
                ctx.cursor += 1;
                move_to_next_line(ctx);
            }
            // '/'
            47 => {
                match get_next_code_from_ctx(ctx) {
                    // '*'
                    42 => {
                        skip_block_comment(ctx);
                    }
                    // '/'
                    47 => {
                        skip_line_comment(ctx);
                    }
                    _ => break,
                }
                ctx.cursor += 1;
            }
            9 | 11 | 12 => {
                ctx.cursor += 1;
            }
            c @ _ => {
                if is_non_ascii_white_space(c) {
                    ctx.cursor += 1;
                } else {
                    break;
                }
            }
        }
    }
}

fn skip_block_comment(ctx: &mut Parser) {
    ctx.cursor += 2;
    let mut pos = ctx.cursor;
    let mut last_is_star = false;
    let mut line_inc = 0;
    let mut line_start = ctx.line_start;

    loop {
        let cur_code = ctx.codes.get(pos);
        if cur_code.is_none() {
            break;
        }
        let c = *cur_code.unwrap();
        if c == 47 && last_is_star {
            pos += 1;
            break;
        } else if c == 42 {
            last_is_star = true;
        } else if is_new_line(c) {
            if c == 13 && ctx.codes.get(pos + 1).map_or(false, |next_c| *next_c == 10) {
                pos += 1;
            }
            line_start = pos;
            line_inc += 1;
        }
        pos += 1;
    }

    ctx.cursor = pos;
    ctx.line += line_inc;
    ctx.line_start = line_start;
}

fn skip_line_comment(ctx: &mut Parser) {
    ctx.cursor += 2;
    let mut pos = ctx.cursor;
    let mut incre_line = true;
    loop {
        let next_c_op = ctx.codes.get(pos);
        if next_c_op.is_none() {
            incre_line = false;
            break;
        }
        let c = *next_c_op.unwrap();
        if is_new_line(c) {
            pos += 1;
            // handle windows break line '\r\n'
            if c == 13
                && ctx
                    .codes
                    .get(pos)
                    .map_or(false, |next_code| *next_code == 10)
            {
                pos += 1;
            }
            break;
        }
        pos += 1;
    }

    ctx.cursor = pos;
    if incre_line {
        move_to_next_line(ctx);
    }
}
