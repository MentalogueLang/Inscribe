use crate::span::{Span, Spanned};

// TODO: Grow these nodes as semantic analysis and lowering need richer syntax data.

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
    Function(FunctionDecl),
}

impl Spanned for Item {
    fn span(&self) -> Span {
        match self {
            Self::Import(item) => item.span,
            Self::Struct(item) => item.span,
            Self::Function(item) => item.span,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Import {
    pub path: Path,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructDecl {
    pub name: String,
    pub fields: Vec<StructField>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructField {
    pub name: String,
    pub ty: TypeRef,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionDecl {
    pub receiver: Option<Path>,
    pub name: String,
    pub params: Vec<Param>,
    pub return_type: Option<TypeRef>,
    pub body: Option<Block>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Param {
    pub name: String,
    pub ty: Option<TypeRef>,
    pub span: Span,
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
    pub ty: Option<TypeRef>,
    pub value: Expr,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConstStmt {
    pub name: String,
    pub ty: Option<TypeRef>,
    pub value: Expr,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ForStmt {
    pub pattern: Pattern,
    pub iterable: Expr,
    pub body: Block,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WhileStmt {
    pub condition: Expr,
    pub body: Block,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReturnStmt {
    pub value: Option<Expr>,
    pub span: Span,
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
    pub value: Expr,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub value: Expr,
    pub span: Span,
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
    pub path: Path,
    pub arguments: Vec<TypeRef>,
    pub span: Span,
}

impl Spanned for TypeRef {
    fn span(&self) -> Span {
        self.span
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Path {
    pub segments: Vec<String>,
    pub span: Span,
}

impl Path {
    pub fn new(segments: Vec<String>, span: Span) -> Self {
        Self { segments, span }
    }
}

impl Spanned for Path {
    fn span(&self) -> Span {
        self.span
    }
}
