pub mod expression;
pub mod lval;
pub mod scope;
pub mod util;
use self::expression::assignment::parse_maybe_assign;
use self::expression::function::parse_function_body;
use self::expression::literal::parse_literal;
use self::expression::object::parse_method;
use self::expression::subscript::parse_expr_subscripts;
use self::expression::{
    parse_expression, parse_ident, parse_paren_expression, parse_private_ident,
};
use self::lval::{parse_binding_atom, parse_binding_list};
use self::scope::{get_func_flags, SCOPE_CLASS_STATIC_BLOCK, SCOPE_SIMPLE_CATCH, SCOPE_SUPER};
use self::util::{
    after_trailing_comma, can_insert_semicolon, check_label_destination,
    check_private_name_conflicts, check_used_private_name, is_async_func, is_directive_candidate,
    is_import_expr, is_let, unexpected,
};
use crate::ast::directive::Directive;
use crate::ast::expression::{
    Expression, Identifier, Literal, LiteralValue, PrivateIdentifier, StaticBlock,
};
use crate::ast::import_export_declaration::{
    AnonymousDefaultExportedClassDeclaration, AnonymousDefaultExportedFunctionDeclaration,
    ExportAllDeclaration, ExportAllExportedType, ExportDeclaration, ExportDefaultDeclaration,
    ExportDefaultDeclarationType, ExportLocal, ExportNamedDeclaration, ExportSpecifier,
    ExportedType, ImportDeclaration, ImportDefaultSpecifier, ImportNamespaceSpecifier,
    ImportOrExportDeclaration, ImportSpecifier, ImportSpecifiers, ImportedType,
};
use crate::ast::pattern::Pattern;
use crate::ast::statement::{
    BlockStatement, BreakStatement, CatchClause, ClassBody, ClassBodyEl, ClassDeclaration,
    ClassDeclarationType, ClassMethodKey, ClassPropertyKey, ContinueStatement, DebuggerStatement,
    DoWhileStatement, EmptyStatement, ExpressionStatement, ForInOfStatementLeft, ForInStatement,
    ForOfStatement, ForStatement, ForStatementInit, FunctionDeclaration, FunctionDeclarationType,
    IfStatement, LabeledStatement, MethodDefinition, MethodKind, PropertyDefinition,
    ReturnStatement, Statement, SwitchCase, SwitchStatement, ThrowStatement, TryStatement,
    VariableDeclaration, VariableDeclarator, VariableKind, WhileStatement, WithStatement,
};
use crate::ast::{Program, _LocationNode, create_program_node};
use crate::parser::{AccessorKind, ForInitType, Label, LabelKind, Parser, StatementContext};
use crate::tokenizer::js_token::TokenLabel;
use crate::tokenizer::util::has_break_in_range;
use std::vec;

pub fn parse_top_level(ctx: &mut Parser) -> Program {
    let mut root_node = create_program_node();
    let mut maybe_directive = true;

    ctx.next_unwrap();

    loop {
        if ctx.cur_token.as_ref().map_or(false, |t| t.is_eof()) {
            break;
        }
        let start_token = ctx.cur_token.as_ref().unwrap();
        match start_token.label {
            TokenLabel::_Import => {
                // TODO: check ecma version >= 10
                if !is_import_expr(ctx) {
                    root_node.body.push(parse_statement(ctx).into());
                    continue;
                }
                // TODO: check if it's in a module
                root_node
                    .body
                    .push(ImportOrExportDeclaration::from(parse_import(ctx)).into());
            }
            TokenLabel::_Export => {
                // TODO: check if it's in a module
                root_node.body.push(parse_export(ctx).into());
            }
            _ => {
                let stmt = parse_statement(ctx);
                if !maybe_directive {
                    root_node.body.push(stmt.into());
                } else if is_directive_candidate(&stmt) {
                    if let Statement::ExpressionStatement(ExpressionStatement {
                        expression: Expression::Literal(literal),
                        ..
                    }) = stmt
                    {
                        root_node.body.push(Directive::from(literal).into());
                    }
                } else {
                    maybe_directive = false;
                    root_node.body.push(stmt.into());
                }
            }
        }
    }

    root_node.end = ctx.cursor;
    root_node.loc.end = ctx.get_cursor_position();

    root_node
}

pub fn parse_statement(ctx: &mut Parser) -> Statement {
    if is_let(ctx) {
        return parse_var_stmt(ctx, VariableKind::Let).into();
    }

    let start_type = ctx.cur_token.as_ref().unwrap();
    match start_type.label {
        TokenLabel::_Break | TokenLabel::_Continue => parse_break_continue(ctx),
        TokenLabel::_Debugger => parse_debugger(ctx).into(),
        TokenLabel::_Do => parse_do_loop(ctx).into(),
        TokenLabel::_For => parse_for_loop(ctx),
        TokenLabel::_Function => parse_function_stmt(ctx, false, false).into(),
        TokenLabel::_Class => {
            if !ctx.cur_stmt_ctx_is(StatementContext::TopLevel) {
                unexpected(ctx.cur_token.clone().unwrap());
            }
            parse_class(ctx, false).into()
        }
        TokenLabel::_If => parse_if(ctx).into(),
        TokenLabel::_Return => parse_return(ctx).into(),
        TokenLabel::_Switch => parse_switch(ctx).into(),
        TokenLabel::_Throw => parse_throw(ctx).into(),
        TokenLabel::_Try => parse_try(ctx).into(),
        TokenLabel::_Const | TokenLabel::_Var => parse_var_stmt(
            ctx,
            if ctx.cur_token_is(TokenLabel::_Const) {
                VariableKind::Const
            } else {
                VariableKind::Var
            },
        )
        .into(),
        TokenLabel::_While => parse_while(ctx).into(),
        TokenLabel::_With => parse_with(ctx).into(),
        TokenLabel::BraceL => parse_block_stmt(ctx, true).into(),
        TokenLabel::Semi => parse_empty(ctx).into(),
        // import alse could be expression
        TokenLabel::_Import => {
            let start_loc = ctx.start_location_node();
            // TODO: add parse import expression function, didn't need to use the general parse_expression function
            ctx.semicolon();
            ExpressionStatement::new(parse_expression(ctx), ctx.compose_loc_info(start_loc)).into()
        }
        _ => {
            let start_loc = ctx.start_location_node();
            if is_async_func(ctx) {
                return parse_function_stmt(ctx, false, true).into();
            }
            let expr = parse_expression(ctx);
            if ctx.cur_token_is(TokenLabel::Colon) {
                if let Expression::Identifier(ident) = expr {
                    ctx.next_unwrap();
                    return parse_labeled_stmt(ctx, start_loc, ident).into();
                }
            }

            ctx.semicolon();
            ExpressionStatement::new(expr, ctx.compose_loc_info(start_loc)).into()
        }
    }
}

