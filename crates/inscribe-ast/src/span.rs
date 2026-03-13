// TODO: Expand source tracking as the compiler gains file tables and macro spans.

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

    pub const fn empty(position: Position) -> Self {
        Self {
            start: position,
            end: position,
        }
    }

    pub fn join(self, other: Span) -> Self {
        Self {
            start: self.start,
            end: other.end,
        }
    }
}

pub trait Spanned {
    fn span(&self) -> Span;
}
