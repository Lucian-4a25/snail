use crate::ast::{
    expression::{AssignmentExpressionLeft, Expression, Identifier, ParenthesizedExpression},
    pattern::Pattern,
};

pub fn is_lhs_expr_simple(mut expr: &Expression, strict: bool) -> bool {
    while matches!(expr, Expression::ParenthesizedExpression(..)) {
        if let Expression::ParenthesizedExpression(ParenthesizedExpression {
            expression: val,
            ..
        }) = expr
        {
            expr = &*val;
        }
    }
    match expr {
        Expression::Identifier(ident_val) => {
            if strict && (ident_val.name == "eval" || ident_val.name == "arguments") {
                return false;
            }
            return true;
        }
        Expression::MemberExpression(..) => {
            return true;
        }
        _ => {
            return false;
        }
    }
}

pub fn get_paren_expr_val(expr: Expression) -> Expression {
    let mut val = expr;
    loop {
        if let Expression::ParenthesizedExpression(ParenthesizedExpression { expression, .. }) = val
        {
            val = *expression;
        } else {
            break;
        }
    }

    val
}

pub fn get_assign_left_ident(assign: AssignmentExpressionLeft) -> Option<Identifier> {
    match assign {
        AssignmentExpressionLeft::Expression(expr) => {
            if let Expression::Identifier(ident) = *expr {
                return Some(ident);
            }
        }
        AssignmentExpressionLeft::Pattern(Pattern::Identifier(ident)) => {
            return Some(ident);
        }
        _ => {}
    }

    None
}