pub fn parse_labeled_stmt(
    ctx: &mut Parser,
    start_loc: _LocationNode,
    label: Identifier,
) -> LabeledStatement {
    for l in ctx.labels.iter() {
        if l.name.as_ref().map_or(false, |n| n == &label.name) {
            panic!("Label '{}' is already declared", label.name);
        }
    }
    let label_kind = if ctx.cur_token_test(|t| t.is_loop) {
        LabelKind::Loop
    } else if ctx.cur_token_is(TokenLabel::_Switch) {
        LabelKind::Switch
    } else {
        LabelKind::Custom
    };

    ctx.labels.push(Label {
        kind: label_kind,
        name: Some(label.name.clone()),
    });

    ctx.enter_stmt_ctx(StatementContext::LabelStmt);
    let body = parse_statement(ctx);
    ctx.exit_stmt_ctx();

    LabeledStatement::new(label, Box::new(body), ctx.compose_loc_info(start_loc))
}

// TODO: check if exported thing exists.
pub fn parse_export(ctx: &mut Parser) -> ImportOrExportDeclaration {
    let start_loc = ctx.start_location_node();
    ctx.next_unwrap();
    // export * [as ident] from ''
    if ctx.cur_token_is(TokenLabel::Star) {
        // TODO: check ecma >= 11
        let exported = if ctx.eat_contextual("as") {
            match parse_import_export_name(ctx) {
                ImportExportName::Identifier(ident) => ExportAllExportedType::Identifier(ident),
                ImportExportName::Literal(literal) => ExportAllExportedType::Literal(literal),
            }
        } else {
            ExportAllExportedType::Null
        };
        ctx.expect_contexual("from");
        if !ctx.cur_token_is(TokenLabel::String) {
            unexpected(ctx.cur_token.clone().unwrap());
        }
        let source = parse_literal(ctx);
        ctx.semicolon();
        return ExportAllDeclaration::new(exported, source, ctx.compose_loc_info(start_loc)).into();
    }

    // export default [function declaration|class declaration|expression]
    if ctx.eat(TokenLabel::_Default) {
        return parse_export_default(ctx, start_loc).into();
    }

    // parse may variable declaration, export var|let|const|[async] function|class ..
    if ctx.cur_token_test(|t| {
        let label = t.label;
        label == TokenLabel::_Var
            || label == TokenLabel::_Const
            || label == TokenLabel::_Function
            || label == TokenLabel::_Class
    }) || is_let(ctx)
        || is_async_func(ctx)
    {
        let stmt_declaration = parse_statement(ctx);
        let declaration = match stmt_declaration {
            Statement::FunctionDeclaration(func_decl) => {
                ExportDeclaration::FunctionDeclaration(func_decl)
            }
            Statement::ClassDeclaration(class_decl) => {
                ExportDeclaration::ClassDeclaration(class_decl)
            }
            Statement::VariableDeclaration(var_decl) => {
                ExportDeclaration::VariableDeclaration(var_decl)
            }
            _ => {
                unexpected(ctx.cur_token.clone().unwrap());
            }
        };
        return ExportNamedDeclaration::new(
            Some(declaration),
            None,
            None,
            ctx.compose_loc_info(start_loc),
        )
        .into();
    }

    // parse named exported, { x, y as z } [from '...']
    ctx.expect(TokenLabel::BraceL);
    let mut specifiers = vec![];
    let mut first = true;
    while !ctx.eat(TokenLabel::BraceR) {
        if !first {
            ctx.expect(TokenLabel::Comma);
            if after_trailing_comma(ctx, TokenLabel::BraceR, true) {
                break;
            }
        } else {
            first = false;
        }

        let specifier_start_loc = ctx.start_location_node();
        let local = parse_import_export_name(ctx);
        let exported = if ctx.eat_contextual("as") {
            parse_import_export_name(ctx)
        } else {
            local.clone()
        };
        specifiers.push(ExportSpecifier::new(
            local.into(),
            exported.into(),
            ctx.compose_loc_info(specifier_start_loc),
        ));
    }

    let source = if ctx.eat_contextual("from") {
        if !ctx.cur_token_is(TokenLabel::String) {
            unexpected(ctx.cur_token.clone().unwrap());
        }
        Some(parse_literal(ctx))
    } else {
        // check if there is local literal in export specifiers
        None
    };

    ExportNamedDeclaration::new(
        None,
        Some(specifiers),
        source,
        ctx.compose_loc_info(start_loc),
    )
    .into()
}

