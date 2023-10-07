use super::{
    array::{
        build_array_ir_as_expr, build_array_ir_as_pattern, parse_arr_ir, Array_IR, Element_IR_Value,
    },
    assignment::parse_maybe_assign,
    binary::{parse_expr_op, ExpressionOperatorLeft},
    function::parse_function_body,
    literal::parse_literal,
    parse_ident,
    subscript::parse_subscripts,
    util::{get_assign_left_ident, get_paren_expr_val},
};
use crate::{
    ast::{
        _LocationNode,
        expression::{
            AssignmentExpression, AssignmentExpressionLeft, AssignmentOperator,
            ConditionalExpression, Expression, FunctionExpression, ObjectExpression,
            ObjectProperty, Property, PropertyKind, SpreadElement,
        },
        pattern::{
            AssignmentPattern, AssignmentProperty, ObjectPattern, ObjectPatternProperty, Pattern,
            RestElement,
        },
    },
    parser::Parser,
    statement::{
        lval::parse_binding_list,
        scope::{get_func_flags, SCOPE_DIRECT_SUPER},
        util::{after_trailing_comma, unexpected},
    },
    tokenizer::{js_token::TokenLabel, util::has_break_in_range},
};
use std::vec;

// represents object expression or object pattern.
#[allow(non_camel_case_types)]
pub struct Object_IR {
    pub start_loc: _LocationNode,
    pub end_loc: _LocationNode,
    pub source: Option<String>,
    pub properties: Vec<Object_Property_IR>,
    /// indicate it must be a object pattern.
    pub assert_destructuring: bool,
    /// indicate it must be a object expression.
    pub assert_expr: bool,
}

#[allow(non_camel_case_types)]
pub struct Object_Property_IR {
    start_loc: _LocationNode,
    end_loc: _LocationNode,
    // when is_dots is true, the value of key is None.
    key: Option<Expression>,
    value: Property_IR_Value,
    /// The type of rhs_expr is Some when the operator between value and rhs_expr is equal,
    /// and the value starts with another object_ir or array_ir,
    /// such as:
    /// - { a: { b } = [rhs_expr] }
    /// - { a: [c] = [rhs_expr] }
    /// When rhs_expr is some, the type of object_ir_value could be assignment expression or assignment pattern.
    rhs_expr: Option<Expression>,
    kind: PropertyKind,
    is_method: bool,
    is_async: bool,
    is_generator: bool,
    is_shorthand: bool,
    computed: bool,
    // '...'
    is_dots: bool,
}

#[allow(non_camel_case_types)]
pub enum Property_IR_Value {
    Object_IR(Object_IR), // for case like: { a: { b, c.. }, }
    Array_IR(Array_IR),
    Expression(Expression), // for case like method、expression except object_ir or array_ir
}

