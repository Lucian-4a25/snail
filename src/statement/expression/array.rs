use super::{
    assignment::parse_maybe_assign,
    object::{build_obj_ir_as_expr, build_obj_ir_as_pattern, parse_ir_value, Object_IR},
    util::{get_assign_left_ident, get_paren_expr_val},
};
use crate::ast::_LocationNode;
use crate::ast::expression::{
    ArrayExprEle, ArrayExpression, AssignmentExpression, AssignmentExpressionLeft,
    AssignmentOperator, Expression, SpreadElement,
};
use crate::ast::pattern::{ArrayPattern, AssignmentPattern, Pattern, RestElement};
use crate::{
    parser::Parser, statement::util::after_trailing_comma, tokenizer::js_token::TokenLabel,
};

// represents an array expression or array pattern
#[allow(non_camel_case_types)]
pub struct Array_IR {
    pub start_loc: _LocationNode,
    pub end_loc: _LocationNode,
    pub source: Option<String>,
    pub elements: Vec<Array_Element_IR>,
    /// indicate it must be a array pattern.
    pub assert_destructuring: bool,
    /// indicate it must be a array expression.
    pub assert_expr: bool,
}

#[allow(non_camel_case_types)]
pub struct Array_Element_IR {
    pub start_loc: _LocationNode,
    pub end_loc: _LocationNode,
    // if starts with '...'
    pub is_dots: bool,
    pub value: Element_IR_Value,
    pub rhs_expr: Option<Expression>,
}

#[allow(non_camel_case_types)]
pub enum Element_IR_Value {
    Elision,                // represents case like, [,,]
    Expression(Expression), // expression except Object_IR or Array_IR
    Object_IR(Object_IR),   // for case like: [{ a: { b, c.. }, }]
    Array_IR(Array_IR),     // for case like: [[a], ]
}

pub fn build_array_ir_as_expr(arr_ir: Array_IR, strict_mode: bool) -> ArrayExpression {
    if arr_ir.assert_destructuring {
        panic!("Invalid array initializer")
    }

    let mut els = vec![];
    for element in arr_ir.elements {
        match element.value {
            Element_IR_Value::Array_IR(arr_ir_val) => {
                let arr_ir_val_start_loc = arr_ir_val.start_loc.clone();
                let has_rhs_expr = element.rhs_expr.is_some();
                let el_val = if has_rhs_expr {
                    Expression::AssignmentExpression(AssignmentExpression::new(
                        AssignmentExpressionLeft::Pattern(Pattern::ArrayPattern(
                            build_array_ir_as_pattern(arr_ir_val, strict_mode),
                        )),
                        AssignmentOperator::Assignment,
                        Box::new(element.rhs_expr.unwrap()),
                        (
                            arr_ir_val_start_loc,
                            element.end_loc.clone(),
                            arr_ir.source.clone(),
                        ),
                    ))
                } else {
                    Expression::ArrayExpression(build_array_ir_as_expr(arr_ir_val, strict_mode))
                };
                // for case: [...[w] = [1, 2, 3]]
                els.push(if element.is_dots {
                    ArrayExprEle::SpreadElement(SpreadElement::new(
                        el_val,
                        (element.start_loc, element.end_loc, arr_ir.source.clone()),
                    ))
                } else {
                    ArrayExprEle::Expression(el_val)
                });
            }
            Element_IR_Value::Object_IR(obj_ir_val) => {
                let has_rhs_expr = element.rhs_expr.is_some();
                let object_ir_start_loc = obj_ir_val.start_loc.clone();
                let el_val = if has_rhs_expr {
                    Expression::AssignmentExpression(AssignmentExpression::new(
                        AssignmentExpressionLeft::Pattern(Pattern::ObjectPattern(
                            build_obj_ir_as_pattern(obj_ir_val, strict_mode),
                        )),
                        AssignmentOperator::Assignment,
                        Box::new(element.rhs_expr.unwrap()),
                        (
                            object_ir_start_loc,
                            element.end_loc.clone(),
                            arr_ir.source.clone(),
                        ),
                    ))
                } else {
                    Expression::ObjectExpression(build_obj_ir_as_expr(obj_ir_val, strict_mode))
                };
                // [...{} = {} ]
                els.push(if element.is_dots {
                    ArrayExprEle::SpreadElement(SpreadElement::new(
                        el_val,
                        (element.start_loc, element.end_loc, arr_ir.source.clone()),
                    ))
                } else {
                    ArrayExprEle::Expression(el_val)
                });
            }
            Element_IR_Value::Expression(expr_val) => {
                els.push(if element.is_dots {
                    ArrayExprEle::SpreadElement(SpreadElement::new(
                        expr_val,
                        (element.start_loc, element.end_loc, arr_ir.source.clone()),
                    ))
                } else {
                    ArrayExprEle::Expression(expr_val)
                });
            }
            Element_IR_Value::Elision => {
                els.push(ArrayExprEle::Null);
            }
        }
    }

    ArrayExpression::new(els, (arr_ir.start_loc, arr_ir.end_loc, arr_ir.source))
}

