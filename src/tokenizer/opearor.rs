use super::{
    js_token::TokenLabel,
    number::read_number_token,
    regex::read_regex_token,
    util::{get_next_code_from_ctx, get_next_code_from_iter, get_token_from_map},
    TokenResult,
};
use crate::parser::Parser;

// read token which starts with '?'
pub fn read_token_from_question(ctx: &mut Parser) -> TokenResult {
    let mut iter = ctx.content.chars().skip(ctx.cursor + 1);

    let next_code = get_next_code_from_iter(&mut iter);
    // .
    if next_code == 46 {
        let nnext_code = get_next_code_from_iter(&mut iter);
        if nnext_code < 48 && nnext_code > 57 {
            ctx.cursor += 2;
            return get_token_from_map(TokenLabel::QuestionDot);
        }
    }

    // '?'
    if next_code == 63 {
        let nnext_code = get_next_code_from_iter(&mut iter);
        // '='
        if nnext_code == 61 {
            ctx.cursor += 3;
            return get_token_from_map(TokenLabel::Assign).map(|mut r| {
                r.value = Some("??=".to_string());
                r
            });
        }
        ctx.cursor += 2;
        return get_token_from_map(TokenLabel::Coalesce);
    }

    ctx.cursor += 1;
    get_token_from_map(TokenLabel::Question)
}

// read token which starts with '>' or '<'
pub fn read_token_from_lt_rt(ctx: &mut Parser, code: usize) -> TokenResult {
    let mut iter = ctx.content.chars().skip(ctx.cursor + 1);
    let next_code = get_next_code_from_iter(&mut iter);
    if next_code == code {
        let nnext_code = get_next_code_from_iter(&mut iter);
        // '>>>'
        if next_code == 62 && nnext_code == 62 {
            // '>>>='
            if get_next_code_from_iter(&mut iter) == 61 {
                ctx.cursor += 4;
                return get_token_from_map(TokenLabel::Assign).map(|mut r| {
                    r.value = Some(">>>=".to_string());
                    r
                });
            }

            ctx.cursor += 3;
            return get_token_from_map(TokenLabel::BitShift).map(|mut r| {
                r.value = Some(">>>".to_string());
                r
            });
        }

        // '<<=' '>>='
        if nnext_code == 61 {
            ctx.cursor += 3;
            return get_token_from_map(TokenLabel::Assign).map(|mut r| {
                r.value = if code == 62 {
                    Some(">>=".to_string())
                } else {
                    Some("<<=".to_string())
                };
                r
            });
        }

        ctx.cursor += 2;
        return get_token_from_map(TokenLabel::BitShift).map(|mut r| {
            r.value = if code == 62 {
                Some(">>".to_string())
            } else {
                Some("<<".to_string())
            };
            r
        });
    }

    get_token_from_map(TokenLabel::Relational).map(|mut r| {
        let is_included = next_code == 61;
        r.value = if code == 60 {
            if is_included {
                Some(">=".to_string())
            } else {
                Some(">".to_string())
            }
        } else {
            if is_included {
                Some("<=".to_string())
            } else {
                Some("<".to_string())
            }
        };
        ctx.cursor += if is_included { 2 } else { 1 };
        r
    })
}

// read token which starts with '+' or '-'
pub fn read_token_from_plus_min(ctx: &mut Parser, code: usize) -> TokenResult {
    let next_code = get_next_code_from_ctx(ctx);
    if next_code == code {
        ctx.cursor += 2;
        return get_token_from_map(TokenLabel::IncDec).map(|mut r| {
            r.value = if code == 43 {
                Some("++".to_string())
            } else {
                Some("--".to_string())
            };
            r
        });
    }

    if next_code == 61 {
        ctx.cursor += 2;
        return get_token_from_map(TokenLabel::Assign).map(|mut r| {
            r.value = if code == 43 {
                Some("+=".to_string())
            } else {
                Some("-=".to_string())
            };
            r
        });
    }

    get_token_from_map(TokenLabel::PlusMin).map(|mut r| {
        r.value = if code == 43 {
            Some("+".to_string())
        } else {
            Some("-".to_string())
        };
        r
    })
}

// read token which starts with '/', may could be regex
pub fn read_slash_token(ctx: &mut Parser) -> TokenResult {
    // TODO: 补充判断是否允许解析正则的实现
    if ctx.expr_allowed {
        ctx.cursor += 1;
        return read_regex_token(ctx);
    }
    let mut chars = ctx.content.chars().skip(ctx.cursor + 1);
    if get_next_code_from_iter(&mut chars) == 61 {
        ctx.cursor += 2;
        return get_token_from_map(TokenLabel::Assign).map(|mut r| {
            r.value = Some("\\=".to_string());
            r
        });
    }

    ctx.cursor += 1;
    get_token_from_map(TokenLabel::Slash)
}

