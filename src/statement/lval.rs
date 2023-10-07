use super::expression::assignment::parse_maybe_assign;
use super::expression::literal::parse_literal;
use super::expression::parse_ident;
use super::util::{after_trailing_comma, unexpected};
use crate::ast::_LocationNode;
use crate::ast::expression::{Expression, Identifier};
use crate::ast::pattern::{
    ArrayPattern, AssignmentPattern, AssignmentProperty, ObjectPattern, ObjectPatternProperty,
    Pattern, RestElement,
};
use crate::{parser::Parser, tokenizer::js_token::TokenLabel};
use std::vec;

// For object and array destructuring, there are two kinds of destructruing patterns, binding pattern and assignment pattern.
// There is a little difference between them,
// For binding pattern, which always starts with a declaration keyword (var、let or、const), eg.
// const obj = { a: 1, b: { c: 2 } };
// const {
//   a,
//   b: { c: d },
// } = obj;
// In assigment pattern, the pattern did not start with a keyword, each destructuring property
// was assigned to a target of assignment (as the name indicates), which was called LeftHandSideEpxression. eg.
// const numbers = [];
// const obj = { a: 1, b: 2 };
// ({ a: numbers[0], b: numbers[1] } = obj);

pub enum PatternAtom {
    ArrayPattern(ArrayPattern),
    ObjectPattern(ObjectPattern),
    Identifier(Identifier),
}

impl From<ArrayPattern> for PatternAtom {
    fn from(value: ArrayPattern) -> Self {
        Self::ArrayPattern(value)
    }
}

impl From<ObjectPattern> for PatternAtom {
    fn from(value: ObjectPattern) -> Self {
        Self::ObjectPattern(value)
    }
}

impl From<Identifier> for PatternAtom {
    fn from(value: Identifier) -> Self {
        Self::Identifier(value)
    }
}

impl From<PatternAtom> for Pattern {
    fn from(value: PatternAtom) -> Self {
        match value {
            PatternAtom::ArrayPattern(v) => Self::ArrayPattern(v),
            PatternAtom::ObjectPattern(v) => Self::ObjectPattern(v),
            PatternAtom::Identifier(v) => Self::Identifier(v),
        }
    }
}

// parse a pattern atom, the atom may be one of:
// - ArrayPattern, such as [a, b, ...c]
// - ObjectPattern, such as { a, b: b1, c = cDefault, ...d }
// - Identifier
// The RestElement and AssingmentPattern is not binding atom, because they only can appear in
// the body of pattern like ArrayPattern、ObjectPattern、function params list ect...
pub fn parse_binding_atom(ctx: &mut Parser) -> PatternAtom {
    if ctx.cur_token_is(TokenLabel::BracketL) {
        let start_loc = ctx.start_location_node();
        ctx.next_unwrap();
        let elements = parse_binding_list(ctx, TokenLabel::BracketR, true, true);
        return ArrayPattern::new(elements, ctx.compose_loc_info(start_loc)).into();
    }

    if ctx.cur_token_is(TokenLabel::BraceL) {
        return parse_object_binding(ctx).into();
    }

    parse_ident(ctx, false).into()
}

pub fn parse_object_binding(ctx: &mut Parser) -> ObjectPattern {
    let start_loc = ctx.start_location_node();
    let mut properties = vec![];
    ctx.expect(TokenLabel::BraceL);

    let mut first = true;
    while !ctx.eat(TokenLabel::BraceR) {
        if !first {
            ctx.expect(TokenLabel::Comma);
            if after_trailing_comma(ctx, TokenLabel::BraceR, true) {
                break;
            }
        } else {
            first = true;
        }

        if ctx.cur_token_is(TokenLabel::Ellipsis) {
            let property_start_loc = ctx.start_location_node();
            ctx.next_unwrap();
            let value = parse_ident(ctx, false);
            if ctx.cur_token_is(TokenLabel::Comma) {
                panic!("Comma is not permitted after the rest element");
            }

            properties.push(ObjectPatternProperty::RestElement(RestElement::new(
                Box::new(Pattern::Identifier(value)),
                ctx.compose_loc_info(property_start_loc),
            )));
            continue;
        }

        properties.push(ObjectPatternProperty::AssignmentProperty(
            parse_binding_property(ctx),
        ));
    }

    ObjectPattern::new(properties, ctx.compose_loc_info(start_loc))
}