pub fn build_obj_ir_as_expr(obj_ir: Object_IR, strict_mode: bool) -> ObjectExpression {
    if obj_ir.assert_destructuring {
        panic!("Invalid shorthand property initializer");
    }

    // build a object expression
    let mut obj_expr_props: Vec<ObjectProperty> = vec![];
    for prop in obj_ir.properties {
        match prop.value {
            Property_IR_Value::Array_IR(arr_ir_val) => {
                let has_rhs_expr = prop.rhs_expr.is_some();
                let arr_val_start_loc = arr_ir_val.start_loc.clone();
                let prop_val = if has_rhs_expr {
                    Expression::AssignmentExpression(AssignmentExpression::new(
                        AssignmentExpressionLeft::Pattern(Pattern::ArrayPattern(
                            build_array_ir_as_pattern(arr_ir_val, strict_mode),
                        )),
                        AssignmentOperator::Assignment,
                        prop.rhs_expr.unwrap().into(),
                        (
                            arr_val_start_loc,
                            prop.end_loc.clone(),
                            obj_ir.source.clone(),
                        ),
                    ))
                } else {
                    Expression::ArrayExpression(build_array_ir_as_expr(arr_ir_val, strict_mode))
                };
                obj_expr_props.push(if prop.is_dots {
                    ObjectProperty::SpreadElement(SpreadElement::new(
                        prop_val,
                        (prop.start_loc, prop.end_loc, obj_ir.source.clone()),
                    ))
                } else {
                    ObjectProperty::Property(Property::new(
                        prop.key.unwrap(),
                        prop_val,
                        PropertyKind::Init,
                        prop.is_method,
                        prop.is_shorthand,
                        prop.computed,
                        (prop.start_loc, prop.end_loc, obj_ir.source.clone()),
                    ))
                });
            }
            Property_IR_Value::Object_IR(obj_ir_val) => {
                let has_rhs_expr = prop.rhs_expr.is_some();
                let obj_ir_val_start_loc = obj_ir_val.start_loc.clone();
                let prop_val = if has_rhs_expr {
                    Expression::AssignmentExpression(AssignmentExpression::new(
                        AssignmentExpressionLeft::Pattern(Pattern::ObjectPattern(
                            build_obj_ir_as_pattern(obj_ir_val, strict_mode),
                        )),
                        AssignmentOperator::Assignment,
                        prop.rhs_expr.unwrap().into(),
                        (
                            obj_ir_val_start_loc,
                            prop.end_loc.clone(),
                            obj_ir.source.clone(),
                        ),
                    ))
                } else {
                    Expression::ObjectExpression(build_obj_ir_as_expr(obj_ir_val, strict_mode))
                };
                obj_expr_props.push(if prop.is_dots {
                    ObjectProperty::SpreadElement(SpreadElement::new(
                        prop_val,
                        (prop.start_loc, prop.end_loc, obj_ir.source.clone()),
                    ))
                } else {
                    ObjectProperty::Property(Property::new(
                        prop.key.unwrap(),
                        prop_val,
                        PropertyKind::Init,
                        prop.is_method,
                        prop.is_shorthand,
                        prop.computed,
                        (prop.start_loc, prop.end_loc, obj_ir.source.clone()),
                    ))
                });
            }
            Property_IR_Value::Expression(expr_val) => {
                obj_expr_props.push(if prop.is_dots {
                    ObjectProperty::SpreadElement(SpreadElement::new(
                        expr_val,
                        (prop.start_loc, prop.end_loc, obj_ir.source.clone()),
                    ))
                } else {
                    ObjectProperty::Property(Property::new(
                        prop.key.unwrap(),
                        expr_val,
                        prop.kind,
                        prop.is_method,
                        prop.is_shorthand,
                        prop.computed,
                        (prop.start_loc, prop.end_loc, obj_ir.source.clone()),
                    ))
                });
            }
        }
    }

    ObjectExpression::new(
        obj_expr_props,
        (obj_ir.start_loc, obj_ir.end_loc, obj_ir.source.clone()),
    )
}

