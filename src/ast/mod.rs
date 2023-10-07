pub mod directive;
pub mod expression;
pub mod import_export_declaration;
pub mod pattern;
pub mod statement;

use self::{
    directive::Directive, import_export_declaration::ImportOrExportDeclaration,
    statement::Statement,
};
use serde::Serialize;

// Node 类型包含所有节点的类型，大部分是 StatementType
#[derive(Debug, Clone, Serialize)]
pub enum NodeType {
    Program,   // 根节点类型
    Undefiend, // 占位符
    ExpressionStatement,
    BlockStatement,
    EmptyStatement,
    DebuggerStatement,
    WithStatement,
    ReturnStatement,
    LabeledStatement,
    BreakStatement,
    ContinueStatement,
    IfStatement,
    SwitchStatement,
    ThrowStatement,
    TryStatement,
    WhileStatement,
    DoWhileStatement,
    ForStatement,
    ForInStatement,
    ForOfStatement,
    FunctionDeclaration,
    VariableDeclaration,
    VaraiableDeclarator,
    ClassDeclaration,
    SwitchCase,
    CatchClause,
    // Class Related
    Super,
    SpreadElement,
    TemplateElement,
    // 大部分是 Expression
    Identifier,
    Literal,
    ThisExpression,
    ArrayExpression,
    ObjectExpression,
    Property,
    FunctionExpression,
    ParenthesizedExpression,
    //  "-" | "+" | "!" | "~" | "typeof" | "void" | "delete"
    UnaryExpression,
    // "++" | "--"
    UpdateExpression,
    // "==" | "!=" | "===" | "!==" "<" | "<=" | ">" | ">=" | "<<" | ">>"
    // "">>>" | "+" | "-" | "*" | "/" | "%" "|" | "^" | "&" | "in" "instanceof"
    BinaryExpression,
    // "=" | "+=" | "-=" | "*=" | "/=" | "%=" | "<<=" |
    // ">>=" | ">>>=" | "|=" | "^=" | "&="
    AssignmentExpression,
    // "||" | "&&"
    LogicalExpression,
    // a[b] or a.b
    MemberExpression,
    // ? : ternary operatot
    ConditionalExpression,
    // a()
    CallExpression,
    // new A()
    NewExpression,
    // a, b
    SequenceExpression,
    // (a) => b,
    ArrowFunctionExpression,
    // yield a;
    YieldExpression,
    // `ab${c}`
    TemplateLiteral,
    TaggedTemplateExpression,
    ObjectPattern,
    ArrayPattern,
    RestElement,
    AssignmentPattern,
    ClassBody,
    MethodDefinition,
    ClassExpression,
    MetaProperty,
    ImportDeclaration,
    ImportSpecifier,
    ImportDefaultSpecifier,
    ImportNamespaceSpecifier,
    ExportNamedDeclaration,
    ExportAllDeclaration,
    ExportDefaultDeclaration,
    ExportSpecifier,
    ImportExpression,
    AwaitExpression,
    ChainExpression,
    PrivateIdentifier,
    StaticBlock,
}

pub fn create_program_node() -> Program {
    Program {
        _type: NodeType::Program,
        start: 0,
        end: 0,
        loc: SourceLocation {
            source: None,
            start: Position { line: 1, col: 0 },
            end: Position { line: 0, col: 0 },
        },
        body: vec![],
    }
}

pub fn get_location_at(start: usize, start_loc: Position) -> _LocationNode {
    _LocationNode {
        pos: start,
        loc: start_loc,
    }
}

pub type AstNodePos = (_LocationNode, _LocationNode, Option<String>);

// Rust 中没有继承，只有组合，关于为什么 Rust 不支持继承的原因，社区的理由是认为 OOP 会带来很多不必要的丑陋的编译后的代码的原因，
// 以及后续维护代码时灵活性不够难以拆解导致的耦合过高，然而使用组合的方式可以让代码实现更加简洁，解除继承带来的耦合。
// 问题讨论可见:
// https://users.rust-lang.org/t/how-to-think-without-field-inheritance/78116/11
// https://henrietteharmse.com/2015/04/18/the-rectanglesquare-controversy/
#[derive(Clone, Debug, Serialize)]
pub struct SourceLocation {
    pub source: Option<String>,
    pub start: Position,
    pub end: Position,
}

#[derive(Debug, Clone, Serialize)]
pub struct Position {
    pub line: usize,
    pub col: usize,
}

/// a node which contains location info only
#[derive(Clone)]
pub struct _LocationNode {
    pub pos: usize,
    pub loc: Position,
}

#[derive(Serialize)]
pub enum ProgramNode {
    Directive(Directive),
    Statement(Statement),
    // starts from es6
    ImportOrExportDeclaration(ImportOrExportDeclaration),
}

impl From<Statement> for ProgramNode {
    fn from(value: Statement) -> Self {
        Self::Statement(value)
    }
}

#[derive(Serialize)]
pub struct Program {
    #[serde(rename(serialize = "type"))]
    pub _type: NodeType,
    pub start: usize,
    pub end: usize,
    pub loc: SourceLocation,
    // 包含所有 programe node
    pub body: Vec<ProgramNode>,
}
