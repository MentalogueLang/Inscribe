pub mod cursor;
pub mod literals;
pub mod token;

use std::fmt;

use crate::cursor::Cursor;
use crate::literals::{lex_number, lex_string};
use crate::token::{Span, Token, TokenKind};

// TODO: Add file identifiers and structured diagnostics once session plumbing is in place.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexError {
    pub message: String,
    pub span: Span,
}

impl LexError {
    pub fn new(message: impl Into<String>, span: Span) -> Self {
        Self {
            message: message.into(),
            span,
        }
    }
}

impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} at line {}, column {}",
            self.message, self.span.start.line, self.span.start.column
        )
    }
}

impl std::error::Error for LexError {}

pub fn lex(source: &str) -> Result<Vec<Token>, LexError> {
    let mut cursor = Cursor::new(source);
    let mut tokens = Vec::new();

    while !cursor.is_eof() {
        match cursor.peek() {
            Some(' ' | '\t') => {
                let _ = cursor.bump();
            }
            Some('\n' | '\r') => {
                let start = cursor.position();
                let _ = cursor.consume_newline();
                tokens.push(Token::new(
                    TokenKind::Newline,
                    Span::new(start, cursor.position()),
                ));
            }
            Some('/') if cursor.peek_next() == Some('/') => {
                let _ = cursor.bump();
                let _ = cursor.bump();
                while !matches!(cursor.peek(), Some('\n' | '\r') | None) {
                    let _ = cursor.bump();
                }
            }
            Some('/') if cursor.peek_next() == Some('*') => {
                let start = cursor.position();
                let _ = cursor.bump();
                let _ = cursor.bump();
                loop {
                    match (cursor.peek(), cursor.peek_next()) {
                        (Some('*'), Some('/')) => {
                            let _ = cursor.bump();
                            let _ = cursor.bump();
                            break;
                        }
                        (Some('\n' | '\r'), _) => {
                            let _ = cursor.consume_newline();
                        }
                        (Some(_), _) => {
                            let _ = cursor.bump();
                        }
                        (None, _) => {
                            return Err(LexError::new(
                                "unterminated block comment",
                                Span::new(start, cursor.position()),
                            ));
                        }
                    }
                }
            }
            Some(ch) if is_ident_start(ch) => {
                let start = cursor.position();
                let start_offset = start.offset;
                let _ = cursor.bump();
                while matches!(cursor.peek(), Some(next) if is_ident_continue(next)) {
                    let _ = cursor.bump();
                }
                let text = cursor.slice(start_offset, cursor.position().offset);
                let kind = match text {
                    "let" => TokenKind::Let,
                    "const" => TokenKind::Const,
                    "fn" => TokenKind::Fn,
                    "struct" => TokenKind::Struct,
                    "enum" => TokenKind::Enum,
                    "if" => TokenKind::If,
                    "else" => TokenKind::Else,
                    "for" => TokenKind::For,
                    "in" => TokenKind::In,
                    "while" => TokenKind::While,
                    "return" => TokenKind::Return,
                    "import" => TokenKind::Import,
                    "priv" => TokenKind::Private,
                    "match" => TokenKind::Match,
                    "true" => TokenKind::True,
                    "false" => TokenKind::False,
                    _ => TokenKind::Identifier(text.to_string()),
                };
                tokens.push(Token::new(kind, Span::new(start, cursor.position())));
            }
            Some(ch) if ch.is_ascii_digit() => {
                let start = cursor.position();
                tokens.push(lex_number(&mut cursor, start));
            }
            Some('"') => {
                let start = cursor.position();
                tokens.push(lex_string(&mut cursor, start)?);
            }
            Some(_) => {
                let start = cursor.position();
                let token = lex_punctuation(&mut cursor).ok_or_else(|| {
                    LexError::new("unexpected character", Span::new(start, cursor.position()))
                })?;
                tokens.push(token);
            }
            None => break,
        }
    }

    let position = cursor.position();
    tokens.push(Token::new(TokenKind::Eof, Span::new(position, position)));
    Ok(tokens)
}

