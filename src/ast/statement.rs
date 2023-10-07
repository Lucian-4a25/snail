use serde::Serialize;

use super::{
    directive::Directive,
    expression::{Expression, FunctionExpression, Identifier, PrivateIdentifier, StaticBlock},
    import_export_declaration::{
        AnonymousDefaultExportedClassDeclaration, AnonymousDefaultExportedFunctionDeclaration,
    },
    pattern::Pattern,
    AstNodePos, NodeType, SourceLocation,
};

#[derive(Serialize)]
pub enum Statement {
    ExpressionStatement(ExpressionStatement),
    BlockStatement(BlockStatement),
    EmptyStatement(EmptyStatement),
    DebuggerStatement(DebuggerStatement),
    WithStatement(WithStatement),
    ReturnStatement(ReturnStatement),
    LabeledStatement(LabeledStatement),
    BreakStatement(BreakStatement),
    ContinueStatement(ContinueStatement),
    IfStatement(IfStatement),
    SwitchStatement(SwitchStatement),
    ThrowStatement(ThrowStatement),
    TryStatement(TryStatement),
    WhileStatement(WhileStatement),
    DoWhileStatement(DoWhileStatement),
    ForStatement(ForStatement),
    ForInStatement(ForInStatement),
    // starts from es6
    ForOfStatement(ForOfStatement),
    // 在 ESTree 的定义里，declaration 也属于 Statement
    FunctionDeclaration(FunctionDeclaration),
    AnonymousDefaultExportedFunctionDeclaration(AnonymousDefaultExportedFunctionDeclaration),
    AnonymousDefaultExportedClassDeclaration(AnonymousDefaultExportedClassDeclaration),
    VariableDeclaration(VariableDeclaration),
    // starts from es6
    ClassDeclaration(ClassDeclaration),
}

impl From<ExpressionStatement> for Statement {
    fn from(value: ExpressionStatement) -> Self {
        Self::ExpressionStatement(value)
    }
}

impl From<BlockStatement> for Statement {
    fn from(value: BlockStatement) -> Self {
        Self::BlockStatement(value)
    }
}

impl From<EmptyStatement> for Statement {
    fn from(value: EmptyStatement) -> Self {
        Self::EmptyStatement(value)
    }
}

impl From<DebuggerStatement> for Statement {
    fn from(value: DebuggerStatement) -> Self {
        Self::DebuggerStatement(value)
    }
}

impl From<WithStatement> for Statement {
    fn from(value: WithStatement) -> Self {
        Self::WithStatement(value)
    }
}

impl From<ReturnStatement> for Statement {
    fn from(value: ReturnStatement) -> Self {
        Self::ReturnStatement(value)
    }
}

impl From<LabeledStatement> for Statement {
    fn from(value: LabeledStatement) -> Self {
        Self::LabeledStatement(value)
    }
}

impl From<BreakStatement> for Statement {
    fn from(value: BreakStatement) -> Self {
        Self::BreakStatement(value)
    }
}

impl From<ContinueStatement> for Statement {
    fn from(value: ContinueStatement) -> Self {
        Self::ContinueStatement(value)
    }
}

impl From<IfStatement> for Statement {
    fn from(value: IfStatement) -> Self {
        Self::IfStatement(value)
    }
}

impl From<SwitchStatement> for Statement {
    fn from(value: SwitchStatement) -> Self {
        Self::SwitchStatement(value)
    }
}

impl From<ThrowStatement> for Statement {
    fn from(value: ThrowStatement) -> Self {
        Self::ThrowStatement(value)
    }
}

impl From<TryStatement> for Statement {
    fn from(value: TryStatement) -> Self {
        Self::TryStatement(value)
    }
}

impl From<WhileStatement> for Statement {
    fn from(value: WhileStatement) -> Self {
        Self::WhileStatement(value)
    }
}

impl From<DoWhileStatement> for Statement {
    fn from(value: DoWhileStatement) -> Self {
        Self::DoWhileStatement(value)
    }
}

impl From<ForStatement> for Statement {
    fn from(value: ForStatement) -> Self {
        Self::ForStatement(value)
    }
}

