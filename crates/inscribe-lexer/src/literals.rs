use crate::cursor::Cursor;
use crate::token::{Span, Token, TokenKind};
use crate::LexError;

// TODO: Support numeric suffixes, raw strings, and richer escape forms.

pub fn lex_number(cursor: &mut Cursor<'_>, start: crate::token::Position) -> Token {
    let start_offset = start.offset;
    while matches!(cursor.peek(), Some(ch) if ch.is_ascii_digit()) {
        let _ = cursor.bump();
    }

    let kind =
        if cursor.peek() == Some('.') && cursor.peek_next().is_some_and(|ch| ch.is_ascii_digit()) {
            let _ = cursor.bump();
            while matches!(cursor.peek(), Some(ch) if ch.is_ascii_digit()) {
                let _ = cursor.bump();
            }
            TokenKind::Float(
                cursor
                    .slice(start_offset, cursor.position().offset)
                    .to_string(),
            )
        } else {
            TokenKind::Integer(
                cursor
                    .slice(start_offset, cursor.position().offset)
                    .to_string(),
            )
        };

    Token::new(kind, Span::new(start, cursor.position()))
}

pub fn lex_string(
    cursor: &mut Cursor<'_>,
    start: crate::token::Position,
) -> Result<Token, LexError> {
    let _ = cursor.bump();
    let mut value = String::new();

    while let Some(ch) = cursor.peek() {
        match ch {
            '"' => {
                let _ = cursor.bump();
                return Ok(Token::new(
                    TokenKind::String(value),
                    Span::new(start, cursor.position()),
                ));
            }
            '\\' => {
                let _ = cursor.bump();
                let escaped = match cursor.bump() {
                    Some('n') => '\n',
                    Some('r') => '\r',
                    Some('t') => '\t',
                    Some('"') => '"',
                    Some('\\') => '\\',
                    Some(other) => {
                        return Err(LexError::new(
                            format!("unsupported escape sequence `\\{other}`"),
                            Span::new(start, cursor.position()),
                        ));
                    }
                    None => {
                        return Err(LexError::new(
                            "unterminated string escape",
                            Span::new(start, cursor.position()),
                        ));
                    }
                };
                value.push(escaped);
            }
            '\n' | '\r' => {
                return Err(LexError::new(
                    "unterminated string literal",
                    Span::new(start, cursor.position()),
                ));
            }
            _ => {
                value.push(ch);
                let _ = cursor.bump();
            }
        }
    }

    Err(LexError::new(
        "unterminated string literal",
        Span::new(start, cursor.position()),
    ))
}
