use super::{
    array::{build_array_ir_as_expr, build_array_ir_as_pattern},
    function::parse_arrow_expr,
    object::{build_obj_ir_as_expr, build_obj_ir_as_pattern, parse_ir_value, IR_Value},
    util::get_assign_left_ident,
};
use crate::{
    ast::{
        _LocationNode,
        expression::{
            AssignmentExpression, AssignmentExpressionLeft, AssignmentOperator, Expression,
            ParenthesizedExpression, SequenceExpression,
        },
        pattern::{AssignmentPattern, Pattern, RestElement},
    },
    parser::Parser,
    statement::util::{after_trailing_comma, can_insert_semicolon, unexpected},
    tokenizer::js_token::TokenLabel,
};

#[allow(non_camel_case_types)]
pub struct Paren_IR_List {
    start_loc: _LocationNode,
    end_loc: _LocationNode,
    inner_start_loc: _LocationNode,
    inner_end_loc: _LocationNode,
    source: Option<String>,
    elements: Vec<Paren_IR_Element>,
    assert_expr: bool,
    assert_binding: bool,
    last_is_comma: bool,
}

#[allow(non_camel_case_types)]
pub struct Paren_IR_Element {
    start_loc: _LocationNode,
    end_loc: _LocationNode,
    is_dots: bool,
    value: IR_Value,
    rhs_expr: Option<Expression>,
}

pub fn build_paren_ir_as_pattern(ir_list: Paren_IR_List, strict_mode: bool) -> Vec<Pattern> {
    if !ir_list.assert_expr {
        panic!("Invalid destructuring assignment target");
    }

    let mut results = vec![];
    for element in ir_list.elements {
        match element.value {
            IR_Value::Array_IR(arr_ir_val) => {
                let arr_ir_start_loc = arr_ir_val.start_loc.clone();
                let arr_pat = build_array_ir_as_pattern(arr_ir_val, strict_mode);
                let mut el_val = if element.rhs_expr.is_some() {
                    Pattern::AssignmentPattern(AssignmentPattern::new(
                        Box::new(Pattern::ArrayPattern(arr_pat)),
                        Box::new(element.rhs_expr.unwrap()),
                        (
                            arr_ir_start_loc,
                            element.end_loc.clone(),
                            ir_list.source.clone(),
                        ),
                    ))
                } else {
                    Pattern::ArrayPattern(arr_pat)
                };
                if element.is_dots {
                    el_val = Pattern::RestElement(RestElement::new(
                        Box::new(el_val),
                        (element.start_loc, element.end_loc, ir_list.source.clone()),
                    ))
                }
                results.push(el_val);
            }
            IR_Value::Object_IR(obj_ir_val) => {
                let obj_ir_start_loc = obj_ir_val.start_loc.clone();
                let has_rhs_expr = element.rhs_expr.is_some();
                let obj_pat = build_obj_ir_as_pattern(obj_ir_val, strict_mode);
                let mut el_val = if has_rhs_expr {
                    Pattern::AssignmentPattern(AssignmentPattern::new(
                        Box::new(Pattern::ObjectPattern(obj_pat)),
                        Box::new(element.rhs_expr.unwrap()),
                        (
                            obj_ir_start_loc,
                            element.end_loc.clone(),
                            ir_list.source.clone(),
                        ),
                    ))
                } else {
                    Pattern::ObjectPattern(obj_pat)
                };
                if element.is_dots {
                    el_val = Pattern::RestElement(RestElement::new(
                        Box::new(el_val),
                        (element.start_loc, element.end_loc, ir_list.source.clone()),
                    ));
                }
                results.push(el_val);
            }
            IR_Value::Expression(expr_val) => match expr_val {
                Expression::Identifier(ident_val) => {
                    results.push(Pattern::Identifier(ident_val));
                }
                Expression::AssignmentExpression(AssignmentExpression { left, right, .. }) => {
                    let left_val = get_assign_left_ident(left);
                    if left_val.is_some() {
                        results.push(Pattern::AssignmentPattern(AssignmentPattern::new(
                            Box::new(Pattern::Identifier(left_val.unwrap())),
                            right,
                            (
                                element.start_loc.clone(),
                                element.end_loc.clone(),
                                ir_list.source.clone(),
                            ),
                        )));
                    } else {
                        panic!("Invalid destructuring assignment target")
                    }
                }
                _ => {
                    panic!("Invalid destructuring assignment target")
                }
            },
        }
    }

    results
}