pub fn build_array_ir_as_pattern(arr_ir: Array_IR, strict_mode: bool) -> ArrayPattern {
    // perfer pattern if array_ir got rhs_expr.
    if arr_ir.assert_expr {
        panic!("Invalid destructuring assignment target");
    }

    let mut els: Vec<Option<Pattern>> = vec![];
    for element in arr_ir.elements {
        match element.value {
            Element_IR_Value::Elision => {
                els.push(None);
            }
            Element_IR_Value::Array_IR(arr_ir_val) => {
                let arr_ir_val_start_loc = arr_ir_val.start_loc.clone();
                let has_rhs_expr = element.rhs_expr.is_some();
                let arr_pat = build_array_ir_as_pattern(arr_ir_val, strict_mode);
                let mut ele_val = if has_rhs_expr {
                    Pattern::AssignmentPattern(AssignmentPattern::new(
                        Box::new(Pattern::ArrayPattern(arr_pat)),
                        Box::new(element.rhs_expr.unwrap()),
                        (
                            arr_ir_val_start_loc,
                            element.end_loc.clone(),
                            arr_ir.source.clone(),
                        ),
                    ))
                } else {
                    Pattern::ArrayPattern(arr_pat)
                };

                if element.is_dots {
                    ele_val = Pattern::RestElement(RestElement::new(
                        Box::new(ele_val),
                        (element.start_loc, element.end_loc, arr_ir.source.clone()),
                    ));
                }
                els.push(Some(ele_val));
            }
            // panic if it's not a simple AssignmentTargetType
            Element_IR_Value::Expression(expr_val) => {
                let outer_is_paren = matches!(&expr_val, Expression::ParenthesizedExpression(..));
                let v = get_paren_expr_val(expr_val);
                match v {
                    Expression::MemberExpression(member_expr) => {
                        els.push(Some(Pattern::MemberExpression(member_expr)));
                    }
                    Expression::Identifier(ident_val) => {
                        if element.is_dots {
                            els.push(Some(
                                RestElement::new(
                                    Box::new(Pattern::Identifier(ident_val)),
                                    (element.start_loc, element.end_loc, arr_ir.source.clone()),
                                )
                                .into(),
                            ));
                        } else {
                            els.push(Some(Pattern::Identifier(ident_val)));
                        }
                    }
                    Expression::CallExpression(call_expr) => {}
                    // DestructuringAssignmentTarget[?Yield, ?Await] Initializer[+In, ?Yield, ?Await]opt
                    Expression::AssignmentExpression(AssignmentExpression {
                        left, right, ..
                    }) => {
                        if element.is_dots || outer_is_paren {
                            panic!("Invalid destructuring assignment target");
                        }
                        let ident_val_op = get_assign_left_ident(left);
                        if ident_val_op.is_some() {
                            let ident_val = ident_val_op.unwrap();
                            let ident_val_start_loc = _LocationNode {
                                pos: ident_val.start,
                                loc: ident_val.loc.start.clone(),
                            };
                            let assi_val = AssignmentPattern::new(
                                Box::new(Pattern::Identifier(ident_val)),
                                right,
                                (ident_val_start_loc, element.end_loc, arr_ir.source.clone()),
                            );
                            els.push(Some(Pattern::AssignmentPattern(assi_val)));
                        } else {
                            panic!("Invalid destructuring assignment target");
                        }
                    }
                    _ => {
                        panic!("Invalid destructuring assignment target");
                    }
                }
            }
            Element_IR_Value::Object_IR(obj_ir_val) => {
                let obj_ir_val_start_loc = obj_ir_val.start_loc.clone();
                let has_rhs_expr = element.rhs_expr.is_some();
                let obj_pat = build_obj_ir_as_pattern(obj_ir_val, strict_mode);
                let mut ele_val = if has_rhs_expr {
                    Pattern::AssignmentPattern(AssignmentPattern::new(
                        Box::new(Pattern::ObjectPattern(obj_pat)),
                        Box::new(element.rhs_expr.unwrap()),
                        (
                            obj_ir_val_start_loc,
                            element.end_loc.clone(),
                            arr_ir.source.clone(),
                        ),
                    ))
                } else {
                    Pattern::ObjectPattern(obj_pat)
                };

                if element.is_dots {
                    ele_val = Pattern::RestElement(RestElement::new(
                        Box::new(ele_val),
                        (element.start_loc, element.end_loc, arr_ir.source.clone()),
                    ));
                }
                els.push(Some(ele_val));
            }
        }
    }

    ArrayPattern::new(els, (arr_ir.start_loc, arr_ir.end_loc, arr_ir.source))
}