pub fn parse_export_default(
    ctx: &mut Parser,
    start_loc: _LocationNode,
) -> ExportDefaultDeclaration {
    let is_async = is_async_func(ctx);
    if is_async || ctx.cur_token_is(TokenLabel::_Function) {
        let func_declaration = parse_function_stmt(ctx, true, is_async);
        let declaration = match func_declaration {
            FunctionDeclarationType::AnonymousDefaultExportedFunctionDeclaration(anony_func) => {
                ExportDefaultDeclarationType::AnonymousDefaultExportedFunctionDeclaration(
                    anony_func,
                )
            }
            FunctionDeclarationType::FunctionDeclaration(func) => {
                ExportDefaultDeclarationType::FunctionDeclaration(func)
            }
        };
        return ExportDefaultDeclaration::new(declaration, ctx.compose_loc_info(start_loc)).into();
    }

    if ctx.cur_token_is(TokenLabel::_Class) {
        let declaration = match parse_class(ctx, true) {
            ClassDeclarationType::AnonymousDefaultExportedClassDeclaration(anony_class) => {
                ExportDefaultDeclarationType::AnonymousDefaultExportedClassDeclaration(anony_class)
            }
            ClassDeclarationType::ClassDeclaration(decl_class) => {
                ExportDefaultDeclarationType::ClassDeclaration(decl_class)
            }
        };
        return ExportDefaultDeclaration::new(declaration, ctx.compose_loc_info(start_loc)).into();
    }

    let declaration = ExportDefaultDeclarationType::Expression(parse_maybe_assign(ctx));
    ctx.semicolon();

    ExportDefaultDeclaration::new(declaration, ctx.compose_loc_info(start_loc))
}

pub fn parse_import(ctx: &mut Parser) -> ImportDeclaration {
    let start_loc = ctx.start_location_node();
    ctx.next_unwrap();

    if ctx.cur_token_is(TokenLabel::String) {
        let source = parse_literal(ctx);
        return ImportDeclaration::new(source, vec![], ctx.compose_loc_info(start_loc));
    }

    let specifiers = parse_import_specifers(ctx);
    ctx.expect_contexual("from");
    if !ctx.cur_token_is(TokenLabel::String) {
        unexpected(ctx.cur_token.clone().unwrap());
    }
    let source = parse_literal(ctx);

    ImportDeclaration::new(source, specifiers, ctx.compose_loc_info(start_loc))
}

// #16.2.2 Imports
// ImportClause :
// ImportedDefaultBinding such as: import a from '..'
// NameSpaceImport      such as: * as ImportedBinding
// NamedImports     such as: { a as b, c, }
// ImportedDefaultBinding , NameSpaceImport
// ImportedDefaultBinding , NamedImports
pub fn parse_import_specifers(ctx: &mut Parser) -> Vec<ImportSpecifiers> {
    let mut specifiers = vec![];
    if ctx.cur_token_is(TokenLabel::Name) {
        let default_start_loc = ctx.start_location_node();
        let local = parse_ident(ctx, true);
        // TODO: check if the value is valid
        specifiers.push(ImportSpecifiers::ImportDefaultSpecifier(
            ImportDefaultSpecifier::new(local, ctx.compose_loc_info(default_start_loc)),
        ));
        if !ctx.eat(TokenLabel::Comma) {
            return specifiers;
        }
    }
    // check if there is NameSpaceImport
    if ctx.cur_token_is(TokenLabel::Star) {
        let space_start_loc = ctx.start_location_node();
        ctx.next_unwrap();
        ctx.expect_contexual("as");
        let local = parse_ident(ctx, true);
        // TODO: check if the value is valid
        specifiers.push(ImportSpecifiers::ImportNamespaceSpecifier(
            ImportNamespaceSpecifier::new(local, ctx.compose_loc_info(space_start_loc)),
        ));
        return specifiers;
    }

    ctx.expect(TokenLabel::BraceL);
    let mut first = true;
    while !ctx.eat(TokenLabel::BraceR) {
        if !first {
            ctx.expect(TokenLabel::Comma);
            if after_trailing_comma(ctx, TokenLabel::BraceR, true) {
                break;
            }
        } else {
            first = false;
        }

        let named_specifier_start = ctx.start_location_node();
        let imported = parse_import_export_name(ctx);
        let local = if ctx.eat_contextual("as") {
            parse_ident(ctx, true)
        } else {
            match &imported {
                ImportExportName::Identifier(ident) => ident.clone(),
                ImportExportName::Literal(_) => unexpected(ctx.cur_token.clone().unwrap()),
            }
        };
        // TODO: check left value
        specifiers.push(ImportSpecifiers::ImportSpecifier(ImportSpecifier::new(
            imported.into(),
            local,
            ctx.compose_loc_info(named_specifier_start),
        )));
    }

    specifiers
}

#[derive(Clone)]
pub enum ImportExportName {
    Identifier(Identifier),
    Literal(Literal),
}

impl From<ImportExportName> for ExportLocal {
    fn from(value: ImportExportName) -> Self {
        match value {
            ImportExportName::Identifier(ident) => Self::Identifier(ident),
            ImportExportName::Literal(literal) => Self::Literal(literal),
        }
    }
}

impl From<ImportExportName> for ExportedType {
    fn from(value: ImportExportName) -> Self {
        match value {
            ImportExportName::Identifier(ident) => Self::Identifier(ident),
            ImportExportName::Literal(literal) => Self::Literal(literal),
        }
    }
}

impl From<ImportExportName> for ImportedType {
    fn from(value: ImportExportName) -> Self {
        match value {
            ImportExportName::Identifier(ident) => Self::Identifier(ident),
            ImportExportName::Literal(literal) => Self::Literal(literal),
        }
    }
}

