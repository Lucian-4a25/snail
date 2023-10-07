use serde::Serialize;

use super::{
    expression::{Expression, Identifier, Literal},
    pattern::Pattern,
    statement::{
        ClassBody, ClassDeclaration, ClassDeclarationType, FunctionBody, FunctionDeclaration,
        FunctionDeclarationType, VariableDeclaration,
    },
    AstNodePos, NodeType, ProgramNode, SourceLocation,
};

#[derive(Serialize)]
pub enum ImportOrExportDeclaration {
    ImportDeclaration(ImportDeclaration),
    ExportNamedDeclaration(ExportNamedDeclaration),
    ExportDefaultDeclaration(ExportDefaultDeclaration), // export default 1 or export default function() {}
    ExportAllDeclaration(ExportAllDeclaration),         // export * from 'mod'
}

impl From<ImportOrExportDeclaration> for ProgramNode {
    fn from(value: ImportOrExportDeclaration) -> Self {
        Self::ImportOrExportDeclaration(value)
    }
}

impl From<ImportDeclaration> for ImportOrExportDeclaration {
    fn from(value: ImportDeclaration) -> Self {
        Self::ImportDeclaration(value)
    }
}

impl From<ExportNamedDeclaration> for ImportOrExportDeclaration {
    fn from(value: ExportNamedDeclaration) -> Self {
        Self::ExportNamedDeclaration(value)
    }
}

impl From<ExportDefaultDeclaration> for ImportOrExportDeclaration {
    fn from(value: ExportDefaultDeclaration) -> Self {
        Self::ExportDefaultDeclaration(value)
    }
}

impl From<ExportAllDeclaration> for ImportOrExportDeclaration {
    fn from(value: ExportAllDeclaration) -> Self {
        Self::ExportAllDeclaration(value)
    }
}

#[derive(Serialize)]
pub enum ImportSpecifiers {
    ImportSpecifier(ImportSpecifier),
    ImportDefaultSpecifier(ImportDefaultSpecifier),
    ImportNamespaceSpecifier(ImportNamespaceSpecifier),
}

// from es13, support imported's value could be literal without lone surrogate
// see: https://github.com/tc39/ecma262/pull/2154
#[derive(Serialize)]
pub enum ImportedType {
    Identifier(Identifier),
    Literal(Literal),
}

// for case like: import {foo} from "mod" or {foo as bar} in import {foo as bar} from "mod"
#[derive(Serialize)]
pub struct ImportSpecifier {
    #[serde(rename(serialize = "type"))]
    pub _type: NodeType,
    pub start: usize,
    pub end: usize,
    pub loc: SourceLocation,
    pub imported: ImportedType,
    pub local: Identifier,
}

