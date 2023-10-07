use super::{
    pattern::Pattern,
    statement::{ClassBody, FunctionBody, Statement},
    AstNodePos, NodeType, SourceLocation,
};
use regex::Regex;
use serde::Serialize;

#[derive(Serialize)]
pub enum Expression {
    Identifier(Identifier),
    Literal(Literal), // literal 的类型可以继续拆分为 number、string、bool、reg、bigint 等
    ThisExpression(ThisExpression),
    ArrayExpression(ArrayExpression),
    ObjectExpression(ObjectExpression),
    FunctionExpression(FunctionExpression),
    UnaryExpression(UnaryExpression),
    UpdateExpression(UpdateExpression),
    BinaryExpression(BinaryExpression),
    AssignmentExpression(AssignmentExpression),
    LogicalExpression(LogicalExpression),
    MemberExpression(MemberExpression), // like a[b]、a.b
    ConditionalExpression(ConditionalExpression),
    CallExpression(CallExpression), // a.func()
    NewExpression(NewExpression),   // new Object()
    SequenceExpression(SequenceExpression),
    ParenthesizedExpression(ParenthesizedExpression),
    // starts from es6
    ArrowFunctionExpression(ArrowFunctionExpression), // let f = () => {}
    YieldExpression(YieldExpression),                 // yield a;
    TemplateLiteral(TemplateLiteral),                 // `some template ${val}`
    TaggedTemplateExpression(TaggedTemplateExpression), // myTag`That ${person} is a ${age}.`
    ClassExpression(ClassExpression),
    MetaProperty(MetaProperty), // to check if function called with new operator
    // starts from es2017
    AwaitExpression(AwaitExpression),
    // starts from es2020
    ChainExpression(ChainExpression),   // ?.
    ImportExpression(ImportExpression), // import(source) represents dynamic import
    // starts from es2022
    StaticBlock(StaticBlock),
}

impl From<Identifier> for Expression {
    fn from(value: Identifier) -> Self {
        Self::Identifier(value)
    }
}

impl From<Literal> for Expression {
    fn from(value: Literal) -> Self {
        Self::Literal(value)
    }
}

impl From<ThisExpression> for Expression {
    fn from(value: ThisExpression) -> Self {
        Self::ThisExpression(value)
    }
}

impl From<ArrayExpression> for Expression {
    fn from(value: ArrayExpression) -> Self {
        Self::ArrayExpression(value)
    }
}

impl From<ObjectExpression> for Expression {
    fn from(value: ObjectExpression) -> Self {
        Self::ObjectExpression(value)
    }
}

impl From<FunctionExpression> for Expression {
    fn from(value: FunctionExpression) -> Self {
        Self::FunctionExpression(value)
    }
}

impl From<UnaryExpression> for Expression {
    fn from(value: UnaryExpression) -> Self {
        Self::UnaryExpression(value)
    }
}

impl From<UpdateExpression> for Expression {
    fn from(value: UpdateExpression) -> Self {
        Self::UpdateExpression(value)
    }
}

impl From<BinaryExpression> for Expression {
    fn from(value: BinaryExpression) -> Self {
        Self::BinaryExpression(value)
    }
}

impl From<AssignmentExpression> for Expression {
    fn from(value: AssignmentExpression) -> Self {
        Self::AssignmentExpression(value)
    }
}

impl From<LogicalExpression> for Expression {
    fn from(value: LogicalExpression) -> Self {
        Self::LogicalExpression(value)
    }
}

impl From<MemberExpression> for Expression {
    fn from(value: MemberExpression) -> Self {
        Self::MemberExpression(value)
    }
}

impl From<ConditionalExpression> for Expression {
    fn from(value: ConditionalExpression) -> Self {
        Self::ConditionalExpression(value)
    }
}

impl From<CallExpression> for Expression {
    fn from(value: CallExpression) -> Self {
        Self::CallExpression(value)
    }
}

impl From<NewExpression> for Expression {
    fn from(value: NewExpression) -> Self {
        Self::NewExpression(value)
    }
}

impl From<SequenceExpression> for Expression {
    fn from(value: SequenceExpression) -> Self {
        Self::SequenceExpression(value)
    }
}

impl From<ArrowFunctionExpression> for Expression {
    fn from(value: ArrowFunctionExpression) -> Self {
        Self::ArrowFunctionExpression(value)
    }
}