// read token which starts with '!' or '='
pub fn read_token_eq_excl(ctx: &mut Parser, c: usize) -> TokenResult {
    let mut iter = ctx.content.chars().skip(ctx.cursor + 1);
    let next_code = get_next_code_from_iter(&mut iter);
    if next_code == 61 {
        let nnext_code = get_next_code_from_iter(&mut iter);
        return get_token_from_map(TokenLabel::Equlity).map(|mut r| {
            r.value = if nnext_code == 61 {
                ctx.cursor += 3;
                if c == 61 {
                    Some("===".to_string())
                } else {
                    Some("!==".to_string())
                }
            } else {
                ctx.cursor += 2;
                if c == 61 {
                    Some("==".to_string())
                } else {
                    Some("!=".to_string())
                }
            };
            r
        });
    }

    if c == 61 && next_code == 62 {
        ctx.cursor += 2;
        return get_token_from_map(TokenLabel::Arrow);
    }

    ctx.cursor += 1;
    if c == 61 {
        get_token_from_map(TokenLabel::Eq)
    } else {
        get_token_from_map(TokenLabel::Prefix)
    }
}

// read token which starts with '%'
pub fn read_modulo_token(ctx: &mut Parser) -> TokenResult {
    let mut chars = ctx.content.chars().skip(ctx.cursor + 1);
    let next_code = get_next_code_from_iter(&mut chars);
    if next_code == 61 {
        ctx.cursor += 2;
        return get_token_from_map(TokenLabel::Assign);
    }

    ctx.cursor += 1;
    get_token_from_map(TokenLabel::Modulo)
}

// read token which starts with '*'
pub fn read_star_token(ctx: &mut Parser) -> TokenResult {
    let mut chars = ctx.content.chars().skip(ctx.cursor + 1);
    let next_code = get_next_code_from_iter(&mut chars);
    let nnext_code = get_next_code_from_iter(&mut chars);
    // '*'
    if next_code == 42 {
        // '='
        if nnext_code == 61 {
            ctx.cursor += 3;
            return get_token_from_map(TokenLabel::Assign).map(|mut r| {
                r.value = Some("**=".to_string());
                r
            });
        }

        ctx.cursor += 2;
        return get_token_from_map(TokenLabel::StarStar);
    }

    ctx.cursor += 1;
    get_token_from_map(TokenLabel::Star)
}

// read token which starts with '^'
pub fn read_caret_token(ctx: &mut Parser) -> TokenResult {
    let next_code = get_next_code_from_ctx(ctx);
    if next_code == 61 {
        ctx.cursor += 2;
        return get_token_from_map(TokenLabel::Assign).map(|mut r| {
            r.value = Some("^=".to_string());
            r
        });
    }

    ctx.cursor += 1;
    get_token_from_map(TokenLabel::BitwiseXor)
}

// read token which starts with '|' or '&'
pub fn read_pipe_amp_token(ctx: &mut Parser, code: usize) -> TokenResult {
    let mut chars = ctx.content.chars().skip(ctx.cursor + 1);
    let next_code = get_next_code_from_iter(&mut chars);
    if next_code == code {
        if get_next_code_from_iter(&mut chars) == 61 {
            ctx.cursor += 3;
            return get_token_from_map(TokenLabel::Assign).map(|mut r| {
                r.value = if code == 124 {
                    Some("&&=".to_string())
                } else {
                    Some("||=".to_string())
                };
                r
            });
        }
        ctx.cursor += 2;

        return if code == 124 {
            get_token_from_map(TokenLabel::LogicalOr)
        } else {
            get_token_from_map(TokenLabel::LogicalAnd)
        };
    }

    if get_next_code_from_iter(&mut chars) == 61 {
        ctx.cursor += 2;
        return get_token_from_map(TokenLabel::Assign).map(|mut r| {
            r.value = if code == 124 {
                Some("&=".to_string())
            } else {
                Some("|=".to_string())
            };
            r
        });
    }

    ctx.cursor += 1;
    if code == 124 {
        get_token_from_map(TokenLabel::BitwiseOr)
    } else {
        get_token_from_map(TokenLabel::BitwiseAnd)
    }
}

// read token which starts with '.'
pub fn read_dot_token(ctx: &mut Parser) -> TokenResult {
    let mut iter = ctx.content.chars().skip(ctx.cursor + 1);
    if let Some(next_code) = iter.next().map(|r| r as usize) {
        // 0-9
        if next_code >= 48 && next_code <= 57 {
            return read_number_token(ctx, true, false);
        }
        let nnext_code = get_next_code_from_iter(&mut iter);
        if nnext_code == 46 && next_code == 46 {
            ctx.cursor += 3;
            return get_token_from_map(TokenLabel::Ellipsis);
        }
    }

    ctx.cursor += 1;
    get_token_from_map(TokenLabel::Dot)
}
