use crate::token::Position;

// TODO: Replace this with an interned source map cursor once files and includes are wired in.

#[derive(Debug, Clone)]
pub struct Cursor<'a> {
    source: &'a str,
    index: usize,
    line: usize,
    column: usize,
}

impl<'a> Cursor<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            index: 0,
            line: 1,
            column: 1,
        }
    }

    pub fn is_eof(&self) -> bool {
        self.index >= self.source.len()
    }

    pub fn position(&self) -> Position {
        Position::new(self.index, self.line, self.column)
    }

    pub fn peek(&self) -> Option<char> {
        self.source[self.index..].chars().next()
    }

    pub fn peek_next(&self) -> Option<char> {
        let mut chars = self.source[self.index..].chars();
        let _ = chars.next()?;
        chars.next()
    }

    pub fn bump(&mut self) -> Option<char> {
        let ch = self.peek()?;
        self.index += ch.len_utf8();
        if ch == '\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
        Some(ch)
    }

    pub fn consume_newline(&mut self) -> bool {
        match self.peek() {
            Some('\n') => {
                self.index += 1;
                self.line += 1;
                self.column = 1;
                true
            }
            Some('\r') => {
                self.index += 1;
                if self.peek() == Some('\n') {
                    self.index += 1;
                }
                self.line += 1;
                self.column = 1;
                true
            }
            _ => false,
        }
    }

    pub fn slice(&self, start: usize, end: usize) -> &'a str {
        &self.source[start..end]
    }
}
