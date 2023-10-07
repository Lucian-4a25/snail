use std::collections::HashMap;

lazy_static! {
    pub static ref KeywordList: Vec<TokenLabel> = vec![
        TokenLabel::_Break,
        TokenLabel::_Case,
        TokenLabel::_Catch,
        TokenLabel::_Continue,
        TokenLabel::_Debugger,
        TokenLabel::_Default,
        TokenLabel::_Do,
        TokenLabel::_Else,
        TokenLabel::_Finally,
        TokenLabel::_For,
        TokenLabel::_Function,
        TokenLabel::_If,
        TokenLabel::_Return,
        TokenLabel::_Switch,
        TokenLabel::_Throw,
        TokenLabel::_Try,
        TokenLabel::_Var,
        TokenLabel::_While,
        TokenLabel::_With,
        TokenLabel::_Null,
        TokenLabel::_True,
        TokenLabel::_False,
        TokenLabel::_InstanceOf,
        TokenLabel::_Typeof,
        TokenLabel::_Void,
        TokenLabel::_Delete,
        TokenLabel::_New,
        TokenLabel::_In,
        TokenLabel::_This,
        TokenLabel::_Export,
        TokenLabel::_Import,
        TokenLabel::_Const,
        TokenLabel::_Class,
        TokenLabel::_Extends,
        TokenLabel::_Export,
        TokenLabel::_Import,
        TokenLabel::_Super
    ];
    pub static ref TokenMap: HashMap<TokenLabel, Token> = HashMap::from([
        (TokenLabel::Number, Token::se(TokenLabel::Number)),
        (TokenLabel::Regexp, Token::se(TokenLabel::Regexp)),
        (TokenLabel::String, Token::se(TokenLabel::String)),
        (TokenLabel::Name, Token { update_ctx: true, ..Token::se(TokenLabel::Name) }),
        (TokenLabel::PrivateId, Token::se(TokenLabel::PrivateId)),
        (TokenLabel::Eof, Token::new(TokenLabel::Eof)),

        // punctuation token types
        (TokenLabel::BracketL, Token { starts_expr: true, before_expr: true, ..Token::new(TokenLabel::BracketL) }),
        (TokenLabel::BracketR, Token::new(TokenLabel::BracketR)),
        (TokenLabel::BraceL, Token { update_ctx: true, starts_expr: true, before_expr: true, ..Token::new(TokenLabel::BraceL)}),
        (TokenLabel::BraceR, Token { update_ctx: true, ..Token::new(TokenLabel::BraceR) }),
        (TokenLabel::ParenL, Token { update_ctx: true, starts_expr: true, before_expr: true, ..Token::new(TokenLabel::ParenL)}),
        (TokenLabel::ParenR, Token { update_ctx: true, ..Token::new(TokenLabel::ParenR) }),
        (TokenLabel::Comma, Token { before_expr: true, ..Token::new(TokenLabel::Comma) }),
        (TokenLabel::Semi, Token { before_expr: true, ..Token::new(TokenLabel::Semi) }),
        (TokenLabel::Colon, Token { before_expr: true, ..Token::new(TokenLabel::Colon) }),
        (TokenLabel::Dot, Token::new(TokenLabel::Dot)),
        (TokenLabel::Question, Token { before_expr: true, ..Token::new(TokenLabel::Question) }),
        (TokenLabel::QuestionDot, Token::new(TokenLabel::QuestionDot)),
        (TokenLabel::Arrow, Token { before_expr: true, ..Token::new(TokenLabel::Arrow) }),
        (TokenLabel::Template, Token::new(TokenLabel::Template)),
        (TokenLabel::Invalidtemplate, Token::new(TokenLabel::Invalidtemplate)),
        (TokenLabel::Ellipsis, Token { before_expr: true, ..Token::new(TokenLabel::Ellipsis) }),
        (TokenLabel::BackQuote, Token { update_ctx: true, starts_expr: true, ..Token::new(TokenLabel::BackQuote)}),
        (TokenLabel::DollarBraceL, Token { update_ctx: true, starts_expr: true, before_expr: true, ..Token::new(TokenLabel::DollarBraceL)}),

        // Operators Token
        (TokenLabel::Eq, Token { before_expr: true, is_assign: true, ..Token::new(TokenLabel::Eq) }),
        (TokenLabel::Assign, Token { before_expr: true, is_assign: true, ..Token::new(TokenLabel::Assign) }),
        (TokenLabel::IncDec, Token { prefix: true, postfix: true, ..Token::new(TokenLabel::IncDec) }),
        (TokenLabel::Prefix, Token { before_expr: true, prefix: true, starts_expr: true, ..Token::new(TokenLabel::Prefix) }),
        (TokenLabel::LogicalOr, Token::binop(TokenLabel::LogicalOr, 1)),
        (TokenLabel::LogicalAnd, Token::binop(TokenLabel::LogicalAnd, 2)),
        (TokenLabel::BitwiseOr, Token::binop(TokenLabel::BitwiseOr, 3)),
        (TokenLabel::BitwiseXor, Token::binop(TokenLabel::BitwiseXor, 4)),
        (TokenLabel::BitwiseAnd, Token::binop(TokenLabel::BitwiseAnd, 5)),
        (TokenLabel::Equlity, Token::binop(TokenLabel::Equlity, 6)),
        (TokenLabel::Relational, Token::binop(TokenLabel::Relational, 7)),
        (TokenLabel::BitShift, Token::binop(TokenLabel::BitShift, 8)),
        (TokenLabel::PlusMin, Token { before_expr: true, prefix: true, starts_expr: true, .. Token::binop(TokenLabel::PlusMin, 9) }),
        (TokenLabel::Modulo, Token::binop(TokenLabel::Modulo, 10)),
        (TokenLabel::Star, Token { update_ctx: true, ..Token::binop(TokenLabel::Star, 10) }),
        (TokenLabel::Slash, Token::binop(TokenLabel::Slash, 10)),
        (TokenLabel::StarStar, Token { before_expr: true, ..Token::binop(TokenLabel::StarStar, 11) }),
        (TokenLabel::Coalesce, Token::binop(TokenLabel::Coalesce, 1)),

        // Keyword Token
        (TokenLabel::_Break, Token::kw(TokenLabel::_Break)),
        (TokenLabel::_Case, Token { before_expr: true, ..Token::kw(TokenLabel::_Case) }),
        (TokenLabel::_Catch, Token::kw(TokenLabel::_Catch)),
        (TokenLabel::_Continue, Token::kw(TokenLabel::_Continue)),
        (TokenLabel::_Debugger, Token::kw(TokenLabel::_Debugger)),
        (TokenLabel::_Default, Token::kw(TokenLabel::_Default)),
        (TokenLabel::_Do, Token { is_loop: true, before_expr: true, ..Token::kw(TokenLabel::_Do) }),
        (TokenLabel::_Else, Token { before_expr: true, ..Token::kw(TokenLabel::_Else) }),
        (TokenLabel::_Finally, Token::kw(TokenLabel::_Finally)),
        (TokenLabel::_For, Token { is_loop: true, ..Token::kw(TokenLabel::_For)}),
        (TokenLabel::_Function, Token { update_ctx: true, starts_expr: true, ..Token::kw(TokenLabel::_Function) }),
        (TokenLabel::_If, Token::kw(TokenLabel::_If)),
        (TokenLabel::_Return, Token { before_expr: true, ..Token::kw(TokenLabel::_Return) }),
        (TokenLabel::_Switch, Token::kw(TokenLabel::_Switch)),
        (TokenLabel::_Throw, Token::kw(TokenLabel::_Throw)),
        (TokenLabel::_Try, Token::kw(TokenLabel::_Try)),
        (TokenLabel::_Var, Token::kw(TokenLabel::_Var)),
        (TokenLabel::_Const, Token::kw(TokenLabel::_Const)),
        (TokenLabel::_While, Token { is_loop: true, ..Token::kw(TokenLabel::_While) }),
        (TokenLabel::_With, Token::kw(TokenLabel::_With)),
        (TokenLabel::_New, Token { starts_expr: true, before_expr: true, ..Token::kw(TokenLabel::_New) }),
        (TokenLabel::_This, Token { starts_expr: true, ..Token::kw(TokenLabel::_This) }),
        (TokenLabel::_Super, Token { starts_expr: true, ..Token::kw(TokenLabel::_Super) }),
        (TokenLabel::_Class, Token { update_ctx: true, starts_expr: true, ..Token::kw(TokenLabel::_Class) }),
        (TokenLabel::_Extends, Token { before_expr: true, ..Token::kw(TokenLabel::_Extends) }),
        (TokenLabel::_Export, Token::kw(TokenLabel::_Export)),
        (TokenLabel::_Import, Token { starts_expr: true, ..Token::kw(TokenLabel::_Import) }),
        (TokenLabel::_Null, Token { starts_expr: true, ..Token::kw(TokenLabel::_Null) }),
        (TokenLabel::_True, Token { starts_expr: true, ..Token::kw(TokenLabel::_True) }),
        (TokenLabel::_False, Token { starts_expr: true, ..Token::kw(TokenLabel::_False) }),
        (TokenLabel::_In, Token { before_expr: true, binop: Some(7), ..Token::kw(TokenLabel::_In) }),
        (TokenLabel::_InstanceOf, Token { before_expr: true, binop: Some(7), ..Token::kw(TokenLabel::_InstanceOf) }),
        (TokenLabel::_Typeof, Token { before_expr: true, prefix: true, starts_expr: true, ..Token::kw(TokenLabel::_Typeof) }),
        (TokenLabel::_Void, Token { before_expr: true, prefix: true, starts_expr: true, ..Token::kw(TokenLabel::_Void) }),
        (TokenLabel::_Delete, Token { before_expr: true, prefix: true, starts_expr: true, ..Token::kw(TokenLabel::_Delete) }),
    ]);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TokenLabel {
    Number,
    Regexp,
    String,
    Name,
    PrivateId,
    Eof,

    // Single Token Type
    BracketL,
    BracketR,
    BraceL,
    BraceR,
    ParenL,
    ParenR,
    Comma,
    Semi,
    Colon,
    Dot,
    Question,
    QuestionDot,
    Arrow,
    Template,
    Invalidtemplate,
    Ellipsis,
    BackQuote,
    DollarBraceL,

    // Opeartors
    Eq,
    Assign,
    IncDec,
    Prefix,
    LogicalOr,
    LogicalAnd,
    BitwiseOr,
    BitwiseXor,
    BitwiseAnd,
    Equlity,
    Relational,
    BitShift,
    PlusMin,
    Modulo,
    Star,
    Slash,
    StarStar,
    Coalesce,

    // Keyword
    _Break,
    _Case,
    _Catch,
    _Continue,
    _Debugger,
    _Default,
    _Do,
    _Else,
    _Finally,
    _For,
    _Function,
    _If,
    _Return,
    _Switch,
    _Throw,
    _Try,
    _Var,
    _Const,
    _While,
    _With,
    _New,
    _This,
    _Super,
    _Class,
    _Extends,
    _Export,
    _Import,
    _Null,
    _True,
    _False,
    _In,
    _InstanceOf,
    _Typeof,
    _Void,
    _Delete,
}

impl TokenLabel {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Number => "num",
            Self::Regexp => "regexp",
            Self::String => "string",
            Self::Name => "name",
            Self::PrivateId => "privateId",
            Self::Eof => "eof",

            // Single Token Type
            Self::BracketL => "[",
            Self::BracketR => "]",
            Self::BraceL => "{",
            Self::BraceR => "}",
            Self::ParenL => "(",
            Self::ParenR => ")",
            Self::Comma => ",",
            Self::Semi => ";",
            Self::Colon => ":",
            Self::Dot => ".",
            Self::Question => "?",
            Self::QuestionDot => "?.",
            Self::Arrow => "=>",
            Self::Template => "template",
            Self::Invalidtemplate => "invalidTemplate",
            Self::Ellipsis => "...",
            Self::BackQuote => "`",
            Self::DollarBraceL => "${",
            // Opeartors
            Self::Eq => "=",
            Self::Assign => "_=",
            Self::IncDec => "++/--",
            Self::Prefix => "!/~",
            Self::LogicalOr => "||",
            Self::LogicalAnd => "&&",
            Self::BitwiseOr => "|",
            Self::BitwiseXor => "^",
            Self::BitwiseAnd => "&",
            Self::Equlity => "==/!=/===/!==",
            Self::Relational => "</>/<=/>=",
            Self::BitShift => "<</>>/>>>",
            Self::PlusMin => "+/-",
            Self::Modulo => "%",
            Self::Star => "*",
            Self::Slash => "/",
            Self::StarStar => "**",
            Self::Coalesce => "??",
            // Keyword
            Self::_Break => "break",
            Self::_Case => "case",
            Self::_Catch => "catch",
            Self::_Continue => "continue",
            Self::_Debugger => "debugger",
            Self::_Default => "default",
            Self::_Do => "do",
            Self::_Else => "else",
            Self::_Finally => "finally",
            Self::_For => "for",
            Self::_Function => "function",
            Self::_If => "if",
            Self::_Return => "return",
            Self::_Switch => "switch",
            Self::_Throw => "throw",
            Self::_Try => "try",
            Self::_Var => "var",
            Self::_Const => "const",
            Self::_While => "while",
            Self::_With => "with",
            Self::_New => "new",
            Self::_This => "this",
            Self::_Super => "super",
            Self::_Class => "class",
            Self::_Extends => "extends",
            Self::_Export => "export",
            Self::_Import => "import",
            Self::_Null => "null",
            Self::_True => "true",
            Self::_False => "false",
            Self::_In => "in",
            Self::_InstanceOf => "instanceof",
            Self::_Typeof => "typeof",
            Self::_Void => "void",
            Self::_Delete => "delete",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Token {
    /// 标识 Token 的类型
    pub label: TokenLabel,
    /// token 的值
    pub value: Option<String>,
    /// 标识是否是内置关键字
    pub keyword: bool,
    /// 用于区分除法和正则
    pub before_expr: bool,
    /// 标识是否可以紧接一个表达式
    pub starts_expr: bool,
    /// 标识这个 token 是否是一个 loop 的开始
    pub is_loop: bool,
    /// 标识这个 token 是否是一个赋值
    pub is_assign: bool,
    /// 标记操作符是前置还是后置一元操作符
    pub prefix: bool,
    pub postfix: bool,
    /// 表示是一个二进制操作符，值为它的优先级
    pub binop: Option<u8>,
    /// 表示是否需要更新上下文
    pub update_ctx: bool,
}

impl Default for Token {
    fn default() -> Self {
        Self {
            label: TokenLabel::Name,
            value: None,
            keyword: false,
            before_expr: false,
            starts_expr: false,
            is_loop: false,
            is_assign: false,
            prefix: false,
            postfix: false,
            binop: None,
            update_ctx: false,
        }
    }
}

impl Token {
    fn new(label: TokenLabel) -> Self {
        Self {
            label: label,
            ..Default::default()
        }
    }

    fn se(label: TokenLabel) -> Self {
        Self {
            label: label,
            starts_expr: true,
            ..Default::default()
        }
    }

    fn kw(label: TokenLabel) -> Self {
        Self {
            label: label,
            keyword: true,
            ..Default::default()
        }
    }

    fn binop(label: TokenLabel, pre: u8) -> Self {
        Self {
            label: label,
            binop: Some(pre),
            ..Default::default()
        }
    }

    pub fn is_eof(&self) -> bool {
        self.label == TokenLabel::Eof
    }

    pub fn is_token_of(&self, label: TokenLabel) -> bool {
        self.label == label
    }
}