impl From<YieldExpression> for Expression {
    fn from(value: YieldExpression) -> Self {
        Self::YieldExpression(value)
    }
}

impl From<TemplateLiteral> for Expression {
    fn from(value: TemplateLiteral) -> Self {
        Self::TemplateLiteral(value)
    }
}

impl From<TaggedTemplateExpression> for Expression {
    fn from(value: TaggedTemplateExpression) -> Self {
        Self::TaggedTemplateExpression(value)
    }
}

impl From<ClassExpression> for Expression {
    fn from(value: ClassExpression) -> Self {
        Self::ClassExpression(value)
    }
}

impl From<MetaProperty> for Expression {
    fn from(value: MetaProperty) -> Self {
        Self::MetaProperty(value)
    }
}

impl From<AwaitExpression> for Expression {
    fn from(value: AwaitExpression) -> Self {
        Self::AwaitExpression(value)
    }
}

impl From<ChainExpression> for Expression {
    fn from(value: ChainExpression) -> Self {
        Self::ChainExpression(value)
    }
}

impl From<ImportExpression> for Expression {
    fn from(value: ImportExpression) -> Self {
        Self::ImportExpression(value)
    }
}

impl From<StaticBlock> for Expression {
    fn from(value: StaticBlock) -> Self {
        Self::StaticBlock(value)
    }
}

#[derive(Clone, Serialize)]
pub enum LiteralValue {
    String(String),
    Boolean(bool),
    Null,
    Number(i64),
    // use regex lib as js regexp's value
    #[serde(skip_serializing)]
    Regx(Option<Regex>),
    // from es11, add bigint type, if language env didn't support BigInt, leave it none.
    BigInt,
}

#[derive(Clone, Serialize)]
pub struct Reg {
    pub pattern: String,
    pub flags: String,
}

#[derive(Clone, Serialize)]
pub struct Literal {
    #[serde(rename(serialize = "type"))]
    pub _type: NodeType,
    pub value: LiteralValue,
    pub start: usize,
    pub end: usize,
    pub loc: SourceLocation,
    // 由于 reg 字面量拓展了字段，使用 Option 来展示
    pub reg: Option<Reg>,
    // from es 11, raw string exclude numberic separators.
    pub bigint: Option<String>,
}