pub fn build_obj_ir_as_pattern(obj_ir: Object_IR, strict_mode: bool) -> ObjectPattern {
    if obj_ir.assert_expr {
        panic!("Invalid destructuring assignment target")
    }

    let mut obj_pattern_props: Vec<ObjectPatternProperty> = vec![];

    for prop in obj_ir.properties {
        if prop.is_dots {
            // check if the assignment target type of destructuring is simple.
            // See: https://tc39.es/ecma262/#sec-static-semantics-assignmenttargettype
            // For AssignmentRestElement, the value must be a simple AssignmentTargetType.
            match prop.value {
                Property_IR_Value::Expression(expr) => {
                    let prop_val = match get_paren_expr_val(expr) {
                        Expression::Identifier(ident_val) => {
                            if strict_mode
                                && (ident_val.name == "eval" || ident_val.name == "arguments")
                            {
                                panic!("Invalid destructuring assignment target");
                            }
                            Box::new(Pattern::Identifier(ident_val))
                        }
                        Expression::MemberExpression(mem_expr) => {
                            Box::new(Pattern::MemberExpression(mem_expr))
                        }
                        _ => {
                            panic!("Invalid destructuring assignment target");
                        }
                    };
                    obj_pattern_props.push(ObjectPatternProperty::RestElement(RestElement::new(
                        prop_val,
                        (prop.start_loc, prop.end_loc, obj_ir.source.clone()),
                    )));
                }
                _ => panic!(
                    "`...` must be followed by an assignable reference in assignment contexts"
                ),
            }
            continue;
        }

        let prop_key = prop.key.unwrap();
        match prop.value {
            Property_IR_Value::Object_IR(obj_ir_val) => {
                let has_rhs_expr = prop.rhs_expr.is_some();
                let pat_start_loc = obj_ir_val.start_loc.clone();
                let obj_pat = build_obj_ir_as_pattern(obj_ir_val, strict_mode);
                let assign_prop_val = if has_rhs_expr {
                    Pattern::AssignmentPattern(AssignmentPattern::new(
                        Box::new(Pattern::ObjectPattern(obj_pat)),
                        prop.rhs_expr.unwrap().into(),
                        (pat_start_loc, prop.end_loc.clone(), obj_ir.source.clone()),
                    ))
                } else {
                    Pattern::ObjectPattern(obj_pat)
                };
                let assign_prop = AssignmentProperty::new(
                    prop_key,
                    assign_prop_val,
                    false,
                    prop.computed,
                    (prop.start_loc, prop.end_loc, obj_ir.source.clone()),
                );
                obj_pattern_props.push(ObjectPatternProperty::AssignmentProperty(assign_prop));
            }
            Property_IR_Value::Array_IR(arr_ir) => {
                let has_rhs_expr = prop.rhs_expr.is_some();
                let pat_start_loc = arr_ir.start_loc.clone();
                let pat_value = build_array_ir_as_pattern(arr_ir, strict_mode);
                let assign_prop_val = if has_rhs_expr {
                    Pattern::AssignmentPattern(AssignmentPattern::new(
                        Box::new(Pattern::ArrayPattern(pat_value)),
                        prop.rhs_expr.unwrap().into(),
                        (pat_start_loc, prop.end_loc.clone(), obj_ir.source.clone()),
                    ))
                } else {
                    Pattern::ArrayPattern(pat_value)
                };
                let assign_prop = AssignmentProperty::new(
                    prop_key,
                    assign_prop_val,
                    false,
                    prop.computed,
                    (prop.start_loc, prop.end_loc, obj_ir.source.clone()),
                );
                obj_pattern_props.push(ObjectPatternProperty::AssignmentProperty(assign_prop));
            }
            Property_IR_Value::Expression(expr_val) => {
                let outer_is_paren = matches!(&expr_val, Expression::ParenthesizedExpression(..));
                let expr_v = get_paren_expr_val(expr_val);
                match expr_v {
                    Expression::Identifier(ident_val) => {
                        let assign_prop = AssignmentProperty::new(
                            prop_key,
                            Pattern::Identifier(ident_val),
                            prop.is_shorthand,
                            prop.computed,
                            (prop.start_loc, prop.end_loc, obj_ir.source.clone()),
                        );
                        obj_pattern_props
                            .push(ObjectPatternProperty::AssignmentProperty(assign_prop));
                    }
                    Expression::MemberExpression(member_expr) => {
                        let assign_prop = AssignmentProperty::new(
                            prop_key,
                            Pattern::MemberExpression(member_expr),
                            false,
                            prop.computed,
                            (prop.start_loc, prop.end_loc, obj_ir.source.clone()),
                        );
                        obj_pattern_props
                            .push(ObjectPatternProperty::AssignmentProperty(assign_prop));
                    }
                    Expression::AssignmentExpression(AssignmentExpression {
                        left, right, ..
                    }) => {
                        if outer_is_paren {
                            panic!("Invalid destructuring assignment target");
                        }
                        let left_ident_op = get_assign_left_ident(left);
                        if left_ident_op.is_some() {
                            let left_ident = left_ident_op.unwrap();
                            let left_ident_start_loc = _LocationNode {
                                pos: left_ident.start,
                                loc: left_ident.loc.start.clone(),
                            };
                            let assign_val = AssignmentPattern::new(
                                Box::new(left_ident.into()),
                                right,
                                (
                                    left_ident_start_loc,
                                    prop.end_loc.clone(),
                                    obj_ir.source.clone(),
                                ),
                            );
                            let assign_prop = AssignmentProperty::new(
                                prop_key,
                                Pattern::AssignmentPattern(assign_val),
                                prop.is_shorthand,
                                prop.computed,
                                (prop.start_loc, prop.end_loc, obj_ir.source.clone()),
                            );
                            obj_pattern_props
                                .push(ObjectPatternProperty::AssignmentProperty(assign_prop));
                        } else {
                            panic!("Invalid destructuring assignment target");
                        }
                    }
                    _ => {
                        panic!("Invalid destructuring assignment target");
                    }
                }
            }
        }
    }

    ObjectPattern::new(
        obj_pattern_props,
        (obj_ir.start_loc, obj_ir.end_loc, obj_ir.source),
    )
}

