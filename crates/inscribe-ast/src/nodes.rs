use crate::span::{Span, Spanned};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Module {
    pub items: Vec<Item>,
    pub span: Span,
}

impl Spanned for Module {
    fn span(&self) -> Span {
        self.span
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Item {
    Import(Import),
    Struct(StructDecl),
    Enum(EnumDecl),
    Function(FunctionDecl),
}

impl Spanned for Item {
    fn span(&self) -> Span {
        match self {
            Self::Import(item) => item.span,
            Self::Struct(item) => item.span,
            Self::Enum(item) => item.span,
            Self::Function(item) => item.span,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Import {
    pub path: Path,
    pub span: Span,
}

impl Spanned for Import {
    fn span(&self) -> Span {
        self.span
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructDecl {
    pub name: String,
    pub name_span: Span,
    pub fields: Vec<StructField>,
    pub span: Span,
}

impl Spanned for StructDecl {
    fn span(&self) -> Span {
        self.span
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructField {
    pub name: String,
    pub name_span: Span,
    pub ty: TypeRef,
    pub span: Span,
}

impl Spanned for StructField {
    fn span(&self) -> Span {
        self.span
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnumDecl {
    pub name: String,
    pub name_span: Span,
    pub variants: Vec<EnumVariant>,
    pub span: Span,
}

impl Spanned for EnumDecl {
    fn span(&self) -> Span {
        self.span
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnumVariant {
    pub name: String,
    pub name_span: Span,
    pub span: Span,
}

impl Spanned for EnumVariant {
    fn span(&self) -> Span {
        self.span
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionDecl {
    pub visibility: Visibility,
    pub receiver: Option<Path>,
    pub name: String,
    pub name_span: Span,
    pub params: Vec<Param>,
    pub return_type: Option<TypeRef>,
    pub body: Option<Block>,
    pub span: Span,
}

impl Spanned for FunctionDecl {
    fn span(&self) -> Span {
        self.span
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Visibility {
    Public,
    Private,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Param {
    pub name: String,
    pub name_span: Span,
    pub ty: Option<TypeRef>,
    pub span: Span,
}

impl Spanned for Param {
    fn span(&self) -> Span {
        self.span
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    pub statements: Vec<Stmt>,
    pub span: Span,
}

impl Spanned for Block {
    fn span(&self) -> Span {
        self.span
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Stmt {
    Let(LetStmt),
    Const(ConstStmt),
    For(ForStmt),
    While(WhileStmt),
    Return(ReturnStmt),
    Expr(Expr),
}

impl Spanned for Stmt {
    fn span(&self) -> Span {
        match self {
            Self::Let(stmt) => stmt.span,
            Self::Const(stmt) => stmt.span,
            Self::For(stmt) => stmt.span,
            Self::While(stmt) => stmt.span,
            Self::Return(stmt) => stmt.span,
            Self::Expr(expr) => expr.span,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LetStmt {
    pub name: String,
    pub name_span: Span,
    pub ty: Option<TypeRef>,
    pub value: Expr,
    pub span: Span,
}

impl Spanned for LetStmt {
    fn span(&self) -> Span {
        self.span
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConstStmt {
    pub name: String,
    pub name_span: Span,
    pub ty: Option<TypeRef>,
    pub value: Expr,
    pub span: Span,
}

impl Spanned for ConstStmt {
    fn span(&self) -> Span {
        self.span
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ForStmt {
    pub pattern: Pattern,
    pub iterable: Expr,
    pub body: Block,
    pub span: Span,
}

impl Spanned for ForStmt {
    fn span(&self) -> Span {
        self.span
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WhileStmt {
    pub condition: Expr,
    pub body: Block,
    pub span: Span,
}

impl Spanned for WhileStmt {
    fn span(&self) -> Span {
        self.span
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReturnStmt {
    pub value: Option<Expr>,
    pub span: Span,
}

impl Spanned for ReturnStmt {
    fn span(&self) -> Span {
        self.span
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Expr {
    pub kind: ExprKind,
    pub span: Span,
}

impl Expr {
    pub fn new(kind: ExprKind, span: Span) -> Self {
        Self { kind, span }
    }
}

impl Spanned for Expr {
    fn span(&self) -> Span {
        self.span
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExprKind {
    Literal(Literal),
    Path(Path),
    Array(Vec<Expr>),
    RepeatArray {
        value: Box<Expr>,
        length: usize,
    },
    Unary {
        op: UnaryOp,
        expr: Box<Expr>,
    },
    Binary {
        op: BinaryOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Call {
        callee: Box<Expr>,
        args: Vec<Expr>,
    },
    Field {
        base: Box<Expr>,
        field: String,
    },
    Index {
        target: Box<Expr>,
        index: Box<Expr>,
    },
    StructLiteral {
        path: Path,
        fields: Vec<StructLiteralField>,
    },
    If {
        condition: Box<Expr>,
        then_block: Block,
        else_branch: Option<Box<Expr>>,
    },
    Match {
        value: Box<Expr>,
        arms: Vec<MatchArm>,
    },
    Block(Block),
    Try(Box<Expr>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructLiteralField {
    pub name: String,
    pub name_span: Span,
    pub value: Expr,
    pub span: Span,
}

impl Spanned for StructLiteralField {
    fn span(&self) -> Span {
        self.span
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub value: Expr,
    pub span: Span,
}

impl Spanned for MatchArm {
    fn span(&self) -> Span {
        self.span
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pattern {
    pub kind: PatternKind,
    pub span: Span,
}

impl Pattern {
    pub fn new(kind: PatternKind, span: Span) -> Self {
        Self { kind, span }
    }
}

impl Spanned for Pattern {
    fn span(&self) -> Span {
        self.span
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PatternKind {
    Wildcard,
    Binding(String),
    Literal(Literal),
    Path(Path),
    Constructor { path: Path, arguments: Vec<Pattern> },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Literal {
    Integer(String),
    Float(String),
    String(String),
    Bool(bool),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Negate,
    Not,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Assign,
    Range,
    Or,
    And,
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    Add,
    Subtract,
    Multiply,
    Divide,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeRef {
    pub kind: TypeRefKind,
    pub span: Span,
}

impl Spanned for TypeRef {
    fn span(&self) -> Span {
        self.span
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeRefKind {
    Path {
        path: Path,
        arguments: Vec<TypeRef>,
    },
    Array {
        element: Box<TypeRef>,
        length: usize,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Path {
    pub segments: Vec<String>,
    pub segment_spans: Vec<Span>,
    pub span: Span,
}

impl Path {
    pub fn new(segments: Vec<String>, span: Span) -> Self {
        Self {
            segment_spans: vec![span; segments.len()],
            segments,
            span,
        }
    }

    pub fn with_segment_spans(segments: Vec<String>, segment_spans: Vec<Span>, span: Span) -> Self {
        debug_assert_eq!(segments.len(), segment_spans.len());
        Self {
            segments,
            segment_spans,
            span,
        }
    }

    pub fn first(&self) -> Option<&str> {
        self.segments.first().map(String::as_str)
    }

    pub fn last(&self) -> Option<&str> {
        self.segments.last().map(String::as_str)
    }

    pub fn segment_span(&self, index: usize) -> Option<Span> {
        self.segment_spans.get(index).copied()
    }
}

impl Spanned for Path {
    fn span(&self) -> Span {
        self.span
    }
}