impl Literal {
    pub fn new(
        value: LiteralValue,
        reg: Option<Reg>,
        bigint: Option<String>,
        (start_loc, end_loc, source_file): AstNodePos,
    ) -> Self {
        Self {
            _type: NodeType::Literal,
            value,
            reg,
            bigint,
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

#[derive(Serialize)]
pub struct ParenthesizedExpression {
    #[serde(rename(serialize = "type"))]
    pub _type: NodeType,
    pub start: usize,
    pub end: usize,
    pub loc: SourceLocation,
    pub expression: Box<Expression>,
}

impl ParenthesizedExpression {
    pub fn new(expression: Box<Expression>, (start_loc, end_loc, source): AstNodePos) -> Self {
        Self {
            _type: NodeType::ParenthesizedExpression,
            expression,
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
pub struct ThisExpression {
    #[serde(rename(serialize = "type"))]
    pub _type: NodeType,
    pub start: usize,
    pub end: usize,
    pub loc: SourceLocation,
}

impl ThisExpression {
    pub fn new((start_loc, end_loc, source): AstNodePos) -> Self {
        Self {
            _type: NodeType::ThisExpression,
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
pub struct SpreadElement {
    #[serde(rename(serialize = "type"))]
    pub _type: NodeType,
    pub start: usize,
    pub end: usize,
    pub loc: SourceLocation,
    pub argument: Expression,
}

impl SpreadElement {
    pub fn new(argument: Expression, (start_loc, end_loc, source_file): AstNodePos) -> Self {
        Self {
            _type: NodeType::SpreadElement,
            argument,
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

#[derive(Serialize)]
pub enum ArrayExprEle {
    Expression(Expression),
    SpreadElement(SpreadElement),
    // use Null for case like E.g. [1,,2], None represents null
    Null,
}

#[derive(Serialize)]
pub struct ArrayExpression {
    #[serde(rename(serialize = "type"))]
    pub _type: NodeType,
    pub start: usize,
    pub end: usize,
    pub loc: SourceLocation,
    pub elements: Vec<ArrayExprEle>,
}

impl ArrayExpression {
    pub fn new(elements: Vec<ArrayExprEle>, (start_loc, end_loc, source): AstNodePos) -> Self {
        Self {
            _type: NodeType::ArrayExpression,
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

#[derive(PartialEq, Eq, Clone, Serialize)]
pub enum PropertyKind {
    Init,
    Get,
    Set,
}

// define object property structure
#[derive(Serialize)]
pub struct Property {
    #[serde(rename(serialize = "type"))]
    pub _type: NodeType, // Property
    pub start: usize,
    pub end: usize,
    pub loc: SourceLocation,
    pub kind: PropertyKind,
    pub value: Expression,
    pub key: Expression,
    // extends from es6
    pub method: bool,
    pub shorthand: bool,
    pub computed: bool,
}

impl Property {
    pub fn new(
        key: Expression,
        value: Expression,
        kind: PropertyKind,
        method: bool,
        shorthand: bool,
        computed: bool,
        (start_loc, end_loc, source): AstNodePos,
    ) -> Self {
        Self {
            _type: NodeType::Property,
            key,
            value,
            kind,
            method,
            shorthand,
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

// from es9, supports spread element as properties, e.g., {a: 1, ...obj, b: 2}.
#[derive(Serialize)]
pub enum ObjectProperty {
    Property(Property),
    SpreadElement(SpreadElement),
}

#[derive(Serialize)]
pub struct ObjectExpression {
    #[serde(rename(serialize = "type"))]
    pub _type: NodeType,
    pub start: usize,
    pub end: usize,
    pub loc: SourceLocation,
    pub properties: Vec<ObjectProperty>,
}

impl ObjectExpression {
    pub fn new(properties: Vec<ObjectProperty>, (start_loc, end_loc, source): AstNodePos) -> Self {
        Self {
            _type: NodeType::ObjectExpression,
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

// ast nodes definition
#[derive(Clone, Serialize)]
pub struct Identifier {
    #[serde(rename(serialize = "type"))]
    pub _type: NodeType,
    pub name: String,
    pub start: usize,
    pub end: usize,
    pub loc: SourceLocation,
}

impl Identifier {
    pub fn new(name: String, (start_loc, end_loc, source_file): AstNodePos) -> Self {
        Self {
            _type: NodeType::Identifier,
            name,
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

#[derive(Serialize)]
pub struct FunctionExpression {
    #[serde(rename(serialize = "type"))]
    pub _type: NodeType,
    pub start: usize,
    pub end: usize,
    pub loc: SourceLocation,
    pub id: Option<Identifier>,
    pub params: Vec<Pattern>,
    pub body: FunctionBody,
    // starts from es6
    pub generator: bool,
    // from es8
    pub is_async: bool,
}

impl FunctionExpression {
    pub fn new(
        id: Option<Identifier>,
        params: Vec<Pattern>,
        body: FunctionBody,
        generator: bool,
        is_async: bool,
        (start_loc, end_loc, source): AstNodePos,
    ) -> Self {
        Self {
            _type: NodeType::FunctionExpression,
            id,
            params,
            body,
            generator,
            is_async,
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
pub enum UnaryOperator {
    Minus,       // '-'
    Plus,        // '+'
    Exclamation, // '!'
    Tilde,       // '~'
    Typeof,      // 'typeof'
    Void,        // 'void'
    Delete,      // 'delete'
}

impl From<String> for UnaryOperator {
    fn from(value: String) -> Self {
        match value.as_str() {
            "-" => Self::Minus,
            "+" => Self::Plus,
            "!" => Self::Exclamation,
            "~" => Self::Tilde,
            "typeof" => Self::Typeof,
            "void" => Self::Void,
            "delte" => Self::Delete,
            _ => {
                panic!("Unexpected operator {}", value);
            }
        }
    }
}

#[derive(Serialize)]
pub struct UnaryExpression {
    #[serde(rename(serialize = "type"))]
    pub _type: NodeType,
    pub start: usize,
    pub end: usize,
    pub loc: SourceLocation,
    pub operator: UnaryOperator,
    pub prefix: bool,
    pub argument: Box<Expression>,
}

impl UnaryExpression {
    pub fn new(
        operator: UnaryOperator,
        argument: Box<Expression>,
        prefix: bool,
        (start_loc, end_loc, source): AstNodePos,
    ) -> Self {
        Self {
            _type: NodeType::UnaryExpression,
            operator,
            argument,
            prefix,
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
pub enum UpdateOperator {
    PlusPlus,
    MinusMinus,
}

impl From<String> for UpdateOperator {
    fn from(value: String) -> Self {
        match value.as_str() {
            "++" => UpdateOperator::PlusPlus,
            "--" => UpdateOperator::MinusMinus,
            _ => {
                panic!("Unexpected update operator {}", value);
            }
        }
    }
}

#[derive(Serialize)]
pub struct UpdateExpression {
    #[serde(rename(serialize = "type"))]
    pub _type: NodeType,
    pub start: usize,
    pub end: usize,
    pub loc: SourceLocation,
    pub operator: UpdateOperator,
    pub argument: Box<Expression>,
    pub prefix: bool,
}

impl UpdateExpression {
    pub fn new(
        operator: UpdateOperator,
        argument: Box<Expression>,
        prefix: bool,
        (start_loc, end_loc, source): AstNodePos,
    ) -> Self {
        Self {
            _type: NodeType::UpdateExpression,
            operator,
            argument,
            prefix,
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
pub enum BinaryOperator {
    Equality,           // '=='
    InEquality,         // '!='
    StrictEquality,     // '==='
    StrictInEquality,   // '!=='
    Greater,            // '>'
    GreaterOrEqual,     // '>=',
    RightShift,         // '>>'
    Less,               // '<'
    LessOrEqual,        // '<='
    LeftShift,          // '<<'
    UnsignedRightShift, // '>>>'
    Plus,               // '+',
    Minus,              // '-'
    Multipl,            // '*'
    Division,           // '/'
    Reminder,           // '%'
    BitwiseOr,          // '|'
    BitwiseAnd,         // '&'
    BitwiseXor,         // '^'
    In,                 // 'in'
    InstanceOf,         // 'instanceof'
    // from es7
    Exponentiation, // '**'
}

impl From<String> for BinaryOperator {
    fn from(value: String) -> Self {
        match value.as_str() {
            "==" => Self::Equality,
            "!=" => Self::InEquality,
            "===" => Self::StrictEquality,
            "!==" => Self::StrictInEquality,
            ">" => Self::Greater,
            ">=" => Self::GreaterOrEqual,
            ">>" => Self::RightShift,
            "<" => Self::Less,
            "<=" => Self::LessOrEqual,
            "<<" => Self::LeftShift,
            ">>>" => Self::UnsignedRightShift,
            "+" => Self::Plus,
            "-" => Self::Minus,
            "*" => Self::Multipl,
            "/" => Self::Division,
            "%" => Self::Reminder,
            "|" => Self::BitwiseOr,
            "&" => Self::BitwiseAnd,
            "^" => Self::BitwiseXor,
            "in" => Self::In,
            "instanceof" => Self::InstanceOf,
            "**" => Self::Exponentiation,
            _ => {
                panic!("Unexpected binary operator {}", value);
            }
        }
    }
}

// left could be PrivateIdentifier when operator is 'in'
#[derive(Serialize)]
pub enum BinaryOpeartorLeft {
    Expression(Box<Expression>),
    PrivateIdentifier(PrivateIdentifier),
}

#[derive(Serialize)]
pub struct BinaryExpression {
    #[serde(rename(serialize = "type"))]
    pub _type: NodeType,
    pub start: usize,
    pub end: usize,
    pub loc: SourceLocation,
    pub operator: BinaryOperator,
    pub left: BinaryOpeartorLeft,
    pub right: Box<Expression>,
}

impl BinaryExpression {
    pub fn new(
        left: BinaryOpeartorLeft,
        operator: BinaryOperator,
        right: Box<Expression>,
        (start_loc, end_loc, source): AstNodePos,
    ) -> Self {
        Self {
            _type: NodeType::BinaryExpression,
            left,
            operator,
            right,
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
pub enum AssignmentOperator {
    Assignment,               // '='
    AdditionAssign,           // "+="
    SubtractionAssign,        // "-="
    MultiplAssign,            // "*="
    DivisionAssign,           // "/="
    RemainderAssign,          // "%="
    LeftShiftAssign,          //  "<<="
    RightShiftAssign,         // ">>="
    UnsignedRightShiftAssign, // ">>>="
    BitwiseORAssign,          // "|="
    BitwiseANDAssign,         // "&="
    BitwiseXORAssign,         // "^="
    // after es5
    LogicalANDAssign,        // "&&="
    LogicalORAssign,         // "||="
    NullishCoalescingAssign, // "??="
    ExponentiationAssign,    // "**="
}

impl AssignmentOperator {
    pub fn as_str(&self) -> String {
        match self {
            Self::Assignment => "=".to_string(),
            Self::AdditionAssign => "+=".to_string(),
            Self::SubtractionAssign => "-=".to_string(),
            Self::MultiplAssign => "*=".to_string(),
            Self::DivisionAssign => "/=".to_string(),
            Self::RemainderAssign => "%=".to_string(),
            Self::LeftShiftAssign => "<<=".to_string(),
            Self::RightShiftAssign => ">>=".to_string(),
            Self::UnsignedRightShiftAssign => ">>>=".to_string(),
            Self::BitwiseORAssign => "|=".to_string(),
            Self::BitwiseANDAssign => "&=".to_string(),
            Self::BitwiseXORAssign => "^=".to_string(),
            Self::LogicalANDAssign => "&&=".to_string(),
            Self::LogicalORAssign => "||=".to_string(),
            Self::NullishCoalescingAssign => "??=".to_string(),
            Self::ExponentiationAssign => "**=".to_string(),
        }
    }
}

impl From<String> for AssignmentOperator {
    fn from(value: String) -> Self {
        match value.as_str() {
            "=" => Self::Assignment,
            "+=" => Self::AdditionAssign,
            "-=" => Self::SubtractionAssign,
            "*=" => Self::MultiplAssign,
            "/=" => Self::DivisionAssign,
            "%=" => Self::RemainderAssign,
            "<<=" => Self::LeftShiftAssign,
            ">>=" => Self::RightShiftAssign,
            ">>>=" => Self::UnsignedRightShiftAssign,
            "|=" => Self::BitwiseORAssign,
            "&=" => Self::BitwiseANDAssign,
            "^=" => Self::BitwiseXORAssign,
            "&&=" => Self::LogicalANDAssign,
            "||=" => Self::LogicalORAssign,
            "??=" => Self::NullishCoalescingAssign,
            "**=" => Self::ExponentiationAssign,
            _ => {
                panic!("unexpected assignment operator {}", value)
            }
        }
    }
}

#[derive(Serialize)]
pub enum AssignmentExpressionLeft {
    Pattern(Pattern),
    Expression(Box<Expression>),
}

impl From<Pattern> for AssignmentExpressionLeft {
    fn from(value: Pattern) -> Self {
        Self::Pattern(value)
    }
}

impl From<Expression> for AssignmentExpressionLeft {
    fn from(value: Expression) -> Self {
        Self::Expression(Box::new(value))
    }
}

#[derive(Serialize)]
pub struct AssignmentExpression {
    #[serde(rename(serialize = "type"))]
    pub _type: NodeType,
    pub start: usize,
    pub end: usize,
    pub loc: SourceLocation,
    pub operator: AssignmentOperator,
    pub left: AssignmentExpressionLeft,
    pub right: Box<Expression>,
}

impl AssignmentExpression {
    pub fn new(
        left: AssignmentExpressionLeft,
        operator: AssignmentOperator,
        right: Box<Expression>,
        (start_loc, end_loc, source): AstNodePos,
    ) -> Self {
        Self {
            _type: NodeType::AssignmentExpression,
            left,
            operator,
            right,
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
pub enum LogicalOperator {
    And,     // '&&'
    Or,      // '||'
    Nullish, // '??'
}

impl From<String> for LogicalOperator {
    fn from(value: String) -> Self {
        match value.as_str() {
            "&&" => LogicalOperator::And,
            "||" => LogicalOperator::Or,
            "??" => LogicalOperator::Nullish,
            _ => {
                panic!("Unexpected Logical operator {}", value);
            }
        }
    }
}

#[derive(Serialize)]
pub struct LogicalExpression {
    #[serde(rename(serialize = "type"))]
    pub _type: NodeType,
    pub start: usize,
    pub end: usize,
    pub loc: SourceLocation,
    pub operator: LogicalOperator,
    pub left: Box<Expression>,
    pub right: Box<Expression>,
}

impl LogicalExpression {
    pub fn new(
        left: Box<Expression>,
        operator: LogicalOperator,
        right: Box<Expression>,
        (start_loc, end_loc, source): AstNodePos,
    ) -> Self {
        Self {
            _type: NodeType::LogicalExpression,
            left,
            operator,
            right,
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
pub struct Super {
    #[serde(rename(serialize = "type"))]
    pub _type: NodeType,
    pub start: usize,
    pub end: usize,
    pub loc: SourceLocation,
}

impl Super {
    pub fn new((start_loc, end_loc, source_file): AstNodePos) -> Self {
        Self {
            _type: NodeType::Super,
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

#[derive(Serialize)]
pub enum MemberExprObject {
    Expression(Box<Expression>),
    Super(Super),
}

impl From<Super> for MemberExprObject {
    fn from(value: Super) -> Self {
        Self::Super(value)
    }
}

impl From<Expression> for MemberExprObject {
    fn from(value: Expression) -> Self {
        Self::Expression(Box::new(value))
    }
}

#[derive(Serialize)]
pub enum MemberExprProperty {
    Expression(Box<Expression>),
    PrivateIdentifier(PrivateIdentifier),
}

impl From<PrivateIdentifier> for MemberExprProperty {
    fn from(value: PrivateIdentifier) -> Self {
        Self::PrivateIdentifier(value)
    }
}

impl From<Expression> for MemberExprProperty {
    fn from(value: Expression) -> Self {
        Self::Expression(Box::new(value))
    }
}

#[derive(Serialize)]
pub struct MemberExpression {
    #[serde(rename(serialize = "type"))]
    pub _type: NodeType,
    pub start: usize,
    pub end: usize,
    pub loc: SourceLocation,
    pub object: MemberExprObject,
    pub property: MemberExprProperty,
    pub computed: bool,
    // from es11, for optinal chaining
    pub optional: bool,
}

impl MemberExpression {
    pub fn new(
        object: MemberExprObject,
        property: MemberExprProperty,
        computed: bool,
        optional: bool,
        (start_loc, end_loc, source): AstNodePos,
    ) -> Self {
        Self {
            _type: NodeType::MemberExpression,
            object,
            property,
            computed,
            optional,
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

// ternary ?/: expression
#[derive(Serialize)]
pub struct ConditionalExpression {
    #[serde(rename(serialize = "type"))]
    pub _type: NodeType,
    pub start: usize,
    pub end: usize,
    pub loc: SourceLocation,
    pub test: Box<Expression>,
    pub alternate: Box<Expression>,
    pub consequent: Box<Expression>,
}

impl ConditionalExpression {
    pub fn new(
        test: Box<Expression>,
        consequent: Box<Expression>,
        alternate: Box<Expression>,
        (start_loc, end_loc, source): AstNodePos,
    ) -> Self {
        Self {
            _type: NodeType::ConditionalExpression,
            test,
            consequent,
            alternate,
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
pub enum CallExprArgs {
    Expression(Expression),
    SpreadElement(SpreadElement),
}

#[derive(Serialize)]
pub enum CallExprCallee {
    Expression(Box<Expression>),
    Super(Super),
}

impl From<Super> for CallExprCallee {
    fn from(value: Super) -> Self {
        Self::Super(value)
    }
}

impl From<Expression> for CallExprCallee {
    fn from(value: Expression) -> Self {
        Self::Expression(Box::new(value))
    }
}

#[derive(Serialize)]
pub struct CallExpression {
    #[serde(rename(serialize = "type"))]
    pub _type: NodeType,
    pub start: usize,
    pub end: usize,
    pub loc: SourceLocation,
    pub callee: CallExprCallee,
    pub arguments: Vec<CallExprArgs>,
    // from es11, for optinal chianing
    pub optional: bool,
}

impl CallExpression {
    pub fn new(
        callee: CallExprCallee,
        arguments: Vec<CallExprArgs>,
        optional: bool,
        (start_loc, end_loc, source): AstNodePos,
    ) -> Self {
        Self {
            _type: NodeType::CallExpression,
            callee,
            arguments,
            optional,
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
pub enum NewExprArgs {
    Expression(Expression),
    SpreadElement(SpreadElement),
}

impl From<Expression> for NewExprArgs {
    fn from(value: Expression) -> Self {
        Self::Expression(value)
    }
}

impl From<SpreadElement> for NewExprArgs {
    fn from(value: SpreadElement) -> Self {
        Self::SpreadElement(value)
    }
}

#[derive(Serialize)]
pub struct NewExpression {
    #[serde(rename(serialize = "type"))]
    pub _type: NodeType,
    pub start: usize,
    pub end: usize,
    pub loc: SourceLocation,
    pub callee: Box<Expression>,
    pub arguments: Vec<NewExprArgs>,
}

impl NewExpression {
    pub fn new(
        callee: Box<Expression>,
        arguments: Vec<NewExprArgs>,
        (start_loc, end_loc, source): AstNodePos,
    ) -> Self {
        Self {
            _type: NodeType::NewExpression,
            callee,
            arguments,
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

// comma-separated sequence of expressions, eg. a,b,c
#[derive(Serialize)]
pub struct SequenceExpression {
    #[serde(rename(serialize = "type"))]
    pub _type: NodeType,
    pub start: usize,
    pub end: usize,
    pub loc: SourceLocation,
    pub expressions: Vec<Expression>,
}

impl SequenceExpression {
    pub fn new(expressions: Vec<Expression>, (start_loc, end_loc, source): AstNodePos) -> Self {
        Self {
            _type: NodeType::SequenceExpression,
            expressions,
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
pub enum ArrowFunctionBody {
    FunctionBoby(FunctionBody),
    Expression(Box<Expression>),
}

// Note: there is not generator arrow function expression.
#[derive(Serialize)]
pub struct ArrowFunctionExpression {
    #[serde(rename(serialize = "type"))]
    pub _type: NodeType,
    pub start: usize,
    pub end: usize,
    pub loc: SourceLocation,
    pub params: Vec<Pattern>,
    pub body: ArrowFunctionBody,
    pub expression: bool,
    pub is_async: bool,
}

impl ArrowFunctionExpression {
    pub fn new(
        params: Vec<Pattern>,
        body: ArrowFunctionBody,
        expression: bool,
        is_async: bool,
        (start_loc, end_loc, source_file): AstNodePos,
    ) -> Self {
        Self {
            _type: NodeType::ArrowFunctionExpression,
            params,
            body,
            expression,
            is_async,
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

#[derive(Serialize)]
pub struct YieldExpression {
    #[serde(rename(serialize = "type"))]
    pub _type: NodeType,
    pub start: usize,
    pub end: usize,
    pub loc: SourceLocation,
    pub argument: Option<Box<Expression>>,
    pub delegate: bool,
}

impl YieldExpression {
    pub fn new(
        delegate: bool,
        argument: Option<Box<Expression>>,
        (start_loc, end_loc, source_file): AstNodePos,
    ) -> Self {
        Self {
            _type: NodeType::YieldExpression,
            delegate,
            argument,
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

// from es9:
// if it's tagged and there is invalid escape, raw should be null
// eg. tag`\unicode and \u{55}`
#[derive(Serialize)]
pub struct TemplateValue {
    pub cooked: Option<String>,
    pub raw: String,
}

#[derive(Serialize)]
pub struct TemplateElement {
    #[serde(rename(serialize = "type"))]
    pub _type: NodeType,
    pub start: usize,
    pub end: usize,
    pub loc: SourceLocation,
    pub tail: bool,
    pub value: TemplateValue,
}

impl TemplateElement {
    pub fn new(
        value: TemplateValue,
        tail: bool,
        (start_loc, end_loc, source_file): AstNodePos,
    ) -> Self {
        Self {
            _type: NodeType::TemplateElement,
            value,
            tail,
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

#[derive(Serialize)]
pub struct TemplateLiteral {
    #[serde(rename(serialize = "type"))]
    pub _type: NodeType,
    pub start: usize,
    pub end: usize,
    pub loc: SourceLocation,
    pub quasis: Vec<TemplateElement>,
    pub expressions: Vec<Expression>,
}

impl TemplateLiteral {
    pub fn new(
        quasis: Vec<TemplateElement>,
        expressions: Vec<Expression>,
        (start_loc, end_loc, source_file): AstNodePos,
    ) -> Self {
        Self {
            _type: NodeType::TemplateLiteral,
            quasis,
            expressions,
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

#[derive(Serialize)]
pub struct TaggedTemplateExpression {
    #[serde(rename(serialize = "type"))]
    pub _type: NodeType,
    pub start: usize,
    pub end: usize,
    pub loc: SourceLocation,
    pub tag: Box<Expression>,
    pub quasi: TemplateLiteral,
}

impl TaggedTemplateExpression {
    pub fn new(
        tag: Box<Expression>,
        quasi: TemplateLiteral,
        (start_loc, end_loc, source_file): AstNodePos,
    ) -> Self {
        Self {
            _type: NodeType::TaggedTemplateExpression,
            tag,
            quasi,
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

#[derive(Serialize)]
pub struct ClassExpression {
    #[serde(rename(serialize = "type"))]
    pub _type: NodeType,
    pub start: usize,
    pub end: usize,
    pub loc: SourceLocation,
    pub id: Option<Identifier>,
    pub super_class: Option<Box<Expression>>,
    pub body: ClassBody,
}

impl ClassExpression {
    pub fn new(
        id: Option<Identifier>,
        super_class: Option<Box<Expression>>,
        body: ClassBody,
        (start_loc, end_loc, source_file): AstNodePos,
    ) -> Self {
        Self {
            _type: NodeType::ClassExpression,
            id,
            super_class,
            body,
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

// MetaProperty node represents new.target
#[derive(Serialize)]
pub struct MetaProperty {
    #[serde(rename(serialize = "type"))]
    pub _type: NodeType,
    pub start: usize,
    pub end: usize,
    pub loc: SourceLocation,
    pub meta: Identifier,
    pub property: Identifier,
}

impl MetaProperty {
    pub fn new(
        meta: Identifier,
        property: Identifier,
        (start_loc, end_loc, source_file): AstNodePos,
    ) -> Self {
        Self {
            _type: NodeType::MetaProperty,
            meta,
            property,
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

#[derive(Serialize)]
pub struct AwaitExpression {
    #[serde(rename(serialize = "type"))]
    pub _type: NodeType,
    pub start: usize,
    pub end: usize,
    pub loc: SourceLocation,
    pub argument: Box<Expression>,
}

impl AwaitExpression {
    pub fn new(argument: Box<Expression>, (start_loc, end_loc, source_file): AstNodePos) -> Self {
        Self {
            _type: NodeType::AwaitExpression,
            argument,
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

pub struct ChainElement {
    pub optional: bool,
}

#[derive(Serialize)]
pub enum ChainExpressionElement {
    CallExpression(CallExpression),
    MemberExpression(MemberExpression),
}

// from es11
// ChainExpression Node is the root of optional chaining.
// Note:
// (obj?.aaa).bbb
// {
//   "type": "MemberExpression",
//   "optional": false,
//   "object": {
//     "type": "ChainExpression",
//     "expression": {
//       "type": "MemberExpression",
//       "optional": true,
//       "object": { "type": "Identifier", "name": "obj" },
//       "property": { "type": "Identifier", "name": "aaa" }
//     }
//   },
//   "property": { "type": "Identifier", "name": "bbb" }
// }
// Which should MemberExpression should be the root node.
#[derive(Serialize)]
pub struct ChainExpression {
    #[serde(rename(serialize = "type"))]
    pub _type: NodeType,
    pub start: usize,
    pub end: usize,
    pub loc: SourceLocation,
    pub expression: ChainExpressionElement,
}

impl ChainExpression {
    pub fn new(
        expression: ChainExpressionElement,
        (start_loc, end_loc, source_file): AstNodePos,
    ) -> Self {
        Self {
            _type: NodeType::ChainExpression,
            expression,
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

// for dynamic import such as import(source)
#[derive(Serialize)]
pub struct ImportExpression {
    #[serde(rename(serialize = "type"))]
    pub _type: NodeType,
    pub start: usize,
    pub end: usize,
    pub loc: SourceLocation,
    pub source: Box<Expression>,
}

impl ImportExpression {
    pub fn new(source: Box<Expression>, (start_loc, end_loc, source_file): AstNodePos) -> Self {
        Self {
            _type: NodeType::ImportExpression,
            source,
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

#[derive(Serialize)]
pub struct StaticBlock {
    #[serde(rename(serialize = "type"))]
    pub _type: NodeType,
    pub start: usize,
    pub end: usize,
    pub loc: SourceLocation,
    pub body: Vec<Statement>,
}

impl StaticBlock {
    pub fn new(body: Vec<Statement>, (start_loc, end_loc, source_file): AstNodePos) -> Self {
        Self {
            _type: NodeType::StaticBlock,
            body,
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

// class field whichs name starts with #, For a private name #a, its name is a.
#[derive(Serialize)]
pub struct PrivateIdentifier {
    #[serde(rename(serialize = "type"))]
    pub _type: NodeType,
    pub start: usize,
    pub end: usize,
    pub loc: SourceLocation,
    pub name: String,
}

impl PrivateIdentifier {
    pub fn new(name: String, (start_loc, end_loc, source_file): AstNodePos) -> Self {
        Self {
            _type: NodeType::PrivateIdentifier,
            name,
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
