use super::js_token::{Token, TokenLabel};
use super::util::has_break_in_range;
use crate::parser::Parser;
use std::collections::HashMap;

lazy_static! {
    pub static ref TokenContextMap: HashMap<TokenContextLabel, TokenContext> = HashMap::from([
        (TokenContextLabel::BraceStat, TokenContext::default()),
        (
            TokenContextLabel::BraceExpr,
            TokenContext {
                label: TokenContextLabel::BraceExpr,
                is_expr: true,
                ..TokenContext::default()
            }
        ),
        (
            TokenContextLabel::BraceTmpl,
            TokenContext {
                label: TokenContextLabel::BraceTmpl,
                ..TokenContext::default()
            }
        ),
        (
            TokenContextLabel::ParenStat,
            TokenContext {
                label: TokenContextLabel::ParenStat,
                ..TokenContext::default()
            }
        ),
        (
            TokenContextLabel::ParenExpr,
            TokenContext {
                label: TokenContextLabel::ParenExpr,
                is_expr: true,
                ..TokenContext::default()
            }
        ),
        (
            TokenContextLabel::QuoteTmpl,
            TokenContext {
                label: TokenContextLabel::QuoteTmpl,
                is_expr: true,
                preserve_space: true,
                unusual: true,
                ..TokenContext::default()
            }
        ),
        (
            TokenContextLabel::FnStat,
            TokenContext {
                label: TokenContextLabel::FnStat,
                ..TokenContext::default()
            }
        ),
        (
            TokenContextLabel::FnExpr,
            TokenContext {
                label: TokenContextLabel::FnExpr,
                is_expr: true,
                ..TokenContext::default()
            }
        ),
        (
            TokenContextLabel::FnExprGen,
            TokenContext {
                label: TokenContextLabel::FnExprGen,
                is_expr: true,
                generator: true,
                ..TokenContext::default()
            }
        ),
        (
            TokenContextLabel::FnGen,
            TokenContext {
                label: TokenContextLabel::FnGen,
                generator: true,
                ..TokenContext::default()
            }
        ),
    ]);
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TokenContextLabel {
    BraceStat, // "{"
    BraceExpr,
    BraceTmpl, // " &{"
    ParenStat, // "("
    ParenExpr,
    QuoteTmpl, // "`"
    FnStat,    // "function"
    FnExpr,
    FnExprGen,
    FnGen,
}

#[derive(Debug, Clone)]
pub struct TokenContext {
    pub label: TokenContextLabel,
    pub is_expr: bool,
    pub preserve_space: bool,
    pub generator: bool,
    pub unusual: bool,
}

impl Default for TokenContext {
    fn default() -> Self {
        Self {
            label: TokenContextLabel::BraceStat,
            is_expr: false,
            preserve_space: false,
            generator: false,
            unusual: false,
        }
    }
}

pub fn update_token_context(ctx: &mut Parser) {
    let cur_token = ctx.cur_token.as_ref();
    let prev_token = ctx.prev_token.as_ref();

    if cur_token.map_or(false, |t| t.keyword)
        && prev_token.map_or(false, |pt| pt.is_token_of(TokenLabel::Dot))
    {
        ctx.expr_allowed = true;
    } else if cur_token.map_or(false, |t| t.update_ctx) {
        match cur_token.map(|t| t.label.clone()).unwrap() {
            TokenLabel::ParenR | TokenLabel::BraceR => {
                update_parenr_bracer_ctx(ctx);
            }
            TokenLabel::BraceL => {
                update_bracel_ctx(ctx);
            }
            TokenLabel::DollarBraceL => {
                update_dollar_brace_ctx(ctx);
            }
            TokenLabel::ParenL => {
                update_parenl_ctx(ctx);
            }
            TokenLabel::_Function | TokenLabel::_Class => {
                update_fn_class_ctx(ctx);
            }
            TokenLabel::BackQuote => {
                update_backquote_ctx(ctx);
            }
            TokenLabel::Star => {
                update_star_ctx(ctx);
            }
            TokenLabel::Name => {
                update_name_ctx(ctx);
            }
            _ => {}
        }
    } else {
        ctx.expr_allowed = cur_token.map_or(false, |t| t.before_expr);
    }
}

fn update_parenr_bracer_ctx(ctx: &mut Parser) {
    if ctx.token_context.len() == 1 {
        ctx.expr_allowed = true;
        return;
    }
    let mut token_ctx = ctx.token_context.pop();
    if token_ctx
        .as_ref()
        .map_or(false, |t| t.label == TokenContextLabel::BraceStat)
        && ctx
            .cur_token_ctx()
            .map_or(false, |t| t.label == TokenContextLabel::FnStat)
    {
        token_ctx = ctx.token_context.pop();
    }

    ctx.expr_allowed = token_ctx.map_or(false, |t| t.is_expr);
}

fn update_bracel_ctx(ctx: &mut Parser) {
    ctx.token_context.push(if brace_is_block(ctx) {
        get_context_by_label(TokenContextLabel::BraceStat)
    } else {
        get_context_by_label(TokenContextLabel::BraceExpr)
    });
    ctx.expr_allowed = true;
}

fn update_dollar_brace_ctx(ctx: &mut Parser) {
    ctx.token_context
        .push(get_context_by_label(TokenContextLabel::BraceTmpl));
    ctx.expr_allowed = true;
}

fn update_parenl_ctx(ctx: &mut Parser) {
    let next_ctx = match &ctx.prev_token {
        Some(Token { label, .. }) => match label {
            TokenLabel::_If | TokenLabel::_For | TokenLabel::_With | TokenLabel::_While => {
                get_context_by_label(TokenContextLabel::ParenStat)
            }
            _ => get_context_by_label(TokenContextLabel::ParenExpr),
        },
        None => get_context_by_label(TokenContextLabel::ParenExpr),
    };

    ctx.token_context.push(next_ctx);
    ctx.expr_allowed = true;
}

fn update_fn_class_ctx(ctx: &mut Parser) {
    match &ctx.prev_token {
        Some(pt) => {
            let cur_ctx = ctx.cur_token_ctx();
            if !pt.before_expr || pt.label == TokenLabel::_Else {
                ctx.token_context
                    .push(get_context_by_label(TokenContextLabel::FnStat));
            } else if !(pt.label == TokenLabel::Semi
                && cur_ctx.map_or(true, |c| c.label != TokenContextLabel::ParenStat))
                && !(pt.label == TokenLabel::_Return
                    && has_break_in_range(ctx, (ctx.last_token_end, ctx.cur_token_start)))
                && !((pt.label == TokenLabel::Colon || pt.label == TokenLabel::BraceL)
                    && cur_ctx.map_or(false, |c| c.label == TokenContextLabel::BraceStat))
            {
                ctx.token_context
                    .push(get_context_by_label(TokenContextLabel::FnExpr));
            }
        }
        None => {
            ctx.token_context
                .push(get_context_by_label(TokenContextLabel::FnStat));
        }
    };

    ctx.expr_allowed = false;
}

fn update_backquote_ctx(ctx: &mut Parser) {
    let cur_ctx = ctx.cur_token_ctx();
    if cur_ctx.map_or(false, |c| c.label == TokenContextLabel::QuoteTmpl) {
        ctx.token_context.pop();
    } else {
        ctx.token_context
            .push(get_context_by_label(TokenContextLabel::QuoteTmpl));
    }

    ctx.expr_allowed = false;
}

fn update_star_ctx(ctx: &mut Parser) {
    if ctx
        .prev_token
        .as_ref()
        .map_or(false, |t| t.label == TokenLabel::_Function)
    {
        let cur_ctx_op = ctx.cur_token_ctx();

        if cur_ctx_op.map_or(false, |c| c.label == TokenContextLabel::FnExpr) {
            ctx.token_context.pop();
            ctx.token_context
                .push(get_context_by_label(TokenContextLabel::FnExprGen));
        } else {
            ctx.token_context.pop();
            ctx.token_context
                .push(get_context_by_label(TokenContextLabel::FnGen));
        }
    }

    ctx.expr_allowed = true;
}

fn update_name_ctx(ctx: &mut Parser) {
    ctx.expr_allowed = if ctx.prev_token.as_ref().map_or(false, |t| {
        t.label != TokenLabel::Dot
            && (t.value.as_ref().map_or(false, |v| v == "of") && !ctx.expr_allowed
                || t.value.as_ref().map_or(false, |v| v == "yield") && ctx.in_generator_ctx())
    }) {
        true
    } else {
        false
    }
}

pub fn get_context_by_label(label: TokenContextLabel) -> TokenContext {
    TokenContextMap.get(&label).map(|t| t.clone()).unwrap()
}

fn brace_is_block(ctx: &Parser) -> bool {
    let prev_token = &ctx.prev_token;
    let parent_ctx = ctx.cur_token_ctx();
    if parent_ctx.map_or(false, |t| {
        t.label == TokenContextLabel::FnExpr || t.label == TokenContextLabel::FnStat
    }) {
        return true;
    }

    match prev_token {
        Some(Token { label, .. }) => match label {
            TokenLabel::Colon => {
                if parent_ctx.map_or(false, |c| {
                    c.label == TokenContextLabel::BraceStat
                        || c.label == TokenContextLabel::BraceExpr
                }) {
                    return !parent_ctx.unwrap().is_expr;
                }
            }
            TokenLabel::_Return => {
                return has_break_in_range(ctx, (ctx.last_token_end, ctx.cur_token_start));
            }
            name if name == &TokenLabel::Name && ctx.expr_allowed => {
                return has_break_in_range(ctx, (ctx.last_token_end, ctx.cur_token_start));
            }
            TokenLabel::_Else
            | TokenLabel::Semi
            | TokenLabel::Eof
            | TokenLabel::ParenR
            | TokenLabel::Arrow => {
                return true;
            }
            TokenLabel::BraceL => {
                return parent_ctx.map_or(false, |c| c.label == TokenContextLabel::BraceStat)
            }
            TokenLabel::_Var | TokenLabel::_Const | TokenLabel::Name => {
                return false;
            }
            _ => {}
        },
        None => {}
    }

    !ctx.expr_allowed
}
