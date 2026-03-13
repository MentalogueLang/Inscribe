// TODO: Add richer token metadata once diagnostics and macro expansion need it.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Position {
    pub offset: usize,
    pub line: usize,
    pub column: usize,
}

impl Position {
    pub const fn new(offset: usize, line: usize, column: usize) -> Self {
        Self {
            offset,
            line,
            column,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Span {
    pub start: Position,
    pub end: Position,
}

impl Span {
    pub const fn new(start: Position, end: Position) -> Self {
        Self { start, end }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Token {
    pub fn new(kind: TokenKind, span: Span) -> Self {
        Self { kind, span }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    Identifier(String),
    Integer(String),
    Float(String),
    String(String),
    True,
    False,
    Let,
    Const,
    Fn,
    Struct,
    If,
    Else,
    For,
    In,
    While,
    Return,
    Import,
    Match,
    LParen,
    RParen,
    LBrace,
    RBrace,
    Comma,
    Colon,
    Dot,
    DotDot,
    Arrow,
    FatArrow,
    Question,
    Plus,
    Minus,
    Star,
    Slash,
    Equal,
    EqualEqual,
    Bang,
    BangEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    AndAnd,
    OrOr,
    Semicolon,
    Newline,
    Eof,
}

impl TokenKind {
    pub fn describe(&self) -> &'static str {
        match self {
            Self::Identifier(_) => "identifier",
            Self::Integer(_) => "integer literal",
            Self::Float(_) => "float literal",
            Self::String(_) => "string literal",
            Self::True => "`true`",
            Self::False => "`false`",
            Self::Let => "`let`",
            Self::Const => "`const`",
            Self::Fn => "`fn`",
            Self::Struct => "`struct`",
            Self::If => "`if`",
            Self::Else => "`else`",
            Self::For => "`for`",
            Self::In => "`in`",
            Self::While => "`while`",
            Self::Return => "`return`",
            Self::Import => "`import`",
            Self::Match => "`match`",
            Self::LParen => "`(`",
            Self::RParen => "`)`",
            Self::LBrace => "`{`",
            Self::RBrace => "`}`",
            Self::Comma => "`,`",
            Self::Colon => "`:`",
            Self::Dot => "`.`",
            Self::DotDot => "`..`",
            Self::Arrow => "`->`",
            Self::FatArrow => "`=>`",
            Self::Question => "`?`",
            Self::Plus => "`+`",
            Self::Minus => "`-`",
            Self::Star => "`*`",
            Self::Slash => "`/`",
            Self::Equal => "`=`",
            Self::EqualEqual => "`==`",
            Self::Bang => "`!`",
            Self::BangEqual => "`!=`",
            Self::Less => "`<`",
            Self::LessEqual => "`<=`",
            Self::Greater => "`>`",
            Self::GreaterEqual => "`>=`",
            Self::AndAnd => "`&&`",
            Self::OrOr => "`||`",
            Self::Semicolon => "`;`",
            Self::Newline => "newline",
            Self::Eof => "end of file",
        }
    }
}