pub fn parse_import_export_name(ctx: &mut Parser) -> ImportExportName {
    // TODO: check if ecam >= 13
    if ctx.cur_token_is(TokenLabel::String) {
        let name = parse_literal(ctx);
        // TODO: check if contains lone surrogate.
        return ImportExportName::Literal(name);
    }

    return ImportExportName::Identifier(parse_ident(ctx, true));
}

pub fn parse_empty(ctx: &mut Parser) -> EmptyStatement {
    let start_loc = ctx.start_location_node();
    ctx.next_unwrap();

    EmptyStatement::new(ctx.compose_loc_info(start_loc))
}

pub fn parse_with(ctx: &mut Parser) -> WithStatement {
    let start_loc = ctx.start_location_node();
    if ctx.strict_mode {
        panic!("'with' in strict mode");
    }
    ctx.next_unwrap();
    let object = parse_paren_expression(ctx);
    ctx.stmt_context.push(StatementContext::With);
    let body = parse_statement(ctx);
    ctx.stmt_context.pop();

    WithStatement::new(object, Box::new(body), ctx.compose_loc_info(start_loc))
}

pub fn parse_while(ctx: &mut Parser) -> WhileStatement {
    let start_loc = ctx.start_location_node();
    ctx.next_unwrap();
    let test = parse_paren_expression(ctx);
    ctx.labels.push(Label {
        kind: LabelKind::Loop,
        name: None,
    });
    ctx.stmt_context.push(StatementContext::While);
    let body = parse_statement(ctx);
    ctx.stmt_context.pop();
    ctx.labels.pop();

    WhileStatement::new(test, Box::new(body), ctx.compose_loc_info(start_loc))
}

pub fn parse_var_stmt(ctx: &mut Parser, kind: VariableKind) -> VariableDeclaration {
    let start_loc = ctx.start_location_node();
    ctx.next_unwrap();
    let declarators = parse_var_declarator(ctx, kind.clone());
    ctx.semicolon();

    VariableDeclaration::new(declarators, kind, ctx.compose_loc_info(start_loc))
}

pub fn parse_try(ctx: &mut Parser) -> TryStatement {
    let start_loc = ctx.start_location_node();
    ctx.next_unwrap();
    let block = parse_block_stmt(ctx, true);
    let mut is_simple_catch = false;
    let handler = if ctx.cur_token_is(TokenLabel::_Catch) {
        let handler_start = ctx.start_location_node();
        ctx.next_unwrap();
        let mut param = None;
        if ctx.eat(TokenLabel::ParenL) {
            param = Some(parse_binding_atom(ctx).into());
            if matches!(param, Some(Pattern::Identifier(..))) {
                is_simple_catch = true;
            }
            // TODO: check left hand pattern
            ctx.expect(TokenLabel::ParenR);
        }
        ctx.enter_scope(if is_simple_catch {
            SCOPE_SIMPLE_CATCH
        } else {
            0
        });
        let handler_body = parse_block_stmt(ctx, false);
        ctx.exit_scope();

        Some(CatchClause::new(
            param,
            handler_body,
            ctx.compose_loc_info(handler_start),
        ))
    } else {
        None
    };

    let finalizer = if ctx.eat(TokenLabel::_Finally) {
        Some(parse_block_stmt(ctx, true))
    } else {
        None
    };

    if handler.is_none() && finalizer.is_none() {
        panic!("Missing catch or finally clause");
    }

    TryStatement::new(block, handler, finalizer, ctx.compose_loc_info(start_loc))
}

pub fn parse_throw(ctx: &mut Parser) -> ThrowStatement {
    let start_loc = ctx.start_location_node();
    ctx.next_unwrap();
    if has_break_in_range(ctx, (ctx.last_token_end, ctx.cur_token_start)) {
        panic!("Illegal newline after throw");
    }
    let argument = parse_expression(ctx);
    ctx.semicolon();

    ThrowStatement::new(argument, ctx.compose_loc_info(start_loc))
}

pub fn parse_switch(ctx: &mut Parser) -> SwitchStatement {
    let start_loc = ctx.start_location_node();
    ctx.next_unwrap();
    let discriminant = parse_paren_expression(ctx);
    let mut cases: Vec<SwitchCase> = vec![];
    ctx.labels.push(Label {
        kind: LabelKind::Switch,
        name: None,
    });
    ctx.expect(TokenLabel::BraceL);
    ctx.enter_scope(0);

    let mut saw_default = false;
    while !ctx.eat(TokenLabel::BraceR) {
        if !ctx.cur_token_test(|t| t.label == TokenLabel::_Case || t.label == TokenLabel::_Default)
        {
            unexpected(ctx.cur_token.clone().unwrap());
        }
        let case_start = ctx.start_location_node();
        let is_case_clause = ctx.cur_token_is(TokenLabel::_Case);
        let mut consequent = vec![];
        ctx.next_unwrap();
        let test = if is_case_clause {
            Some(parse_expression(ctx))
        } else {
            if !saw_default {
                saw_default = true;
            } else {
                println!("Multiple default clauses");
            }
            None
        };
        ctx.expect(TokenLabel::Colon);
        while !ctx.cur_token_test(|t| {
            t.label == TokenLabel::_Case
                || t.label == TokenLabel::_Default
                || t.label == TokenLabel::BraceR
        }) {
            // TODO: may need to reset statment context
            consequent.push(parse_statement(ctx));
        }
        cases.push(SwitchCase::new(
            test,
            consequent,
            ctx.compose_loc_info(case_start),
        ));
    }

    ctx.exit_scope();
    ctx.labels.pop();

    SwitchStatement::new(discriminant, cases, ctx.compose_loc_info(start_loc))
}

