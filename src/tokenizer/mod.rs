pub mod context;
pub mod js_token;
pub mod number;
pub mod opearor;
pub mod regex;
pub mod space;
pub mod string;
pub mod template;
pub mod util;

use self::context::update_token_context;
use self::js_token::{Token, TokenLabel, TokenMap};
use self::number::{read_number_token, read_radix_int};
use self::opearor::{
    read_caret_token, read_dot_token, read_modulo_token, read_pipe_amp_token, read_slash_token,
    read_star_token, read_token_eq_excl, read_token_from_lt_rt, read_token_from_plus_min,
    read_token_from_question,
};
use self::space::skip_space_comment;
use self::string::read_string_token;
use self::util::{
    get_content_len, get_cur_code_from_ctx, get_next_code_from_ctx, get_token_from_map,
    is_identifier_char, is_keyword_token,
};
use crate::parser::Parser;
use crate::statement::util::unexpected;
use crate::tokenizer::template::read_template_token;
use std::{error::Error, fmt::Display, usize};

pub type TokenResult = Result<js_token::Token, Box<dyn Error>>;

#[derive(Debug)]
enum TokenError {
    KeywordNotExist,
    TokenNotFoundError,
    UnexpectedCharactorError,
}

impl Display for TokenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::KeywordNotExist => write!(f, "Keyword did not exist."),
            _ => write!(f, "Others error."),
        }
    }
}

impl Error for TokenError {}

pub fn next_token(ctx: &mut Parser) -> TokenResult {
    if ctx
        .cur_token_ctx()
        .map_or(true, |token_ctx| !token_ctx.preserve_space)
    {
        skip_space_comment(ctx);
    }

    ctx.cur_token_start = ctx.cursor;
    ctx.cur_token_start_loc = Some(ctx.get_cursor_position());

    if ctx
        .cur_token_ctx()
        .map_or(false, |token_ctx| token_ctx.unusual)
    {
        return read_unusual_token(ctx).map(|t| finish_token(ctx, t));
    }

    let is_end = get_cur_code_from_ctx(ctx) == 0 && ctx.cursor == get_content_len(ctx);
    let result = if is_end {
        Ok(get_token_from_map(TokenLabel::Eof).unwrap())
    } else {
        read_token(ctx)
    };

    result.map(|r| finish_token(ctx, r))
}

pub fn read_unusual_token(ctx: &mut Parser) -> TokenResult {
    let cur_token_ctx = ctx.cur_token_ctx().unwrap();
    match cur_token_ctx.label {
        context::TokenContextLabel::QuoteTmpl => read_template_token(ctx),
        _ => unexpected(ctx.cur_token.clone().unwrap()),
    }
}

pub fn read_token(ctx: &mut Parser) -> TokenResult {
    let first_char = get_cur_code_from_ctx(ctx);

    if util::is_identifier_start(first_char) {
        let word = read_word(ctx);
        if let Some(t) = is_keyword_token(&word) {
            return TokenMap
                .get(t)
                .map(|r| r.clone())
                .ok_or(TokenError::KeywordNotExist.into());
        } else {
            return get_token_from_map(TokenLabel::Name).map(|mut r| {
                r.value = Some(word);
                r
            });
        }
    }

    read_token_by_code(ctx, first_char)
}

