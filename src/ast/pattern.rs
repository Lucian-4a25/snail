use serde::Serialize;

use super::{
    expression::{Expression, Identifier, MemberExpression},
    AstNodePos, NodeType, SourceLocation,
};

#[derive(Serialize)]
pub enum Pattern {
    RestElement(RestElement),
    ArrayPattern(ArrayPattern),
    ObjectPattern(ObjectPattern),
    Identifier(Identifier),
    AssignmentPattern(AssignmentPattern),
    MemberExpression(MemberExpression),
}

impl From<Identifier> for Pattern {
    fn from(value: Identifier) -> Self {
        Self::Identifier(value)
    }
}

impl From<RestElement> for Pattern {
    fn from(value: RestElement) -> Self {
        Self::RestElement(value)
    }
}

impl From<ArrayPattern> for Pattern {
    fn from(value: ArrayPattern) -> Self {
        Self::ArrayPattern(value)
    }
}

#[derive(Serialize)]
pub struct RestElement {
    #[serde(rename(serialize = "type"))]
    pub _type: NodeType,
    pub start: usize,
    pub end: usize,
    pub loc: SourceLocation,
    pub argument: Box<Pattern>,
}

impl RestElement {
    pub fn new(arg: Box<Pattern>, (start_loc, end_loc, source): AstNodePos) -> Self {
        Self {
            _type: NodeType::RestElement,
            argument: arg,
            start: start_loc.pos,
            end: end_loc.pos,
            loc: SourceLocation {
                source,
                start: start_loc.loc,
                end: end_loc.loc,
            },
        }
    }
}

#[derive(Serialize)]
pub struct ArrayPattern {
    #[serde(rename(serialize = "type"))]
    pub _type: NodeType,
    pub start: usize,
    pub end: usize,
    pub loc: SourceLocation,
    pub elements: Vec<Option<Pattern>>,
}

impl ArrayPattern {
    pub fn new(elements: Vec<Option<Pattern>>, (start_loc, end_loc, source): AstNodePos) -> Self {
        Self {
            _type: NodeType::ArrayPattern,
            elements,
            start: start_loc.pos,
            end: end_loc.pos,
            loc: SourceLocation {
                source,
                start: start_loc.loc,
                end: end_loc.loc,
            },
        }
    }
}

#[derive(Serialize)]
pub enum AssignmentPropertyKind {
    Init,
}

// comment the redundant property which inherited from Property
#[derive(Serialize)]
pub struct AssignmentProperty {
    #[serde(rename(serialize = "type"))]
    pub _type: NodeType, // Property
    pub start: usize,
    pub end: usize,
    pub loc: SourceLocation,
    pub kind: AssignmentPropertyKind,
    pub value: Pattern,
    pub key: Expression,
    pub method: bool, // has to be false
    // if shorthand is true, computed must be false.
    pub shorthand: bool,
    pub computed: bool,
}

impl AssignmentProperty {
    pub fn new(
        key: Expression,
        value: Pattern,
        shorthand: bool,
        computed: bool,
        (start_loc, end_loc, source): AstNodePos,
    ) -> Self {
        Self {
            _type: NodeType::Property,
            key,
            value,
            shorthand,
            kind: AssignmentPropertyKind::Init,
            method: false,
            computed,
            start: start_loc.pos,
            end: end_loc.pos,
            loc: SourceLocation {
                source,
                start: start_loc.loc,
                end: end_loc.loc,
            },
        }
    }
}

// from es9, object pattern property support RestElement,
// eg.{a, ...rest} = obj
#[derive(Serialize)]
pub enum ObjectPatternProperty {
    AssignmentProperty(AssignmentProperty),
    RestElement(RestElement),
}

impl From<AssignmentProperty> for ObjectPatternProperty {
    fn from(value: AssignmentProperty) -> Self {
        Self::AssignmentProperty(value)
    }
}

impl From<RestElement> for ObjectPatternProperty {
    fn from(value: RestElement) -> Self {
        Self::RestElement(value)
    }
}

#[derive(Serialize)]
pub struct ObjectPattern {
    #[serde(rename(serialize = "type"))]
    pub _type: NodeType,
    pub start: usize,
    pub end: usize,
    pub loc: SourceLocation,
    pub properties: Vec<ObjectPatternProperty>,
}

impl ObjectPattern {
    pub fn new(
        properties: Vec<ObjectPatternProperty>,
        (start_loc, end_loc, source): AstNodePos,
    ) -> Self {
        Self {
            _type: NodeType::ObjectPattern,
            properties,
            start: start_loc.pos,
            end: end_loc.pos,
            loc: SourceLocation {
                source,
                start: start_loc.loc,
                end: end_loc.loc,
            },
        }
    }
}

#[derive(Serialize)]
pub struct AssignmentPattern {
    #[serde(rename(serialize = "type"))]
    pub _type: NodeType,
    pub start: usize,
    pub end: usize,
    pub loc: SourceLocation,
    pub left: Box<Pattern>,
    pub right: Box<Expression>,
}

impl AssignmentPattern {
    pub fn new(
        left: Box<Pattern>,
        right: Box<Expression>,
        (start_loc, end_loc, source_file): AstNodePos,
    ) -> Self {
        Self {
            _type: NodeType::AssignmentPattern,
            left,
            right,
            start: start_loc.pos,
            end: end_loc.pos,
            loc: SourceLocation {
                source: source_file,
                start: start_loc.loc,
                end: end_loc.loc,
            },
        }
    }
}