pub fn parse_return(ctx: &mut Parser) -> ReturnStatement {
    if !ctx.in_function_scope() {
        panic!("'return' outside of function");
    }
    let start_loc = ctx.start_location_node();
    ctx.next_unwrap();

    let argument = if ctx.eat(TokenLabel::Semi) || can_insert_semicolon(ctx) {
        None
    } else {
        let v = Some(parse_expression(ctx));
        ctx.semicolon();
        v
    };

    ReturnStatement::new(argument, ctx.compose_loc_info(start_loc))
}

pub fn parse_if(ctx: &mut Parser) -> IfStatement {
    let start_loc = ctx.start_location_node();
    ctx.next_unwrap();
    let test = parse_paren_expression(ctx);
    ctx.stmt_context.push(StatementContext::If);
    let consequent = parse_statement(ctx);
    let alternate = if ctx.eat(TokenLabel::_Else) {
        Some(parse_statement(ctx))
    } else {
        None
    };
    ctx.stmt_context.pop();

    IfStatement::new(
        test,
        Box::new(consequent),
        alternate.map(|a| Box::new(a)),
        ctx.compose_loc_info(start_loc),
    )
}

pub fn parse_class(ctx: &mut Parser, nullable_id: bool) -> ClassDeclarationType {
    let start_loc = ctx.start_location_node();
    let old_strict = ctx.strict_mode;
    // A class definition is always strict mode code.
    ctx.strict_mode = true;
    ctx.next_unwrap();

    let id = if !nullable_id || ctx.cur_token_is(TokenLabel::Name) {
        Some(parse_ident(ctx, true))
    } else {
        None
    };
    let super_class = if ctx.eat(TokenLabel::_Extends) {
        Some(parse_expr_subscripts(ctx))
    } else {
        None
    };

    let class_body = parse_class_body(ctx, super_class.is_some());
    ctx.strict_mode = old_strict;

    if id.is_some() {
        ClassDeclaration::new(
            id.unwrap(),
            super_class,
            class_body,
            ctx.compose_loc_info(start_loc),
        )
        .into()
    } else {
        AnonymousDefaultExportedClassDeclaration::new(
            super_class,
            class_body,
            ctx.compose_loc_info(start_loc),
        )
        .into()
    }
}

pub fn parse_class_body(ctx: &mut Parser, has_super: bool) -> ClassBody {
    let start_loc = ctx.start_location_node();
    let mut has_constructor = false;
    let mut body_eles: Vec<ClassBodyEl> = vec![];

    ctx.expect(TokenLabel::BraceL);
    ctx.enter_private_name_stack();

    while !ctx.eat(TokenLabel::BraceR) {
        if ctx.cur_token_is(TokenLabel::Semi) {
            continue;
        }
        let element = parse_class_element(ctx, has_super);
        // check if class element definition conflicts
        match &element {
            ClassBodyEl::MethodDefinition(mtd) => {
                if matches!(mtd.kind, MethodKind::Constructor) {
                    if has_constructor {
                        panic!("Duplicate constructor in the same class");
                    } else {
                        has_constructor = true;
                    }
                }
                match &mtd.key {
                    ClassMethodKey::PrivateIdentifier(pri_ident) => {
                        let accessor_kind = match &mtd.kind {
                            MethodKind::Get => Some(AccessorKind::Get),
                            MethodKind::Set => Some(AccessorKind::Set),
                            _ => None,
                        };
                        check_private_name_conflicts(
                            ctx.private_name_stack.last_mut().unwrap(),
                            &pri_ident.name,
                            mtd.is_static,
                            accessor_kind,
                        )
                    }
                    _ => {}
                }
            }
            ClassBodyEl::PropertyDefinition(prop) => match &prop.key {
                ClassPropertyKey::PrivateIdentifier(pri_ident) => check_private_name_conflicts(
                    ctx.private_name_stack.last_mut().unwrap(),
                    &pri_ident.name,
                    prop.is_static,
                    None,
                ),
                _ => {}
            },
            _ => {}
        }
        body_eles.push(element);
    }

    let private_info = ctx.exit_private_name_stack().unwrap();
    check_used_private_name(ctx, private_info);

    ClassBody::new(body_eles, ctx.compose_loc_info(start_loc))
}

pub enum ClassElementKey {
    Identifier(Identifier),
    Expression(Expression),
    PrivateIdentifier(PrivateIdentifier),
    Literal(Literal),
}

impl From<ClassElementKey> for ClassMethodKey {
    fn from(value: ClassElementKey) -> Self {
        match value {
            ClassElementKey::Identifier(ident) => Self::Expression(ident.into()),
            ClassElementKey::Expression(expr) => Self::Expression(expr),
            ClassElementKey::Literal(literal) => Self::Expression(literal.into()),
            ClassElementKey::PrivateIdentifier(priva_ident) => Self::PrivateIdentifier(priva_ident),
        }
    }
}

impl From<ClassElementKey> for ClassPropertyKey {
    fn from(value: ClassElementKey) -> Self {
        match value {
            ClassElementKey::Identifier(ident) => Self::Expression(ident.into()),
            ClassElementKey::Expression(expr) => Self::Expression(expr),
            ClassElementKey::Literal(literal) => Self::Expression(literal.into()),
            ClassElementKey::PrivateIdentifier(priva_ident) => Self::PrivateIdentifier(priva_ident),
        }
    }
}