pub fn parse_arr_expr_or_pattern(ctx: &mut Parser) -> Expression {
    let start_loc = ctx.start_location_node();
    let arr_ir = parse_arr_ir(ctx);

    if ctx.eat(TokenLabel::Eq) {
        let arr_pat = build_array_ir_as_pattern(arr_ir, ctx.strict_mode);
        return Expression::AssignmentExpression(AssignmentExpression::new(
            AssignmentExpressionLeft::Pattern(arr_pat.into()),
            AssignmentOperator::Assignment,
            Box::new(parse_maybe_assign(ctx)),
            ctx.compose_loc_info(start_loc),
        ));
    }

    Expression::ArrayExpression(build_array_ir_as_expr(arr_ir, ctx.strict_mode))
}

pub fn parse_arr_ir(ctx: &mut Parser) -> Array_IR {
    let start_loc = ctx.start_location_node();
    let mut first = true;
    let mut arr_ir_els: Vec<Array_Element_IR> = vec![];
    let mut assert_destructuring = false;
    let mut assert_expr = false;
    let mut saw_dots = false;
    ctx.expect(TokenLabel::BracketL);
    while !ctx.eat(TokenLabel::BracketR) {
        if !first {
            ctx.expect(TokenLabel::Comma);
            if after_trailing_comma(ctx, TokenLabel::BracketR, true) {
                break;
            }
        } else {
            first = false;
        }

        if ctx.cur_token_is(TokenLabel::Comma) {
            let start_loc = ctx.start_location_node();
            ctx.next_unwrap();
            arr_ir_els.push(Array_Element_IR {
                start_loc,
                end_loc: ctx.end_location_node(),
                is_dots: false,
                value: Element_IR_Value::Elision,
                rhs_expr: None,
            });
            continue;
        }

        let element_ir = parse_arr_ir_element(ctx);
        if element_ir.is_dots {
            if saw_dots {
                assert_expr = true;
            } else {
                saw_dots = true;
            }
        }

        arr_ir_els.push(element_ir);
    }

    Array_IR {
        start_loc,
        end_loc: ctx.start_location_node(),
        source: ctx.source_file.clone(),
        elements: arr_ir_els,
        assert_destructuring,
        assert_expr,
    }
}

pub fn parse_arr_ir_element(ctx: &mut Parser) -> Array_Element_IR {
    let start_loc = ctx.start_location_node();
    let is_dots = ctx.eat(TokenLabel::Ellipsis);
    let (ele_ir_value, rhs_expr) = parse_ir_value(ctx, TokenLabel::BracketR);

    Array_Element_IR {
        start_loc,
        end_loc: ctx.end_location_node(),
        is_dots,
        value: ele_ir_value.into(),
        rhs_expr,
    }
}
