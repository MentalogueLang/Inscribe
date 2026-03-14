use inscribe_ast::span::Span;
use inscribe_typeck::{FunctionSignature, Type};

// TODO: Grow this into a canonical compiler IR with stable ids instead of source-driven names.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HirProgram {
    pub items: Vec<HirItem>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HirItem {
    Import(HirImport),
    Struct(HirStruct),
    Function(HirFunction),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HirImport {
    pub path: Vec<String>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HirStruct {
    pub name: String,
    pub fields: Vec<HirField>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HirField {
    pub name: String,
    pub ty: Type,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HirFunction {
    pub receiver: Option<String>,
    pub name: String,
    pub signature: FunctionSignature,
    pub params: Vec<HirParam>,
    pub is_declaration: bool,
    pub body: Option<HirBlock>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HirParam {
    pub name: String,
    pub ty: Type,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HirBlock {
    pub statements: Vec<HirStmt>,
    pub ty: Type,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HirStmt {
    Let(HirBinding),
    Const(HirBinding),
    For(HirFor),
    While(HirWhile),
    Return(Option<HirExpr>, Span),
    Expr(HirExpr),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HirBinding {
    pub name: String,
    pub ty: Type,
    pub value: HirExpr,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HirFor {
    pub binding: String,
    pub binding_ty: Type,
    pub iterable: HirExpr,
    pub body: HirBlock,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HirWhile {
    pub condition: HirExpr,
    pub body: HirBlock,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HirExpr {
    pub kind: HirExprKind,
    pub ty: Type,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HirExprKind {
    Literal(String),
    Path(Vec<String>),
    Unary {
        op: String,
        expr: Box<HirExpr>,
    },
    Binary {
        op: String,
        left: Box<HirExpr>,
        right: Box<HirExpr>,
    },
    Call {
        callee: Box<HirExpr>,
        args: Vec<HirExpr>,
    },
    Field {
        base: Box<HirExpr>,
        field: String,
    },
    StructLiteral {
        path: Vec<String>,
        fields: Vec<(String, HirExpr)>,
    },
    If {
        condition: Box<HirExpr>,
        then_block: HirBlock,
        else_branch: Option<Box<HirExpr>>,
    },
    Match {
        value: Box<HirExpr>,
        arms: Vec<HirMatchArm>,
    },
    Block(HirBlock),
    Try(Box<HirExpr>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HirMatchArm {
    pub pattern: String,
    pub value: HirExpr,
    pub span: Span,
}