// ({} = ..)
// let a = {};
pub fn parse_obj_expr_or_pattern(ctx: &mut Parser) -> Expression {
    let start_loc = ctx.start_location_node();
    let object_ir = parse_obj_ir(ctx);

    if ctx.eat(TokenLabel::Eq) {
        let left = build_obj_ir_as_pattern(object_ir, ctx.strict_mode);
        return Expression::AssignmentExpression(AssignmentExpression::new(
            AssignmentExpressionLeft::Pattern(Pattern::ObjectPattern(left)),
            AssignmentOperator::Assignment,
            Box::new(parse_maybe_assign(ctx)),
            ctx.compose_loc_info(start_loc),
        ));
    }

    Expression::ObjectExpression(build_obj_ir_as_expr(object_ir, ctx.strict_mode))
}

pub fn parse_obj_ir(ctx: &mut Parser) -> Object_IR {
    let start_loc = ctx.start_location_node();
    let mut first = true;
    let mut properties = vec![];

    ctx.expect(TokenLabel::BraceL);
    let mut assert_destructuring = false;
    let mut assert_expr = false;
    let mut saw_dots = false;
    while !ctx.eat(TokenLabel::BraceR) {
        if !first {
            ctx.expect(TokenLabel::Comma);
            if after_trailing_comma(ctx, TokenLabel::BraceR, true) {
                break;
            }
        } else {
            first = false;
        }

        // TODO: check properties clash error
        let property_ir = parse_property_ir(ctx);
        if property_ir.is_dots {
            if !saw_dots {
                saw_dots = true;
            }
            // only expression could contain multiple '...'
            else {
                assert_expr = true;
            }
        }
        if property_ir.is_method {
            assert_expr = true;
        }
        // check format like: { a = 1, }
        if property_ir.is_shorthand
            && matches!(
                property_ir.value,
                Property_IR_Value::Expression(Expression::AssignmentExpression(..))
            )
        {
            assert_destructuring = true;
        }
        properties.push(property_ir);
    }
    if assert_expr && assert_destructuring {
        panic!("Invalid destructuring assignment target");
    }

    Object_IR {
        start_loc,
        end_loc: ctx.start_location_node(),
        source: ctx.source_file.clone(),
        properties,
        assert_destructuring,
        assert_expr,
    }
}

pub fn parse_expr_starts_with_atom(
    ctx: &mut Parser,
    expr: Expression,
    start_loc: _LocationNode,
) -> Expression {
    let expr_with_scripts = parse_subscripts(ctx, expr, start_loc.clone());
    let expr_with_postfix = parse_expr_op(
        ctx,
        ExpressionOperatorLeft::Expression(expr_with_scripts),
        start_loc.clone(),
        -1,
    );
    let expr_with_conditional = if ctx.cur_token_is(TokenLabel::Question) {
        let consequent = parse_maybe_assign(ctx);
        ctx.expect(TokenLabel::Colon);
        let alternate = parse_maybe_assign(ctx);
        ConditionalExpression::new(
            Box::new(expr_with_postfix),
            Box::new(consequent),
            Box::new(alternate),
            ctx.compose_loc_info(start_loc.clone()),
        )
        .into()
    } else {
        expr_with_postfix
    };

    let mut maybe_assignment_expr = expr_with_conditional;
    while ctx.cur_token_test(|t| t.is_assign) {
        maybe_assignment_expr = Expression::AssignmentExpression(AssignmentExpression::new(
            AssignmentExpressionLeft::Expression(Box::new(maybe_assignment_expr)),
            AssignmentOperator::Assignment,
            Box::new(parse_maybe_assign(ctx)),
            ctx.compose_loc_info(start_loc.clone()),
        ));
    }

    maybe_assignment_expr
}

#[allow(non_camel_case_types)]
pub enum IR_Value {
    Array_IR(Array_IR),
    Object_IR(Object_IR),
    Expression(Expression),
}

impl From<IR_Value> for Property_IR_Value {
    fn from(value: IR_Value) -> Self {
        match value {
            IR_Value::Array_IR(arr_ir) => Property_IR_Value::Array_IR(arr_ir),
            IR_Value::Object_IR(obj_ir) => Property_IR_Value::Object_IR(obj_ir),
            IR_Value::Expression(expr) => Property_IR_Value::Expression(expr),
        }
    }
}