pub fn build_paren_ir_as_expr(
    ir_list: Paren_IR_List,
    strict_mode: bool,
    keep_paren: bool,
) -> Expression {
    if ir_list.assert_binding {
        panic!("");
    }

    let mut results = vec![];
    for element in ir_list.elements {
        match element.value {
            IR_Value::Array_IR(arr_ir_val) => {
                let arr_ir_start_loc = arr_ir_val.start_loc.clone();
                let arr_val = build_array_ir_as_expr(arr_ir_val, strict_mode);
                let el_val = if element.rhs_expr.is_some() {
                    Expression::AssignmentExpression(AssignmentExpression::new(
                        AssignmentExpressionLeft::Expression(Box::new(
                            Expression::ArrayExpression(arr_val),
                        )),
                        AssignmentOperator::Assignment,
                        Box::new(element.rhs_expr.unwrap()),
                        (arr_ir_start_loc, element.end_loc, ir_list.source.clone()),
                    ))
                } else {
                    Expression::ArrayExpression(arr_val)
                };
                results.push(el_val);
            }
            IR_Value::Object_IR(obj_ir_val) => {
                let obj_ir_start_loc = obj_ir_val.start_loc.clone();
                let obj_val = build_obj_ir_as_expr(obj_ir_val, strict_mode);
                results.push(if element.rhs_expr.is_some() {
                    Expression::AssignmentExpression(AssignmentExpression::new(
                        AssignmentExpressionLeft::Expression(Box::new(
                            Expression::ObjectExpression(obj_val),
                        )),
                        AssignmentOperator::Assignment,
                        Box::new(element.rhs_expr.unwrap()),
                        (obj_ir_start_loc, element.end_loc, ir_list.source.clone()),
                    ))
                } else {
                    Expression::ObjectExpression(obj_val)
                });
            }
            IR_Value::Expression(expr_val) => {
                results.push(expr_val);
            }
        }
    }

    let expr = if results.len() > 1 {
        SequenceExpression::new(
            results,
            (
                ir_list.inner_start_loc,
                ir_list.inner_end_loc,
                ir_list.source.clone(),
            ),
        )
        .into()
    } else {
        results.remove(0)
    };

    if keep_paren {
        Expression::ParenthesizedExpression(ParenthesizedExpression::new(
            Box::new(expr),
            (ir_list.start_loc, ir_list.end_loc, ir_list.source.clone()),
        ))
    } else {
        expr
    }
}

// parse irs surrounded by pair parens
fn parse_paren_ir_list(ctx: &mut Parser) -> Paren_IR_List {
    let start_loc = ctx.start_location_node();
    ctx.expect(TokenLabel::ParenL);
    let inner_start_loc = ctx.start_location_node();

    let mut assert_binding = false;
    let assert_expr = false;
    let mut first = true;
    let mut last_is_comma = false;
    let mut elements: Vec<Paren_IR_Element> = vec![];

    while !ctx.cur_token_is(TokenLabel::ParenR) {
        if first {
            first = false;
        } else {
            ctx.expect(TokenLabel::Comma);
        }

        if after_trailing_comma(ctx, TokenLabel::ParenR, false) {
            last_is_comma = true;
            break;
        }

        if ctx.cur_token_is(TokenLabel::Ellipsis) {
            let rest_start_loc = ctx.start_location_node();
            ctx.next_unwrap();
            let (ir_val, rhs_expr) = parse_ir_value(ctx, TokenLabel::ParenR);
            if ctx.cur_token_is(TokenLabel::Comma) {
                panic!("Comma is not permitted after the rest element");
            }
            if assert_binding {
                panic!("Rest parameter must be last formal parameter");
            } else {
                assert_binding = true;
            }
            if rhs_expr.is_some() {
                panic!("Rest parameter may not have a default initializer");
            }
            elements.push(Paren_IR_Element {
                start_loc: rest_start_loc,
                end_loc: ctx.end_location_node(),
                is_dots: true,
                value: ir_val,
                rhs_expr: None,
            });
            break;
        }

        let ir_start_loc = ctx.start_location_node();
        let (ir_val, rhs_expr) = parse_ir_value(ctx, TokenLabel::ParenR);
        elements.push(Paren_IR_Element {
            start_loc: ir_start_loc,
            end_loc: ctx.end_location_node(),
            is_dots: false,
            value: ir_val,
            rhs_expr,
        });
    }

    if assert_expr && assert_binding {
        panic!("Invalid destructuring assignment target");
    }

    let inner_end_loc = ctx.end_location_node();
    ctx.expect(TokenLabel::ParenR);

    Paren_IR_List {
        start_loc,
        end_loc: ctx.end_location_node(),
        inner_start_loc,
        inner_end_loc,
        assert_expr,
        assert_binding,
        elements,
        source: ctx.source_file.clone(),
        last_is_comma,
    }
}

// When pasring a single parenl in expression, the case is a little complex,
// because it could be a paren expression or the start of an arrow function, or an assignment pattern ect.
pub fn parse_parenl(ctx: &mut Parser) -> Expression {
    let may_be_arrow = ctx.potential_arrow_pos == ctx.cur_token_start;
    let start_loc = ctx.start_location_node();
    let old_yield_pos = ctx.yield_pos;
    let old_await_pos = ctx.await_pos;

    let paren_ir_list = parse_paren_ir_list(ctx);

    if may_be_arrow && !can_insert_semicolon(ctx) && ctx.eat(TokenLabel::Arrow) {
        // TODO: ignore error check for now
        ctx.yield_pos = old_yield_pos;
        ctx.await_pos = old_await_pos;
        return parse_arrow_expr(
            ctx,
            start_loc,
            build_paren_ir_as_pattern(paren_ir_list, ctx.strict_mode),
            false,
        )
        .into();
    }

    if paren_ir_list.assert_binding {
        unexpected(ctx.cur_token.clone().unwrap());
    }

    // parenthsized expression must contain at lease one element,
    //  and the last one can not be comma.
    if paren_ir_list.elements.len() == 0 || paren_ir_list.last_is_comma {
        unexpected(ctx.cur_token.clone().unwrap());
    }

    // TODO: check expression error
    if old_yield_pos > 0 {
        ctx.yield_pos = old_yield_pos;
    }
    if old_await_pos > 0 {
        ctx.await_pos = old_await_pos;
    }

    // TODO: add option for keep_paren
    build_paren_ir_as_expr(paren_ir_list, ctx.strict_mode, false)
}