pub fn parse_class_element(ctx: &mut Parser, has_super: bool) -> ClassBodyEl {
    let start_loc = ctx.start_location_node();
    let mut element_key: Option<ClassElementKey> = None;
    let mut is_static = false;
    let mut is_async = false;
    let mut is_generator = false;
    let mut method_kind = MethodKind::Method;

    if ctx.is_contextual("static") {
        element_key = Some(ClassElementKey::Identifier(parse_ident(ctx, true)));
        // TODO: check the ecam version >= 13
        if ctx.cur_token_is(TokenLabel::BraceL) {
            let old_labels = ctx.labels.clone();
            let mut body = vec![];
            ctx.labels = vec![];
            ctx.enter_scope(SCOPE_CLASS_STATIC_BLOCK | SCOPE_SUPER);

            while ctx.eat(TokenLabel::BraceR) {
                body.push(parse_statement(ctx));
            }

            ctx.exit_scope();
            ctx.labels = old_labels;

            return ClassBodyEl::StaticBlock(StaticBlock::new(
                body,
                ctx.compose_loc_info(start_loc),
            ));
        }
        if ctx.cur_token_test(|t| {
            let label = t.label;
            return label == TokenLabel::Name
                || label == TokenLabel::PrivateId
                || label == TokenLabel::Number
                || label == TokenLabel::String
                || label == TokenLabel::BracketL
                || label == TokenLabel::Star
                || t.keyword;
        }) {
            is_static = true;
            element_key = None;
        }
    }

    // check if this is async func
    if ctx.is_contextual("async") {
        element_key = Some(ClassElementKey::Identifier(parse_ident(ctx, true)));
        if ctx.cur_token_test(|t| {
            let label = t.label;
            return label == TokenLabel::Name
                || label == TokenLabel::PrivateId
                || label == TokenLabel::Number
                || label == TokenLabel::String
                || label == TokenLabel::BracketL
                || label == TokenLabel::Star
                || t.keyword;
        }) {
            is_async = true;
            element_key = None;
        }
    }

    if ctx.eat(TokenLabel::Star) {
        is_generator = true;
    }

    // maybe getter/setter method
    if !is_async && !is_generator && (ctx.is_contextual("get") || ctx.is_contextual("set")) {
        element_key = Some(ClassElementKey::Identifier(parse_ident(ctx, true)));
        if ctx.cur_token_test(|t| {
            let label = t.label;
            return label == TokenLabel::Name
                || label == TokenLabel::PrivateId
                || label == TokenLabel::Number
                || label == TokenLabel::String
                || label == TokenLabel::BracketL
                || t.keyword;
        }) {
            method_kind = if ctx.cur_token_value_is("get") {
                MethodKind::Get
            } else {
                MethodKind::Set
            };
            element_key = None;
        }
    }

    let mut computed = false;

    // Parse element_key if need.
    if element_key.is_none() {
        if ctx.cur_token_is(TokenLabel::PrivateId) {
            element_key = Some(ClassElementKey::PrivateIdentifier(parse_private_ident(ctx)));
        } else {
            computed = ctx.eat(TokenLabel::ParenL);
            if computed {
                element_key = Some(ClassElementKey::Expression(parse_expression(ctx)));
            } else if ctx
                .cur_token_test(|t| t.label == TokenLabel::String || t.label == TokenLabel::Number)
            {
                element_key = Some(ClassElementKey::Literal(parse_literal(ctx)));
            } else {
                element_key = Some(ClassElementKey::Identifier(parse_ident(ctx, true)));
            }
        }
    }

    if ctx.cur_token_is(TokenLabel::ParenL)
        || is_generator
        || is_async
        || method_kind != MethodKind::Method
    {
        let is_constructor =
            !is_static && check_class_ele_key(element_key.as_ref().unwrap(), "constructor");
        if is_constructor {
            if method_kind != MethodKind::Method {
                panic!("Constructor can't have get/set modifier");
            }
            if is_generator {
                panic!("Constructor can't be a generator");
            }
            if is_async {
                panic!("Constructor can't be an async method");
            }
            method_kind = MethodKind::Constructor;
        }
        if is_static && check_class_ele_key(element_key.as_ref().unwrap(), "prototype") {
            panic!("Classes may not have a static property named prototype");
        }
        let ele_value = parse_method(ctx, is_generator, is_async, has_super && is_constructor);
        match method_kind {
            MethodKind::Get => {
                if ele_value.params.len() != 0 {
                    println!("getter should have no params");
                }
            }
            MethodKind::Set => {
                if ele_value.params.len() != 1 {
                    println!("setter should have exactly one param");
                }
                if let Pattern::RestElement(res) = ele_value.params.last().unwrap() {
                    println!("Setter cannot use rest params");
                }
            }
            _ => {}
        }
        return ClassBodyEl::MethodDefinition(MethodDefinition::new(
            element_key.unwrap().into(),
            ele_value,
            method_kind,
            computed,
            is_static,
            ctx.compose_loc_info(start_loc),
        ));
    }

    // TODO: check ecam version > 13
    if check_class_ele_key(element_key.as_ref().unwrap(), "constructor") {
        panic!("Classes can't have a field named 'constructor'")
    }

    if is_static && check_class_ele_key(element_key.as_ref().unwrap(), "prototype") {
        panic!("Classes can't have a static field named 'prototype'")
    }

    let mut field_value = None;
    if ctx.eat(TokenLabel::Eq) {
        let cur_scope = ctx.scope_stack.last_mut().unwrap();
        let old_in_class_field_init = cur_scope.in_class_field_init;
        cur_scope.in_class_field_init = true;
        field_value = Some(parse_maybe_assign(ctx));
        ctx.scope_stack.last_mut().unwrap().in_class_field_init = old_in_class_field_init;
    }

    ctx.semicolon();

    ClassBodyEl::PropertyDefinition(PropertyDefinition::new(
        element_key.unwrap().into(),
        field_value,
        computed,
        is_static,
        ctx.compose_loc_info(start_loc),
    ))
}

pub fn check_class_ele_key(el: &ClassElementKey, name: &str) -> bool {
    match el {
        ClassElementKey::Identifier(iden) => iden.name == name,
        ClassElementKey::Literal(Literal { value, .. }) => match value {
            LiteralValue::String(v) => v == name,
            _ => false,
        },
        _ => false,
    }
}