pub fn parse_binding_property(ctx: &mut Parser) -> AssignmentProperty {
    let property_start_loc = ctx.start_location_node();
    let computed = ctx.eat(TokenLabel::BracketL);
    let key;
    if computed {
        key = parse_maybe_assign(ctx);
        ctx.expect(TokenLabel::BracketR);
    } else {
        key = if ctx
            .cur_token_test(|t| t.label == TokenLabel::String || t.label == TokenLabel::Number)
        {
            parse_literal(ctx).into()
        } else {
            parse_ident(ctx, true).into()
        };
    }

    let is_shorthand = !computed
        && (ctx.cur_token_is(TokenLabel::Comma)
            || ctx.cur_token_is(TokenLabel::Eq)
            || ctx.cur_token_is(TokenLabel::BraceR));

    if ctx.eat(TokenLabel::Colon) {
        return AssignmentProperty::new(
            key,
            parse_may_assignment_pattern(ctx),
            false,
            computed,
            ctx.compose_loc_info(property_start_loc),
        );
    }

    if is_shorthand {
        // only identifier key could be shorthand
        if let Expression::Identifier(ident) = &key {
            let value = if ctx.eat(TokenLabel::Eq) {
                let assi_start_loc = _LocationNode {
                    pos: ident.start,
                    loc: ident.loc.start.clone(),
                };
                Pattern::AssignmentPattern(AssignmentPattern::new(
                    Box::new(ident.clone().into()),
                    Box::new(parse_maybe_assign(ctx)),
                    ctx.compose_loc_info(assi_start_loc),
                ))
            } else {
                ident.clone().into()
            };
            return AssignmentProperty::new(
                key,
                value,
                true,
                false,
                ctx.compose_loc_info(property_start_loc),
            );
        }
    }

    unexpected(ctx.cur_token.clone().unwrap())
}

pub fn parse_rest_binding(ctx: &mut Parser) -> RestElement {
    let start_loc = ctx.start_location_node();
    ctx.next_unwrap();

    // TODO: in es6, this position must be an identifier
    let argument = parse_binding_atom(ctx);

    RestElement::new(Box::new(argument.into()), ctx.compose_loc_info(start_loc))
}

pub fn parse_binding_list(
    ctx: &mut Parser,
    close_label: TokenLabel,
    allow_empty: bool,
    allow_trailing_comma: bool,
) -> Vec<Option<Pattern>> {
    let mut elems: Vec<Option<Pattern>> = vec![];
    let mut first = true;
    while !ctx.eat(close_label) {
        if first {
            first = false;
        } else {
            ctx.expect(TokenLabel::Comma);
        }

        if allow_empty && ctx.cur_token_is(TokenLabel::Comma) {
            elems.push(None);
            continue;
        }
        if allow_trailing_comma && after_trailing_comma(ctx, close_label, true) {
            break;
        }
        // RestElement must be last element; And for object pattern, the argumemt of rest element must be identifier.
        if ctx.cur_token_is(TokenLabel::Ellipsis) {
            elems.push(Some(parse_rest_binding(ctx).into()));
            if ctx.cur_token_is(TokenLabel::Comma) {
                panic!("Comma is not permitted after the rest element");
            }
            ctx.expect(close_label);
            break;
        }

        elems.push(Some(parse_may_assignment_pattern(ctx)));
    }

    elems
}

pub fn parse_may_assignment_pattern(ctx: &mut Parser) -> Pattern {
    let start_loc = ctx.start_location_node();
    let left: Pattern = parse_binding_atom(ctx).into();
    if !ctx.eat(TokenLabel::Eq) {
        return left;
    }

    let right = parse_maybe_assign(ctx);

    Pattern::AssignmentPattern(AssignmentPattern::new(
        Box::new(left),
        Box::new(right),
        ctx.compose_loc_info(start_loc),
    ))
}