impl From<ForInStatement> for Statement {
    fn from(value: ForInStatement) -> Self {
        Self::ForInStatement(value)
    }
}

impl From<ForOfStatement> for Statement {
    fn from(value: ForOfStatement) -> Self {
        Self::ForOfStatement(value)
    }
}

impl From<FunctionDeclaration> for Statement {
    fn from(value: FunctionDeclaration) -> Self {
        Self::FunctionDeclaration(value)
    }
}

impl From<AnonymousDefaultExportedFunctionDeclaration> for Statement {
    fn from(value: AnonymousDefaultExportedFunctionDeclaration) -> Self {
        Self::AnonymousDefaultExportedFunctionDeclaration(value)
    }
}

impl From<VariableDeclaration> for Statement {
    fn from(value: VariableDeclaration) -> Self {
        Self::VariableDeclaration(value)
    }
}

impl From<ClassDeclaration> for Statement {
    fn from(value: ClassDeclaration) -> Self {
        Self::ClassDeclaration(value)
    }
}

// statements definition starts
#[derive(Serialize)]
pub struct ExpressionStatement {
    #[serde(rename(serialize = "type"))]
    pub _type: NodeType,
    pub start: usize,
    pub end: usize,
    pub loc: SourceLocation,
    pub expression: Expression,
}