pub fn parse_function_stmt(
    ctx: &mut Parser,
    nullable_id: bool,
    is_async: bool,
) -> FunctionDeclarationType {
    let start_loc = ctx.start_location_node();
    // let top_level_ctx = ctx.cur_stmt_ctx_is(StatementContext::TopLevel);
    if is_async {
        ctx.expect_contexual("async");
    }
    ctx.expect(TokenLabel::_Function);
    let is_generator = ctx.eat(TokenLabel::Star);
    // TODO: figure out why this is invalid
    // if is_generator && func_stmt_flags & FUNC_HANGING_STATEMENT > 0 {
    //     unexpected(ctx.cur_token.clone().unwrap());
    // }

    let id = if !nullable_id || ctx.cur_token_is(TokenLabel::Name) {
        Some(parse_ident(ctx, true))
    } else {
        None
    };

    // TODO: check for hanging function statement
    // if id.is_some() && statment_flag & FUNC_HANGING_STATEMENT > 0 {
    // }

    ctx.enter_scope(get_func_flags(is_async, is_generator));
    ctx.expect(TokenLabel::ParenL);

    let params = parse_binding_list(ctx, TokenLabel::ParenR, false, true);
    let body = parse_function_body(ctx);
    ctx.exit_scope();

    if id.is_some() {
        FunctionDeclaration::new(
            id.unwrap(),
            params.into_iter().map(|p| p.unwrap()).collect(),
            body,
            is_generator,
            is_async,
            ctx.compose_loc_info(start_loc),
        )
        .into()
    } else {
        AnonymousDefaultExportedFunctionDeclaration::new(
            params.into_iter().map(|p| p.unwrap()).collect(),
            body,
            is_generator,
            is_async,
            ctx.compose_loc_info(start_loc),
        )
        .into()
    }
}

// There are many different kind of for loop in javascript grammar. As the following:
// - for (let/const/var .. in/of ..) {}
// - for (var/let/const;;) {}
// - for (expr;;) {}
// - for (;;) {}
// - for await (let/const/var .. of ..) {}
// - for (lhs in/of ..) {}
pub fn parse_for_loop(ctx: &mut Parser) -> Statement {
    let start_loc = ctx.start_location_node();
    ctx.next_unwrap();
    ctx.labels.push(Label {
        kind: LabelKind::Loop,
        name: None,
    });
    ctx.enter_scope(0);

    let is_for_await = ctx.can_await() && ctx.eat_contextual("await");
    ctx.expect(TokenLabel::ParenL);
    // handle case: for (;;) {}
    if ctx.cur_token_is(TokenLabel::Semi) {
        if is_for_await {
            unexpected(ctx.cur_token.clone().unwrap())
        }
        return parse_regular_for(ctx, start_loc, ForStatementInit::Null).into();
    }

    let is_let = is_let(ctx);
    if is_let
        || ctx.cur_token_test(|t| t.label == TokenLabel::_Var || t.label == TokenLabel::_Const)
    {
        let init_start_loc = ctx.start_location_node();
        let kind = if is_let {
            VariableKind::Let
        } else if ctx.cur_token_is(TokenLabel::_Var) {
            VariableKind::Var
        } else {
            VariableKind::Const
        };

        ctx.for_init = Some(if is_for_await {
            ForInitType::Await
        } else {
            ForInitType::Normal
        });
        let declarators = parse_var_declarator(ctx, kind.clone());
        ctx.for_init = None;

        // handle case: for (var/let/const .. in/of ..) {}
        if (ctx.cur_token_is(TokenLabel::_In) || ctx.is_contextual("of")) && declarators.len() == 1
        {
            if is_for_await && ctx.cur_token_is(TokenLabel::_In) {
                unexpected(ctx.cur_token.clone().unwrap());
            }
            // check if there is init value in declarator
            if declarators
                .first()
                .map_or(false, |declarator| declarator.init.is_some())
            {
                panic!(
                    "{} loop variable declaration may not have an initializer",
                    if ctx.cur_token_is(TokenLabel::_In) {
                        "for-in"
                    } else {
                        "for-of"
                    }
                );
            }
            return parse_for_in_of(
                ctx,
                VariableDeclaration::new(declarators, kind, ctx.compose_loc_info(init_start_loc))
                    .into(),
                start_loc,
                is_for_await,
            );
        }
        // for await can only be used with for_of syntax
        if is_for_await {
            unexpected(ctx.cur_token.clone().unwrap())
        }
        // hanld case: for (var/let/cont ..;;) {}
        return parse_regular_for(
            ctx,
            start_loc,
            VariableDeclaration::new(declarators, kind, ctx.compose_loc_info(init_start_loc))
                .into(),
        )
        .into();
    }

    // handle case: for (lhs in/of ..) {}
    ctx.for_init = Some(if is_for_await {
        ForInitType::Await
    } else {
        ForInitType::Normal
    });
    let starts_with_let = ctx.is_contextual("let");
    let init = parse_expression(ctx);
    ctx.for_init = None;
    if ctx.cur_token_is(TokenLabel::_In) || ctx.is_contextual("of") {
        if is_for_await && ctx.cur_token_is(TokenLabel::_In) {
            unexpected(ctx.cur_token.clone().unwrap());
        }
        if starts_with_let && ctx.is_contextual("of") {
            panic!("The left-hand side of a for-of loop may not start with 'let'.")
        }
        // TODO: convert expression to lhs
        // return parse_for_in_of(ctx, ForInOfStatementLeft::Pattern(), start_loc, is_await);
    }

    // hanlde case: for (;;) {}
    parse_regular_for(ctx, start_loc, init.into()).into()
}