fn lex_punctuation(cursor: &mut Cursor<'_>) -> Option<Token> {
    let start = cursor.position();
    let kind = match (cursor.peek()?, cursor.peek_next()) {
        ('(', _) => {
            let _ = cursor.bump();
            TokenKind::LParen
        }
        (')', _) => {
            let _ = cursor.bump();
            TokenKind::RParen
        }
        ('[', _) => {
            let _ = cursor.bump();
            TokenKind::LBracket
        }
        (']', _) => {
            let _ = cursor.bump();
            TokenKind::RBracket
        }
        ('{', _) => {
            let _ = cursor.bump();
            TokenKind::LBrace
        }
        ('}', _) => {
            let _ = cursor.bump();
            TokenKind::RBrace
        }
        (',', _) => {
            let _ = cursor.bump();
            TokenKind::Comma
        }
        (':', _) => {
            let _ = cursor.bump();
            TokenKind::Colon
        }
        (';', _) => {
            let _ = cursor.bump();
            TokenKind::Semicolon
        }
        ('.', Some('.')) => {
            let _ = cursor.bump();
            let _ = cursor.bump();
            TokenKind::DotDot
        }
        ('.', _) => {
            let _ = cursor.bump();
            TokenKind::Dot
        }
        ('-', Some('>')) => {
            let _ = cursor.bump();
            let _ = cursor.bump();
            TokenKind::Arrow
        }
        ('=', Some('>')) => {
            let _ = cursor.bump();
            let _ = cursor.bump();
            TokenKind::FatArrow
        }
        ('?', _) => {
            let _ = cursor.bump();
            TokenKind::Question
        }
        ('+', _) => {
            let _ = cursor.bump();
            TokenKind::Plus
        }
        ('-', _) => {
            let _ = cursor.bump();
            TokenKind::Minus
        }
        ('*', _) => {
            let _ = cursor.bump();
            TokenKind::Star
        }
        ('/', _) => {
            let _ = cursor.bump();
            TokenKind::Slash
        }
        ('=', Some('=')) => {
            let _ = cursor.bump();
            let _ = cursor.bump();
            TokenKind::EqualEqual
        }
        ('=', _) => {
            let _ = cursor.bump();
            TokenKind::Equal
        }
        ('!', Some('=')) => {
            let _ = cursor.bump();
            let _ = cursor.bump();
            TokenKind::BangEqual
        }
        ('!', _) => {
            let _ = cursor.bump();
            TokenKind::Bang
        }
        ('<', Some('=')) => {
            let _ = cursor.bump();
            let _ = cursor.bump();
            TokenKind::LessEqual
        }
        ('<', _) => {
            let _ = cursor.bump();
            TokenKind::Less
        }
        ('>', Some('=')) => {
            let _ = cursor.bump();
            let _ = cursor.bump();
            TokenKind::GreaterEqual
        }
        ('>', _) => {
            let _ = cursor.bump();
            TokenKind::Greater
        }
        ('&', Some('&')) => {
            let _ = cursor.bump();
            let _ = cursor.bump();
            TokenKind::AndAnd
        }
        ('|', Some('|')) => {
            let _ = cursor.bump();
            let _ = cursor.bump();
            TokenKind::OrOr
        }
        _ => return None,
    };

    Some(Token::new(kind, Span::new(start, cursor.position())))
}

fn is_ident_start(ch: char) -> bool {
    ch == '_' || ch.is_ascii_alphabetic()
}

fn is_ident_continue(ch: char) -> bool {
    ch == '_' || ch.is_ascii_alphanumeric()
}

#[cfg(test)]
mod tests {
    use super::lex;
    use crate::token::TokenKind;

    #[test]
    fn lexes_core_language_tokens() {
        let tokens = lex(
            "import io.file\nfn main() {\n    let user = User { name: \"Antonio\" }\n    data = readFile(\"data.txt\") ?\n}\n",
        )
        .expect("lexing should succeed");

        assert!(tokens
            .iter()
            .any(|token| matches!(token.kind, TokenKind::Import)));
        assert!(tokens
            .iter()
            .any(|token| matches!(token.kind, TokenKind::Fn)));
        assert!(tokens
            .iter()
            .any(|token| matches!(token.kind, TokenKind::Question)));
    }

    #[test]
    fn skips_comments_and_keeps_newlines() {
        let tokens = lex("// hello\nlet value = 1\n/* block\ncomment */\nvalue\n")
            .expect("lexing should succeed");

        let newline_count = tokens
            .iter()
            .filter(|token| matches!(token.kind, TokenKind::Newline))
            .count();
        assert!(newline_count >= 3);
    }
}
