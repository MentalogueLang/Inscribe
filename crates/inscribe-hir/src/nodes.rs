use inscribe_ast::Visibility;
use inscribe_ast::span::Span;
use inscribe_typeck::{FunctionSignature, Type};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HirSymbolId(pub usize);

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum HirSymbolKind {
    Import,
    Struct,
    Enum,
    Variant,
    Function,
    Field,
    Param,
    Local,
    Unresolved,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct HirSymbol {
    pub id: HirSymbolId,
    pub name: String,
    pub kind: HirSymbolKind,
    pub span: Span,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct HirProgram {
    pub items: Vec<HirItem>,
    pub symbols: Vec<HirSymbol>,
    pub span: Span,
}

impl HirProgram {
    pub fn symbol(&self, id: HirSymbolId) -> &HirSymbol {
        &self.symbols[id.0]
    }

    pub fn symbol_name(&self, id: HirSymbolId) -> &str {
        &self.symbol(id).name
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum HirItem {
    Import(HirImport),
    Struct(HirStruct),
    Enum(HirEnum),
    Function(HirFunction),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct HirImport {
    pub symbol: HirSymbolId,
    pub path: Vec<String>,
    pub span: Span,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct HirStruct {
    pub symbol: HirSymbolId,
    pub fields: Vec<HirField>,
    pub span: Span,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct HirField {
    pub symbol: HirSymbolId,
    pub ty: Type,
    pub span: Span,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct HirEnum {
    pub symbol: HirSymbolId,
    pub variants: Vec<HirEnumVariant>,
    pub span: Span,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct HirEnumVariant {
    pub symbol: HirSymbolId,
    pub discriminant: usize,
    pub span: Span,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct HirFunction {
    pub symbol: HirSymbolId,
    pub visibility: Visibility,
    pub receiver: Option<HirSymbolId>,
    pub signature: FunctionSignature,
    pub params: Vec<HirParam>,
    pub is_declaration: bool,
    pub body: Option<HirBlock>,
    pub span: Span,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct HirParam {
    pub symbol: HirSymbolId,
    pub ty: Type,
    pub span: Span,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct HirBlock {
    pub statements: Vec<HirStmt>,
    pub ty: Type,
    pub span: Span,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum HirStmt {
    Let(HirBinding),
    Const(HirBinding),
    For(HirFor),
    While(HirWhile),
    Return(Option<HirExpr>, Span),
    Expr(HirExpr),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct HirBinding {
    pub symbol: HirSymbolId,
    pub ty: Type,
    pub value: HirExpr,
    pub span: Span,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct HirFor {
    pub binding: HirSymbolId,
    pub binding_ty: Type,
    pub iterable: HirExpr,
    pub body: HirBlock,
    pub span: Span,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct HirWhile {
    pub condition: HirExpr,
    pub body: HirBlock,
    pub span: Span,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct HirExpr {
    pub kind: HirExprKind,
    pub ty: Type,
    pub span: Span,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum HirExprKind {
    Literal(String),
    EnumVariant {
        enum_id: HirSymbolId,
        variant_id: HirSymbolId,
        discriminant: usize,
    },
    Path(HirSymbolId),
    Array(Vec<HirExpr>),
    RepeatArray {
        value: Box<HirExpr>,
        length: usize,
    },
    Cast {
        expr: Box<HirExpr>,
    },
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
        field: HirSymbolId,
    },
    Index {
        target: Box<HirExpr>,
        index: Box<HirExpr>,
    },
    StructLiteral {
        struct_id: HirSymbolId,
        fields: Vec<(HirSymbolId, HirExpr)>,
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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct HirMatchArm {
    pub pattern: String,
    pub value: HirExpr,
    pub span: Span,
}