impl From<IR_Value> for Element_IR_Value {
    fn from(value: IR_Value) -> Self {
        match value {
            IR_Value::Array_IR(arr_ir) => Self::Array_IR(arr_ir),
            IR_Value::Object_IR(obj_ir) => Self::Object_IR(obj_ir),
            IR_Value::Expression(expr) => Self::Expression(expr),
        }
    }
}

pub fn parse_property_ir(ctx: &mut Parser) -> Object_Property_IR {
    let start_loc = ctx.start_location_node();

    if ctx.eat(TokenLabel::Ellipsis) {
        let (ir_val, rhs_expr) = parse_ir_value(ctx, TokenLabel::BraceR);

        return Object_Property_IR {
            start_loc,
            end_loc: ctx.start_location_node(),
            key: None,
            value: ir_val.into(),
            rhs_expr,
            kind: PropertyKind::Init,
            is_method: false,
            is_async: false,
            is_generator: false,
            is_shorthand: false,
            computed: false,
            is_dots: true,
        };
    }

    let mut kind = PropertyKind::Init;
    let mut is_async = false;
    let mut property_key: Option<Expression> = None;

    // maybe async function, such as: let obj = { async *[fn]() {} };
    if ctx.is_contextual("async") {
        property_key = Some(parse_ident(ctx, true).into());
        if !has_break_in_range(ctx, (ctx.last_token_end, ctx.cur_token_start))
            && ctx.cur_token_test(|t| {
                let label = t.label;
                return label == TokenLabel::Name
                    || label == TokenLabel::Number
                    || label == TokenLabel::BracketL
                    || label == TokenLabel::String
                    || label == TokenLabel::Star
                    || t.keyword;
            })
        {
            is_async = true;
            property_key = None;
        }
    }

    // maybe generator function
    let is_generator = if kind == PropertyKind::Init {
        ctx.eat(TokenLabel::Star)
    } else {
        false
    };

    // maybe a getter or setter function, such as: let obj = { get method() {}, set setVal() }
    if !is_generator && !is_async && (ctx.is_contextual("get") || ctx.is_contextual("set")) {
        property_key = Some(parse_ident(ctx, true).into());
        if ctx.cur_token_test(|t| {
            let label = t.label;
            return label == TokenLabel::Name
                || label == TokenLabel::Number
                || label == TokenLabel::BracketL
                || label == TokenLabel::String
                || t.keyword;
        }) {
            kind = if ctx.cur_token_value_is("get") {
                PropertyKind::Get
            } else {
                PropertyKind::Set
            };
            property_key = None;
        }
    }

    let mut computed = false;

    // To parse property key here if need.
    if property_key.is_none() {
        let key;
        computed = ctx.eat(TokenLabel::BracketL);
        if computed {
            key = parse_maybe_assign(ctx);
            ctx.expect(TokenLabel::BracketR);
        } else if ctx
            .cur_token_test(|t| t.label == TokenLabel::String || t.label == TokenLabel::Number)
        {
            key = parse_literal(ctx).into();
        } else {
            key = parse_ident(ctx, true).into();
        }
        property_key = Some(key);
    }

    let is_shorthand = !computed
        && (ctx.cur_token_is(TokenLabel::Comma)
            || ctx.cur_token_is(TokenLabel::Eq)
            || ctx.cur_token_is(TokenLabel::BraceR));
    let key = property_key.unwrap();

    // parse method first
    if ctx.cur_token_is(TokenLabel::ParenL) {
        let func_expr = parse_method(ctx, is_generator, is_async, false);
        match kind {
            PropertyKind::Get => {
                if func_expr.params.len() != 0 {
                    println!("getter should have no params");
                }
            }
            PropertyKind::Set => {
                if func_expr.params.len() != 1 {
                    println!("setter should have exactly one param");
                }
                if func_expr
                    .params
                    .first()
                    .map_or(false, |p| matches!(p, Pattern::RestElement(..)))
                {
                    println!("Setter cannot use rest params")
                }
            }
            _ => {}
        }

        return Object_Property_IR {
            start_loc,
            end_loc: ctx.start_location_node(),
            key: Some(key),
            value: Property_IR_Value::Expression(Expression::FunctionExpression(func_expr)),
            rhs_expr: None,
            kind: kind.clone(),
            is_method: true,
            is_async,
            is_generator,
            is_shorthand,
            computed,
            is_dots: false,
        };
    }

    if is_async || is_generator || kind != PropertyKind::Init {
        unexpected(ctx.cur_token.clone().unwrap());
    }

    if ctx.eat(TokenLabel::Colon) {
        let (ir_val, rhs_expr) = parse_ir_value(ctx, TokenLabel::BraceR);

        return Object_Property_IR {
            start_loc,
            end_loc: ctx.start_location_node(),
            key: None,
            value: ir_val.into(),
            rhs_expr,
            kind: PropertyKind::Init,
            is_method: false,
            is_async: false,
            is_generator: false,
            is_shorthand: false,
            computed: false,
            is_dots: false,
        };
    }

    if is_shorthand {
        if let Expression::Identifier(ident_key) = &key {
            let ir_value = if ctx.eat(TokenLabel::Eq) {
                let right_val = parse_maybe_assign(ctx);
                Expression::AssignmentExpression(AssignmentExpression::new(
                    AssignmentExpressionLeft::Expression(Box::new(ident_key.clone().into())),
                    AssignmentOperator::Assignment,
                    right_val.into(),
                    ctx.compose_loc_info(start_loc.clone()),
                ))
            } else {
                ident_key.clone().into()
            };

            return Object_Property_IR {
                start_loc,
                end_loc: ctx.start_location_node(),
                key: Some(key),
                value: Property_IR_Value::Expression(ir_value),
                rhs_expr: None,
                kind: PropertyKind::Init,
                is_method: false,
                is_async: false,
                is_generator: false,
                is_shorthand: true,
                computed: false,
                is_dots: false,
            };
        }
    }

    unexpected(ctx.cur_token.clone().unwrap());
}

