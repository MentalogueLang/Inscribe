use inscribe_ast::span::Span;
use inscribe_typeck::{FunctionSignature, Type};
use serde::{Deserialize, Serialize};

// TODO: Introduce stable ids and richer provenance once optimization passes begin mutating MIR.

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct MirProgram {
    pub functions: Vec<MirFunction>,
    pub span: Span,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BasicBlockId(pub usize);

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LocalId(pub usize);

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct MirFunction {
    pub receiver: Option<String>,
    pub name: String,
    pub signature: FunctionSignature,
    pub is_declaration: bool,
    pub locals: Vec<LocalDecl>,
    pub blocks: Vec<BasicBlockData>,
    pub entry: BasicBlockId,
    pub return_local: LocalId,
    pub span: Span,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct LocalDecl {
    pub id: LocalId,
    pub name: String,
    pub ty: Type,
    pub mutable: bool,
    pub temp: bool,
    pub span: Span,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct BasicBlockData {
    pub id: BasicBlockId,
    pub statements: Vec<Statement>,
    pub terminator: TerminatorKind,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Statement {
    pub kind: StatementKind,
    pub span: Span,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum StatementKind {
    StorageLive(LocalId),
    StorageDead(LocalId),
    Assign(Place, Rvalue),
    Drop(LocalId),
    Nop,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Place {
    pub local: LocalId,
    pub projection: Vec<ProjectionElem>,
}

impl Place {
    pub fn new(local: LocalId) -> Self {
        Self {
            local,
            projection: Vec::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum ProjectionElem {
    Field(String),
    Index(Operand),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum Rvalue {
    Use(Operand),
    UnaryOp {
        op: String,
        operand: Operand,
    },
    BinaryOp {
        op: String,
        left: Operand,
        right: Operand,
    },
    AggregateStruct {
        path: Vec<String>,
        fields: Vec<(String, Operand)>,
    },
    AggregateArray {
        elements: Vec<Operand>,
    },
    RepeatArray {
        value: Operand,
        length: usize,
    },
    ResultOk(Operand),
    ResultErr(Operand),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum Operand {
    Copy(Place),
    Move(Place),
    Constant(Constant),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Constant {
    pub ty: Type,
    pub value: ConstantValue,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum ConstantValue {
    Unit,
    Integer(String),
    Float(String),
    String(String),
    Bool(bool),
    Function(String),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum TerminatorKind {
    Goto {
        target: BasicBlockId,
    },
    Branch {
        condition: Operand,
        then_bb: BasicBlockId,
        else_bb: BasicBlockId,
    },
    Match {
        discriminant: Operand,
        arms: Vec<MatchTarget>,
        otherwise: BasicBlockId,
    },
    Call {
        callee: Operand,
        args: Vec<Operand>,
        destination: Option<Place>,
        target: BasicBlockId,
    },
    IterNext {
        iterator: Place,
        binding: LocalId,
        loop_body: BasicBlockId,
        exit: BasicBlockId,
    },
    Try {
        operand: Operand,
        ok_local: LocalId,
        err_local: LocalId,
        ok_target: BasicBlockId,
        err_target: BasicBlockId,
    },
    Return,
    Unreachable,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct MatchTarget {
    pub pattern: String,
    pub target: BasicBlockId,
}