pub fn read_token_by_code(ctx: &mut Parser, code: usize) -> TokenResult {
    match code {
        // '!' or '='
        c @ (33 | 61) => read_token_eq_excl(ctx, c),
        c @ (34 | 39) => {
            // ''"'
            ctx.cursor += 1;
            read_string_token(ctx, c)
        }
        // '#'
        35 => read_token_num_sign(ctx),
        // '%'
        37 => read_modulo_token(ctx), // %
        // '|&'
        38 | 124 => read_pipe_amp_token(ctx, code),
        40 => {
            ctx.cursor += 1;
            get_token_from_map(TokenLabel::ParenL)
        }
        41 => {
            ctx.cursor += 1;
            get_token_from_map(TokenLabel::ParenR)
        }
        42 => read_star_token(ctx),
        c @ (43 | 45) => read_token_from_plus_min(ctx, c),
        44 => {
            ctx.cursor += 1;
            get_token_from_map(TokenLabel::Comma)
        }
        // .
        46 => read_dot_token(ctx),
        // '/'
        47 => read_slash_token(ctx),
        // 0..9
        c @ 48..=57 => {
            if c == 48 {
                let mut iter = ctx.content.chars().skip(ctx.cursor + 1);
                if let Some(next) = iter.next().map(|v| v as usize) {
                    if next == 120 || next == 88 {
                        // 读取 0x 0X 等十六进制数字
                        ctx.cursor += 2;
                        let v = read_radix_int(ctx, 16);
                        return get_token_from_map(TokenLabel::Number).map(|mut r| {
                            r.value = Some(v.to_string());
                            r
                        });
                    }
                    if next == 111 || next == 79 {
                        // 读取 0o 0O 等八进制数字
                        ctx.cursor += 2;
                        let v = read_radix_int(ctx, 8);
                        return get_token_from_map(TokenLabel::Number).map(|mut r| {
                            r.value = Some(v.to_string());
                            r
                        });
                    }
                    if next == 98 || next == 66 {
                        // 读取 0b 0B 等二进制数字
                        ctx.cursor += 2;
                        let v = read_radix_int(ctx, 2);
                        return get_token_from_map(TokenLabel::Number).map(|mut r| {
                            r.value = Some(v.to_string());
                            r
                        });
                    }
                }
            }

            read_number_token(ctx, false, c == 48)
        }
        58 => {
            ctx.cursor += 1;
            get_token_from_map(TokenLabel::Colon)
        }
        59 => {
            ctx.cursor += 1;
            get_token_from_map(TokenLabel::Semi)
        }
        // '>' or '<'
        c @ (60 | 62) => read_token_from_lt_rt(ctx, c),
        // '?'
        63 => read_token_from_question(ctx),
        91 => {
            ctx.cursor += 1;
            get_token_from_map(TokenLabel::BracketL)
        }
        93 => {
            ctx.cursor += 1;
            get_token_from_map(TokenLabel::BracketR)
        }
        94 => read_caret_token(ctx),
        // '`'
        96 => {
            ctx.cursor += 1;
            get_token_from_map(TokenLabel::BackQuote)
        }
        123 => {
            ctx.cursor += 1;
            get_token_from_map(TokenLabel::BraceL)
        }
        125 => {
            ctx.cursor += 1;
            get_token_from_map(TokenLabel::BraceR)
        }
        126 => {
            ctx.cursor += 1;
            get_token_from_map(TokenLabel::Prefix)
        }
        _ => Err(TokenError::UnexpectedCharactorError.into()),
    }
}

fn read_token_num_sign(ctx: &mut Parser) -> TokenResult {
    let next_code = get_next_code_from_ctx(ctx);
    if is_identifier_char(next_code) {
        ctx.cursor += 1;
        let word = read_word(ctx);
        return get_token_from_map(TokenLabel::PrivateId).map(|mut r| {
            r.value = Some(word);
            r
        });
    }

    Err(TokenError::UnexpectedCharactorError.into())
}

fn read_word(ctx: &mut Parser) -> String {
    let start_pos = ctx.cursor;
    let mut new_pos = ctx.cursor;
    let mut result = String::new();
    for (i, c) in ctx.content.char_indices().skip(start_pos) {
        if i == start_pos && util::is_identifier_start(c as usize)
            || (i != start_pos && util::is_identifier_char(c as usize))
        {
            new_pos += 1;
            result.push(c);
        } else {
            break;
        }
    }

    ctx.cursor = new_pos;
    result
}

pub fn finish_token(ctx: &mut Parser, token: Token) -> Token {
    ctx.prev_token = ctx.cur_token.clone();
    ctx.cur_token_end = ctx.cursor;
    ctx.cur_token_end_loc = Some(ctx.get_cursor_position());
    ctx.cur_token = Some(token.clone());

    update_token_context(ctx);

    token
}
