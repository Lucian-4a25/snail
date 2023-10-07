use serde::Serialize;

use super::{
    expression::{Literal, LiteralValue},
    NodeType, ProgramNode, SourceLocation,
};

#[derive(Serialize)]
pub struct Directive {
    #[serde(rename(serialize = "type"))]
    pub _type: NodeType, // ExpressionStatement
    pub start: usize,
    pub end: usize,
    pub loc: SourceLocation,
    pub expression: Literal,
    pub directive: String, // raw string exclude the quotes
}

impl From<Literal> for Directive {
    fn from(value: Literal) -> Self {
        let expression = value.clone();
        let directive = if let LiteralValue::String(v) = value.value {
            v
        } else {
            panic!("Directive can only be String Literal")
        };
        Self {
            _type: NodeType::ExpressionStatement,
            expression,
            directive,
            start: value.start,
            end: value.end,
            loc: value.loc,
        }
    }
}

impl From<Directive> for ProgramNode {
    fn from(value: Directive) -> Self {
        Self::Directive(value)
    }
}