// parse a for-in or for-of loop
pub fn parse_for_in_of(
    ctx: &mut Parser,
    left: ForInOfStatementLeft,
    start_loc: _LocationNode,
    is_await: bool,
) -> Statement {
    let is_for_in = ctx.cur_token_is(TokenLabel::_In);
    ctx.next_unwrap();
    let right = if is_for_in {
        parse_expression(ctx)
    } else {
        parse_maybe_assign(ctx)
    };
    ctx.expect(TokenLabel::ParenR);
    ctx.enter_stmt_ctx(StatementContext::For);
    let body = parse_statement(ctx);
    ctx.exit_stmt_ctx();
    ctx.exit_scope();
    ctx.labels.pop();

    if is_for_in {
        ForInStatement::new(left, right, Box::new(body), ctx.compose_loc_info(start_loc)).into()
    } else {
        ForOfStatement::new(
            left,
            right,
            Box::new(body),
            is_await,
            ctx.compose_loc_info(start_loc),
        )
        .into()
    }
}

// parse the regular for loop: for(;;) {}
pub fn parse_regular_for(
    ctx: &mut Parser,
    start_loc: _LocationNode,
    init: ForStatementInit,
) -> ForStatement {
    ctx.expect(TokenLabel::Semi);
    let test = if ctx.cur_token_is(TokenLabel::Semi) {
        None
    } else {
        Some(parse_expression(ctx))
    };
    ctx.expect(TokenLabel::Semi);
    let update = if ctx.cur_token_is(TokenLabel::Semi) {
        None
    } else {
        Some(parse_expression(ctx))
    };
    ctx.expect(TokenLabel::Semi);

    ctx.enter_stmt_ctx(StatementContext::For);
    let body = parse_statement(ctx);
    ctx.exit_stmt_ctx();
    ctx.exit_scope();
    ctx.labels.pop();

    ForStatement::new(
        init,
        test,
        update,
        Box::new(body),
        ctx.compose_loc_info(start_loc),
    )
}

pub fn parse_var_declarator(ctx: &mut Parser, kind: VariableKind) -> Vec<VariableDeclarator> {
    let mut declarations: Vec<VariableDeclarator> = vec![];
    loop {
        let dec_start_loc = ctx.start_location_node();
        let id: Pattern = parse_binding_atom(ctx).into();
        let mut init = None;
        // TODO: check if pattern is valid
        if ctx.eat(TokenLabel::Eq) {
            init = Some(parse_maybe_assign(ctx));
        }
        // the const declarations must have initial value when it's not in for[in/of] loop
        else if matches!(kind, VariableKind::Const)
            && (ctx.cur_token_is(TokenLabel::_In) || ctx.is_contextual("of"))
        {
            panic!("const declarations must have initial value");
        }
        // complex pattern can have no initial value only when it appear in for/[in/of] loop
        else if !matches!(id, Pattern::Identifier(..))
            && !(ctx.for_init.is_some()
                && (ctx.cur_token_is(TokenLabel::_In) || ctx.is_contextual("of")))
        {
            panic!("Complex binding patterns require an initialization value");
        }

        declarations.push(VariableDeclarator::new(
            id,
            init,
            ctx.compose_loc_info(dec_start_loc),
        ));

        if !ctx.eat(TokenLabel::Comma) {
            break;
        }
    }

    declarations
}

pub fn parse_do_loop(ctx: &mut Parser) -> DoWhileStatement {
    let start_loc = ctx.start_location_node();
    ctx.next_unwrap();
    ctx.labels.push(Label {
        kind: LabelKind::Loop,
        name: None,
    });

    ctx.enter_stmt_ctx(StatementContext::DoWhile);
    let body = parse_statement(ctx);
    ctx.exit_stmt_ctx();
    ctx.labels.pop();
    ctx.expect(TokenLabel::_While);
    let test = parse_paren_expression(ctx);

    ctx.semicolon();

    DoWhileStatement::new(Box::new(body), test, ctx.compose_loc_info(start_loc))
}

pub fn parse_debugger(ctx: &mut Parser) -> DebuggerStatement {
    let start_loc = ctx.start_location_node();
    ctx.next_unwrap();
    ctx.semicolon();

    DebuggerStatement::new(ctx.compose_loc_info(start_loc))
}

pub fn parse_break_continue(ctx: &mut Parser) -> Statement {
    let is_break = ctx.cur_token_is(TokenLabel::_Break);
    let start_loc = ctx.start_location_node();
    let label;

    ctx.next_unwrap();

    if ctx.eat(TokenLabel::Semi) || can_insert_semicolon(ctx) {
        label = None;
    } else if !ctx.cur_token_is(TokenLabel::Name) {
        unexpected(ctx.cur_token.clone().unwrap());
    } else {
        label = Some(parse_ident(ctx, false));
        ctx.semicolon();
    }

    if !check_label_destination(&ctx.labels, &label, is_break) {
        println!(
            "Unsyntactic {}",
            if is_break { "break" } else { "continue" }
        );
    }

    if is_break {
        BreakStatement::new(label, ctx.compose_loc_info(start_loc)).into()
    } else {
        ContinueStatement::new(label, ctx.compose_loc_info(start_loc)).into()
    }
}

// Parse a block statement
pub fn parse_block_stmt(ctx: &mut Parser, new_lexical_scope: bool) -> BlockStatement {
    let mut body = vec![];
    let start_loc = ctx.start_location_node();
    if new_lexical_scope {
        ctx.enter_scope(0);
    }
    ctx.expect(TokenLabel::BraceL);
    while !ctx.eat(TokenLabel::BraceR) {
        let stmt = parse_statement(ctx);
        body.push(stmt);
    }

    if new_lexical_scope {
        ctx.exit_scope();
    }

    BlockStatement::new(body, ctx.compose_loc_info(start_loc))
}
