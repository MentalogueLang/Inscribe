use inscribe_ast::nodes::BinaryOp;
use inscribe_lexer::token::TokenKind;

// TODO: Extend precedence handling for tuples and additional operators.

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Precedence {
    Assignment = 1,
    Range = 2,
    Or = 3,
    And = 4,
    Equality = 5,
    Comparison = 6,
    Term = 7,
    Factor = 8,
}

impl Precedence {
    pub const fn level(self) -> u8 {
        self as u8
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Associativity {
    Left,
    Right,
}

pub fn binary_operator(kind: &TokenKind) -> Option<(Precedence, Associativity, BinaryOp)> {
    match kind {
        TokenKind::Equal => Some((
            Precedence::Assignment,
            Associativity::Right,
            BinaryOp::Assign,
        )),
        TokenKind::DotDot => Some((Precedence::Range, Associativity::Left, BinaryOp::Range)),
        TokenKind::OrOr => Some((Precedence::Or, Associativity::Left, BinaryOp::Or)),
        TokenKind::AndAnd => Some((Precedence::And, Associativity::Left, BinaryOp::And)),
        TokenKind::EqualEqual => Some((Precedence::Equality, Associativity::Left, BinaryOp::Equal)),
        TokenKind::BangEqual => Some((
            Precedence::Equality,
            Associativity::Left,
            BinaryOp::NotEqual,
        )),
        TokenKind::Less => Some((Precedence::Comparison, Associativity::Left, BinaryOp::Less)),
        TokenKind::LessEqual => Some((
            Precedence::Comparison,
            Associativity::Left,
            BinaryOp::LessEqual,
        )),
        TokenKind::Greater => Some((
            Precedence::Comparison,
            Associativity::Left,
            BinaryOp::Greater,
        )),
        TokenKind::GreaterEqual => Some((
            Precedence::Comparison,
            Associativity::Left,
            BinaryOp::GreaterEqual,
        )),
        TokenKind::Plus => Some((Precedence::Term, Associativity::Left, BinaryOp::Add)),
        TokenKind::Minus => Some((Precedence::Term, Associativity::Left, BinaryOp::Subtract)),
        TokenKind::Star => Some((Precedence::Factor, Associativity::Left, BinaryOp::Multiply)),
        TokenKind::Slash => Some((Precedence::Factor, Associativity::Left, BinaryOp::Divide)),
        _ => None,
    }
}