pub fn parse_ir_value(ctx: &mut Parser, close_label: TokenLabel) -> (IR_Value, Option<Expression>) {
    if ctx.cur_token_is(TokenLabel::BraceL) {
        let obj_ir_value = parse_obj_ir(ctx);
        if ctx.eat(TokenLabel::Eq) {
            return (
                IR_Value::Object_IR(obj_ir_value),
                Some(parse_maybe_assign(ctx)),
            );
        }

        if ctx.cur_token_test(|t| t.label == TokenLabel::Comma || t.label == close_label) {
            return (IR_Value::Object_IR(obj_ir_value), None);
        }

        // try to parse ir as expression:
        // build obj ir as expression, then parse an expression with it.
        let obj_ir_start_loc = obj_ir_value.start_loc.clone();
        let expr_val = parse_expr_starts_with_atom(
            ctx,
            Expression::ObjectExpression(build_obj_ir_as_expr(obj_ir_value, ctx.strict_mode)),
            obj_ir_start_loc,
        );

        return (IR_Value::Expression(expr_val), None);
    }

    if ctx.cur_token_is(TokenLabel::BracketL) {
        let arr_ir_val = parse_arr_ir(ctx);
        if ctx.eat(TokenLabel::Eq) {
            return (
                IR_Value::Array_IR(arr_ir_val),
                Some(parse_maybe_assign(ctx)),
            );
        }

        if ctx.cur_token_test(|t| t.label == TokenLabel::Comma || t.label == close_label) {
            return (IR_Value::Array_IR(arr_ir_val), None);
        }

        let arr_ir_start_loc = arr_ir_val.start_loc.clone();
        let expr_val = parse_expr_starts_with_atom(
            ctx,
            Expression::ArrayExpression(build_array_ir_as_expr(arr_ir_val, ctx.strict_mode)),
            arr_ir_start_loc,
        );

        return (IR_Value::Expression(expr_val), None);
    }

    return (IR_Value::Expression(parse_maybe_assign(ctx)), None);
}

pub fn parse_method(
    ctx: &mut Parser,
    is_generator: bool,
    is_async: bool,
    allow_super: bool,
) -> FunctionExpression {
    let start_loc = ctx.start_location_node();
    let flags = get_func_flags(is_async, is_generator);
    // TODO: check the usage of SCOPE_SUPER as default here
    ctx.enter_scope(flags | if allow_super { SCOPE_DIRECT_SUPER } else { 0 });

    ctx.expect(TokenLabel::ParenL);
    let params = parse_binding_list(ctx, TokenLabel::ParenR, false, true);

    // TODO: check param error.
    let function_body = parse_function_body(ctx);
    ctx.exit_scope();

    FunctionExpression::new(
        None,
        params.into_iter().map(|p| p.unwrap()).collect(),
        function_body,
        is_generator,
        is_async,
        ctx.compose_loc_info(start_loc),
    )
}