impl ExpressionStatement {
    pub fn new(expr: Expression, (start_loc, end_loc, source): AstNodePos) -> Self {
        Self {
            _type: NodeType::ExpressionStatement,
            expression: expr,
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
pub struct BlockStatement {
    #[serde(rename(serialize = "type"))]
    pub _type: NodeType,
    pub start: usize,
    pub end: usize,
    pub loc: SourceLocation,
    pub body: Vec<Statement>,
}

impl BlockStatement {
    pub fn new(body: Vec<Statement>, (start_loc, end_loc, source): AstNodePos) -> Self {
        Self {
            _type: NodeType::BlockStatement,
            body,
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

// for a solitary semicolon
#[derive(Serialize)]
pub struct EmptyStatement {
    #[serde(rename(serialize = "type"))]
    pub _type: NodeType,
    pub start: usize,
    pub end: usize,
    pub loc: SourceLocation,
}

impl EmptyStatement {
    pub fn new((start_loc, end_loc, source): AstNodePos) -> Self {
        Self {
            _type: NodeType::EmptyStatement,
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
pub struct DebuggerStatement {
    #[serde(rename(serialize = "type"))]
    pub _type: NodeType,
    pub start: usize,
    pub end: usize,
    pub loc: SourceLocation,
}

impl DebuggerStatement {
    pub fn new((start_loc, end_loc, source): AstNodePos) -> Self {
        Self {
            _type: NodeType::DebuggerStatement,
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
pub struct WithStatement {
    #[serde(rename(serialize = "type"))]
    pub _type: NodeType,
    pub start: usize,
    pub end: usize,
    pub loc: SourceLocation,
    pub object: Expression,
    pub body: Box<Statement>,
}

impl WithStatement {
    pub fn new(
        object: Expression,
        body: Box<Statement>,
        (start_loc, end_loc, source): AstNodePos,
    ) -> Self {
        Self {
            _type: NodeType::WithStatement,
            object,
            body,
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
pub struct ReturnStatement {
    #[serde(rename(serialize = "type"))]
    pub _type: NodeType,
    pub start: usize,
    pub end: usize,
    pub loc: SourceLocation,
    pub argument: Option<Expression>,
}

impl ReturnStatement {
    pub fn new(argument: Option<Expression>, (start_loc, end_loc, source): AstNodePos) -> Self {
        Self {
            _type: NodeType::ReturnStatement,
            argument,
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
pub struct LabeledStatement {
    #[serde(rename(serialize = "type"))]
    pub _type: NodeType,
    pub start: usize,
    pub end: usize,
    pub loc: SourceLocation,
    pub label: Identifier,
    pub body: Box<Statement>,
}

impl LabeledStatement {
    pub fn new(
        label: Identifier,
        body: Box<Statement>,
        (start_loc, end_loc, source): AstNodePos,
    ) -> Self {
        Self {
            _type: NodeType::LabeledStatement,
            label,
            body,
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
pub struct BreakStatement {
    #[serde(rename(serialize = "type"))]
    pub _type: NodeType,
    pub start: usize,
    pub end: usize,
    pub loc: SourceLocation,
    pub label: Option<Identifier>,
}

impl BreakStatement {
    pub fn new(label: Option<Identifier>, (start_loc, end_loc, source): AstNodePos) -> Self {
        Self {
            _type: NodeType::BreakStatement,
            label,
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
pub struct ContinueStatement {
    #[serde(rename(serialize = "type"))]
    pub _type: NodeType,
    pub start: usize,
    pub end: usize,
    pub loc: SourceLocation,
    pub label: Option<Identifier>,
}

impl ContinueStatement {
    pub fn new(label: Option<Identifier>, (start_loc, end_loc, source): AstNodePos) -> Self {
        Self {
            _type: NodeType::ContinueStatement,
            label,
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
pub struct IfStatement {
    #[serde(rename(serialize = "type"))]
    pub _type: NodeType,
    pub start: usize,
    pub end: usize,
    pub loc: SourceLocation,
    pub test: Expression,
    pub consequent: Box<Statement>,
    pub alternate: Option<Box<Statement>>,
}

impl IfStatement {
    pub fn new(
        test: Expression,
        consequent: Box<Statement>,
        alternate: Option<Box<Statement>>,
        (start_loc, end_loc, source): AstNodePos,
    ) -> Self {
        Self {
            _type: NodeType::IfStatement,
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
pub struct SwitchCase {
    #[serde(rename(serialize = "type"))]
    pub _type: NodeType,
    pub start: usize,
    pub end: usize,
    pub loc: SourceLocation,
    // test == None indicates it's a defualt clause
    pub test: Option<Expression>,
    pub consequent: Vec<Statement>,
}

impl SwitchCase {
    pub fn new(
        test: Option<Expression>,
        consequent: Vec<Statement>,
        (start_loc, end_loc, source): AstNodePos,
    ) -> Self {
        Self {
            _type: NodeType::SwitchCase,
            test,
            consequent,
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
pub struct SwitchStatement {
    #[serde(rename(serialize = "type"))]
    pub _type: NodeType,
    pub start: usize,
    pub end: usize,
    pub loc: SourceLocation,
    pub discriminant: Expression,
    pub cases: Vec<SwitchCase>,
}

impl SwitchStatement {
    pub fn new(
        discriminant: Expression,
        cases: Vec<SwitchCase>,
        (start_loc, end_loc, source): AstNodePos,
    ) -> Self {
        Self {
            _type: NodeType::SwitchStatement,
            discriminant,
            cases,
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
pub struct ThrowStatement {
    #[serde(rename(serialize = "type"))]
    pub _type: NodeType,
    pub start: usize,
    pub end: usize,
    pub loc: SourceLocation,
    pub argument: Expression,
}

impl ThrowStatement {
    pub fn new(argument: Expression, (start_loc, end_loc, source): AstNodePos) -> Self {
        Self {
            _type: NodeType::ThrowStatement,
            argument,
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

// from es10, catch clause's param could be null
#[derive(Serialize)]
pub struct CatchClause {
    #[serde(rename(serialize = "type"))]
    pub _type: NodeType,
    pub start: usize,
    pub end: usize,
    pub loc: SourceLocation,
    pub param: Option<Pattern>,
    pub body: BlockStatement,
}

impl CatchClause {
    pub fn new(
        param: Option<Pattern>,
        body: BlockStatement,
        (start_loc, end_loc, source): AstNodePos,
    ) -> Self {
        Self {
            _type: NodeType::CatchClause,
            param,
            body,
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
pub struct TryStatement {
    #[serde(rename(serialize = "type"))]
    pub _type: NodeType,
    pub start: usize,
    pub end: usize,
    pub loc: SourceLocation,
    pub block: BlockStatement,
    pub handler: Option<CatchClause>,
    pub finalizer: Option<BlockStatement>,
}

impl TryStatement {
    pub fn new(
        block: BlockStatement,
        handler: Option<CatchClause>,
        finalizer: Option<BlockStatement>,
        (start_loc, end_loc, source): AstNodePos,
    ) -> Self {
        Self {
            _type: NodeType::TryStatement,
            block,
            handler,
            finalizer,
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
pub struct WhileStatement {
    #[serde(rename(serialize = "type"))]
    pub _type: NodeType,
    pub start: usize,
    pub end: usize,
    pub loc: SourceLocation,
    pub test: Expression,
    pub body: Box<Statement>,
}

impl WhileStatement {
    pub fn new(
        test: Expression,
        body: Box<Statement>,
        (start_loc, end_loc, source): AstNodePos,
    ) -> Self {
        Self {
            _type: NodeType::WhileStatement,
            test,
            body,
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
pub struct DoWhileStatement {
    #[serde(rename(serialize = "type"))]
    pub _type: NodeType,
    pub start: usize,
    pub end: usize,
    pub loc: SourceLocation,
    pub body: Box<Statement>,
    pub test: Expression,
}

impl DoWhileStatement {
    pub fn new(
        body: Box<Statement>,
        test: Expression,
        (start_loc, end_loc, source): AstNodePos,
    ) -> Self {
        Self {
            _type: NodeType::DoWhileStatement,
            body,
            test,
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
pub enum FunctionBodyContent {
    Directive(Directive),
    Statement(Box<Statement>),
}

// Function body is the same with BlockStatement except it could contain Directive in the beginning.
#[derive(Serialize)]
pub struct FunctionBody {
    #[serde(rename(serialize = "type"))]
    pub _type: NodeType,
    pub start: usize,
    pub end: usize,
    pub loc: SourceLocation,
    pub body: Vec<FunctionBodyContent>,
}

impl FunctionBody {
    pub fn new(body: Vec<FunctionBodyContent>, (start_loc, end_loc, source): AstNodePos) -> Self {
        Self {
            _type: NodeType::BlockStatement,
            body,
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
pub struct FunctionDeclaration {
    #[serde(rename(serialize = "type"))]
    pub _type: NodeType,
    pub start: usize,
    pub end: usize,
    pub loc: SourceLocation,
    pub id: Identifier,
    pub params: Vec<Pattern>,
    pub body: FunctionBody,
    // starts from es6
    pub generator: bool,
    // starts from es8, async is a reserved word, so prefix it with is_
    pub is_async: bool,
}

impl FunctionDeclaration {
    pub fn new(
        id: Identifier,
        params: Vec<Pattern>,
        body: FunctionBody,
        is_generator: bool,
        is_async: bool,
        (start_loc, end_loc, source): AstNodePos,
    ) -> Self {
        Self {
            _type: NodeType::FunctionDeclaration,
            id,
            params,
            body,
            generator: is_generator,
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
pub struct VariableDeclarator {
    #[serde(rename(serialize = "type"))]
    pub _type: NodeType,
    pub start: usize,
    pub end: usize,
    pub loc: SourceLocation,
    pub id: Pattern,
    pub init: Option<Expression>,
}

impl VariableDeclarator {
    pub fn new(
        id: Pattern,
        init: Option<Expression>,
        (start_loc, end_loc, source): AstNodePos,
    ) -> Self {
        Self {
            _type: NodeType::VaraiableDeclarator,
            id,
            init,
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

#[derive(Clone, Serialize)]
pub enum VariableKind {
    Var,
    Let,
    Const,
}

#[derive(Serialize)]
pub struct VariableDeclaration {
    #[serde(rename(serialize = "type"))]
    pub _type: NodeType,
    pub start: usize,
    pub end: usize,
    pub loc: SourceLocation,
    pub declarations: Vec<VariableDeclarator>,
    pub kind: VariableKind,
}

impl VariableDeclaration {
    pub fn new(
        declarators: Vec<VariableDeclarator>,
        kind: VariableKind,
        (start_loc, end_loc, source): AstNodePos,
    ) -> Self {
        Self {
            _type: NodeType::VariableDeclaration,
            declarations: declarators,
            kind,
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
pub enum ForStatementInit {
    VariableDeclaration(VariableDeclaration),
    Expression(Expression),
    Null,
}

impl From<VariableDeclaration> for ForStatementInit {
    fn from(value: VariableDeclaration) -> Self {
        Self::VariableDeclaration(value)
    }
}

impl From<Expression> for ForStatementInit {
    fn from(value: Expression) -> Self {
        Self::Expression(value)
    }
}

#[derive(Serialize)]
pub struct ForStatement {
    #[serde(rename(serialize = "type"))]
    pub _type: NodeType,
    pub start: usize,
    pub end: usize,
    pub loc: SourceLocation,
    pub init: ForStatementInit,
    pub test: Option<Expression>,
    pub update: Option<Expression>,
    pub body: Box<Statement>,
}

impl ForStatement {
    pub fn new(
        init: ForStatementInit,
        test: Option<Expression>,
        update: Option<Expression>,
        body: Box<Statement>,
        (start_loc, end_loc, source): AstNodePos,
    ) -> Self {
        Self {
            _type: NodeType::ForStatement,
            init,
            test,
            update,
            body,
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
pub enum ForInOfStatementLeft {
    VariableDeclaration(VariableDeclaration),
    Pattern(Pattern),
}

impl From<VariableDeclaration> for ForInOfStatementLeft {
    fn from(value: VariableDeclaration) -> Self {
        Self::VariableDeclaration(value)
    }
}

impl From<Pattern> for ForInOfStatementLeft {
    fn from(value: Pattern) -> Self {
        Self::Pattern(value)
    }
}

#[derive(Serialize)]
pub struct ForInStatement {
    #[serde(rename(serialize = "type"))]
    pub _type: NodeType,
    pub start: usize,
    pub end: usize,
    pub loc: SourceLocation,
    pub left: ForInOfStatementLeft,
    pub right: Expression,
    pub body: Box<Statement>,
}

impl ForInStatement {
    pub fn new(
        left: ForInOfStatementLeft,
        right: Expression,
        body: Box<Statement>,
        (start_loc, end_loc, source): AstNodePos,
    ) -> Self {
        Self {
            _type: NodeType::ForInStatement,
            left,
            right,
            body,
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

// following definition all have to greater than es5
#[derive(Serialize)]
pub struct ForOfStatement {
    #[serde(rename(serialize = "type"))]
    pub _type: NodeType,
    pub start: usize,
    pub end: usize,
    pub loc: SourceLocation,
    pub left: ForInOfStatementLeft,
    pub right: Expression,
    pub body: Box<Statement>,
    // from es9
    pub is_await: bool,
}

impl ForOfStatement {
    pub fn new(
        left: ForInOfStatementLeft,
        right: Expression,
        body: Box<Statement>,
        is_await: bool,
        (start_loc, end_loc, source): AstNodePos,
    ) -> Self {
        Self {
            _type: NodeType::ForOfStatement,
            left,
            right,
            body,
            is_await,
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
pub enum ClassMethodKey {
    Expression(Expression),
    PrivateIdentifier(PrivateIdentifier),
}

// from es13, class body supports property definition and staticblock
#[derive(Serialize)]
pub enum ClassBodyEl {
    MethodDefinition(MethodDefinition),
    PropertyDefinition(PropertyDefinition),
    StaticBlock(StaticBlock),
}

#[derive(PartialEq, Eq, Serialize)]
pub enum MethodKind {
    Constructor,
    Method,
    Get,
    Set,
}

#[derive(Serialize)]
pub struct MethodDefinition {
    #[serde(rename(serialize = "type"))]
    pub _type: NodeType,
    pub start: usize,
    pub end: usize,
    pub loc: SourceLocation,
    pub key: ClassMethodKey,
    pub value: FunctionExpression,
    pub kind: MethodKind,
    pub computed: bool,
    // static is reserved word, use is_static here
    pub is_static: bool,
}

impl MethodDefinition {
    pub fn new(
        key: ClassMethodKey,
        value: FunctionExpression,
        kind: MethodKind,
        computed: bool,
        is_static: bool,
        (start_loc, end_loc, source_file): AstNodePos,
    ) -> Self {
        Self {
            _type: NodeType::MethodDefinition,
            key,
            value,
            kind,
            computed,
            is_static,
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
pub enum ClassPropertyKey {
    Expression(Expression),
    PrivateIdentifier(PrivateIdentifier),
}

// if key is PrivateIdentifier, computed must be false.
#[derive(Serialize)]
pub struct PropertyDefinition {
    #[serde(rename(serialize = "type"))]
    pub _type: NodeType,
    pub start: usize,
    pub end: usize,
    pub loc: SourceLocation,
    pub key: ClassPropertyKey,
    pub value: Option<Expression>,
    pub computed: bool,
    pub is_static: bool,
}

impl PropertyDefinition {
    pub fn new(
        key: ClassPropertyKey,
        value: Option<Expression>,
        computed: bool,
        is_static: bool,
        (start_loc, end_loc, source_file): AstNodePos,
    ) -> Self {
        Self {
            _type: NodeType::Property,
            key,
            value,
            computed,
            is_static,
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
pub struct ClassBody {
    #[serde(rename(serialize = "type"))]
    pub _type: NodeType,
    pub start: usize,
    pub end: usize,
    pub loc: SourceLocation,
    pub body: Vec<ClassBodyEl>,
}

impl ClassBody {
    pub fn new(body: Vec<ClassBodyEl>, (start_loc, end_loc, source_file): AstNodePos) -> Self {
        Self {
            _type: NodeType::ClassBody,
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

#[derive(Serialize)]
pub struct ClassDeclaration {
    #[serde(rename(serialize = "type"))]
    pub _type: NodeType,
    pub start: usize,
    pub end: usize,
    pub loc: SourceLocation,
    pub id: Identifier,
    pub super_class: Option<Expression>,
    pub body: ClassBody,
}

impl ClassDeclaration {
    pub fn new(
        id: Identifier,
        super_class: Option<Expression>,
        body: ClassBody,
        (start_loc, end_loc, source_file): AstNodePos,
    ) -> Self {
        Self {
            _type: NodeType::ClassDeclaration,
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

// pub struct Class {
//     #[serde(rename(serialize = "type"))]
//     pub _type: NodeType,
//     pub start: usize,
//     pub end: usize,
//     pub loc: SourceLocation,
//     pub id: Option<Identifier>,
//     pub super_class: Option<Expression>,
//     pub body: ClassBody,
// }

pub enum FunctionDeclarationType {
    FunctionDeclaration(FunctionDeclaration),
    AnonymousDefaultExportedFunctionDeclaration(AnonymousDefaultExportedFunctionDeclaration),
}

impl From<FunctionDeclarationType> for Statement {
    fn from(value: FunctionDeclarationType) -> Self {
        match value {
            FunctionDeclarationType::FunctionDeclaration(v) => v.into(),
            FunctionDeclarationType::AnonymousDefaultExportedFunctionDeclaration(v) => v.into(),
        }
    }
}

impl From<FunctionDeclaration> for FunctionDeclarationType {
    fn from(value: FunctionDeclaration) -> Self {
        Self::FunctionDeclaration(value)
    }
}

pub enum ClassDeclarationType {
    AnonymousDefaultExportedClassDeclaration(AnonymousDefaultExportedClassDeclaration),
    ClassDeclaration(ClassDeclaration),
}

impl From<ClassDeclaration> for ClassDeclarationType {
    fn from(value: ClassDeclaration) -> Self {
        Self::ClassDeclaration(value)
    }
}

impl From<ClassDeclarationType> for Statement {
    fn from(value: ClassDeclarationType) -> Self {
        match value {
            ClassDeclarationType::AnonymousDefaultExportedClassDeclaration(v) => {
                Statement::AnonymousDefaultExportedClassDeclaration(v)
            }
            ClassDeclarationType::ClassDeclaration(v) => Statement::ClassDeclaration(v),
        }
    }
}