impl ImportSpecifier {
    pub fn new(
        imported: ImportedType,
        local: Identifier,
        (start_loc, end_loc, source_file): AstNodePos,
    ) -> Self {
        Self {
            _type: NodeType::ImportSpecifier,
            imported,
            local,
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

// for case: import foo from "mod.js".
#[derive(Serialize)]
pub struct ImportDefaultSpecifier {
    #[serde(rename(serialize = "type"))]
    pub _type: NodeType,
    pub start: usize,
    pub end: usize,
    pub loc: SourceLocation,
    pub local: Identifier,
}

impl ImportDefaultSpecifier {
    pub fn new(local: Identifier, (start_loc, end_loc, source_file): AstNodePos) -> Self {
        Self {
            _type: NodeType::ImportDefaultSpecifier,
            local,
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

// for case: import * as foo from "mod.js"
#[derive(Serialize)]
pub struct ImportNamespaceSpecifier {
    #[serde(rename(serialize = "type"))]
    pub _type: NodeType,
    pub start: usize,
    pub end: usize,
    pub loc: SourceLocation,
    pub local: Identifier,
}

impl ImportNamespaceSpecifier {
    pub fn new(local: Identifier, (start_loc, end_loc, source_file): AstNodePos) -> Self {
        Self {
            _type: NodeType::ImportNamespaceSpecifier,
            local,
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
pub struct ImportDeclaration {
    #[serde(rename(serialize = "type"))]
    pub _type: NodeType,
    pub start: usize,
    pub end: usize,
    pub loc: SourceLocation,
    pub specifiers: Vec<ImportSpecifiers>,
    pub source: Literal,
}

impl ImportDeclaration {
    pub fn new(
        source: Literal,
        specifiers: Vec<ImportSpecifiers>,
        (start_loc, end_loc, source_file): AstNodePos,
    ) -> Self {
        Self {
            _type: NodeType::ImportDeclaration,
            source,
            specifiers,
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
pub enum ExportDeclaration {
    FunctionDeclaration(FunctionDeclaration),
    VariableDeclaration(VariableDeclaration),
    ClassDeclaration(ClassDeclaration),
}

// if the type of local value is literal, then the type of source must be Some.
#[derive(Serialize)]
pub enum ExportLocal {
    Identifier(Identifier),
    Literal(Literal), // without lone surrogate
}

#[derive(Serialize)]
pub enum ExportedType {
    Identifier(Identifier),
    Literal(Literal), // without lone surrogate
}

#[derive(Serialize)]
pub struct ExportSpecifier {
    #[serde(rename(serialize = "type"))]
    pub _type: NodeType,
    pub start: usize,
    pub end: usize,
    pub loc: SourceLocation,
    pub local: ExportLocal,
    pub exported: ExportedType,
}

impl ExportSpecifier {
    pub fn new(
        local: ExportLocal,
        exported: ExportedType,
        (start_loc, end_loc, file_source): AstNodePos,
    ) -> Self {
        Self {
            _type: NodeType::ExportSpecifier,
            local,
            exported,
            start: start_loc.pos,
            end: end_loc.pos,
            loc: SourceLocation {
                source: file_source,
                start: start_loc.loc,
                end: end_loc.loc,
            },
        }
    }
}

// When declaration is Some, for case like: export var foo = 1.
// And there can not be specifiers or source.
#[derive(Serialize)]
pub struct ExportNamedDeclaration {
    #[serde(rename(serialize = "type"))]
    pub _type: NodeType,
    pub start: usize,
    pub end: usize,
    pub loc: SourceLocation,
    pub declaration: Option<ExportDeclaration>,
    pub specifiers: Option<Vec<ExportSpecifier>>,
    pub source: Option<Literal>,
}

impl ExportNamedDeclaration {
    pub fn new(
        declaration: Option<ExportDeclaration>,
        specifiers: Option<Vec<ExportSpecifier>>,
        source: Option<Literal>,
        (start_loc, end_loc, file_source): AstNodePos,
    ) -> Self {
        Self {
            _type: NodeType::ExportNamedDeclaration,
            declaration,
            specifiers,
            source,
            start: start_loc.pos,
            end: end_loc.pos,
            loc: SourceLocation {
                source: file_source,
                start: start_loc.loc,
                end: end_loc.loc,
            },
        }
    }
}

// Note: Although anonymouse decalaration seems like a expression,
// here still need to use same type value with normal decalaration
#[derive(Serialize)]
pub enum ExportDefaultDeclarationType {
    AnonymousDefaultExportedFunctionDeclaration(AnonymousDefaultExportedFunctionDeclaration),
    FunctionDeclaration(FunctionDeclaration),
    AnonymousDefaultExportedClassDeclaration(AnonymousDefaultExportedClassDeclaration),
    ClassDeclaration(ClassDeclaration),
    Expression(Expression),
}

#[derive(Serialize)]
pub struct AnonymousDefaultExportedFunctionDeclaration {
    #[serde(rename(serialize = "type"))]
    pub _type: NodeType, // "FunctionDeclaration"
    pub start: usize,
    pub end: usize,
    pub loc: SourceLocation,
    // pub id: None; // the value is constant, ignore it directly
    pub params: Vec<Pattern>,
    pub body: FunctionBody,
    pub generator: bool,
    // from es8
    pub is_async: bool,
}

impl AnonymousDefaultExportedFunctionDeclaration {
    pub fn new(
        params: Vec<Pattern>,
        body: FunctionBody,
        is_generator: bool,
        is_async: bool,
        (start_loc, end_loc, source): AstNodePos,
    ) -> Self {
        Self {
            _type: NodeType::FunctionDeclaration,
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

impl From<AnonymousDefaultExportedFunctionDeclaration> for FunctionDeclarationType {
    fn from(value: AnonymousDefaultExportedFunctionDeclaration) -> Self {
        Self::AnonymousDefaultExportedFunctionDeclaration(value)
    }
}

#[derive(Serialize)]
pub struct AnonymousDefaultExportedClassDeclaration {
    #[serde(rename(serialize = "type"))]
    pub _type: NodeType,
    pub start: usize,
    pub end: usize,
    pub loc: SourceLocation,
    // pub id: Identifier, // the value is constant, ignore it directly
    pub super_class: Option<Expression>,
    pub body: ClassBody,
}

impl AnonymousDefaultExportedClassDeclaration {
    pub fn new(
        super_class: Option<Expression>,
        body: ClassBody,
        (start_loc, end_loc, source_file): AstNodePos,
    ) -> Self {
        Self {
            _type: NodeType::ClassDeclaration,
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

impl From<AnonymousDefaultExportedClassDeclaration> for ClassDeclarationType {
    fn from(value: AnonymousDefaultExportedClassDeclaration) -> Self {
        Self::AnonymousDefaultExportedClassDeclaration(value)
    }
}

// e.g., export default function () {}; or export default 1;.
#[derive(Serialize)]
pub struct ExportDefaultDeclaration {
    #[serde(rename(serialize = "type"))]
    pub _type: NodeType,
    pub start: usize,
    pub end: usize,
    pub loc: SourceLocation,
    pub declaration: ExportDefaultDeclarationType,
}

impl ExportDefaultDeclaration {
    pub fn new(
        declaration: ExportDefaultDeclarationType,
        (start_loc, end_loc, source_file): AstNodePos,
    ) -> Self {
        Self {
            _type: NodeType::ExportDefaultDeclaration,
            declaration,
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

// from es11, eg. export * as foo from "mod";
// from es13, support literal without lone surrogate
#[derive(Serialize)]
pub enum ExportAllExportedType {
    Identifier(Identifier),
    Literal(Literal),
    Null,
}

// e.g., export * from "mod";.
#[derive(Serialize)]
pub struct ExportAllDeclaration {
    #[serde(rename(serialize = "type"))]
    pub _type: NodeType,
    pub start: usize,
    pub end: usize,
    pub loc: SourceLocation,
    pub source: Literal,
    pub exported: ExportAllExportedType,
}

impl ExportAllDeclaration {
    pub fn new(
        exported: ExportAllExportedType,
        source: Literal,
        (start_loc, end_loc, source_file): AstNodePos,
    ) -> Self {
        Self {
            _type: NodeType::ExportAllDeclaration,
            exported,
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
