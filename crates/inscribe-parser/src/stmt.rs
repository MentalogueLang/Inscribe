use inscribe_lexer::token::TokenKind;

// TODO: Teach statement detection about declarations introduced by future language features.

pub fn starts_statement(kind: &TokenKind) -> bool {
    matches!(
        kind,
        TokenKind::Let
            | TokenKind::Const
            | TokenKind::For
            | TokenKind::While
            | TokenKind::Return
            | TokenKind::If
            | TokenKind::Match
            | TokenKind::LBrace
            | TokenKind::Identifier(_)
            | TokenKind::Integer(_)
            | TokenKind::Float(_)
            | TokenKind::String(_)
            | TokenKind::True
            | TokenKind::False
            | TokenKind::LParen
            | TokenKind::Bang
            | TokenKind::Minus
    )
}

pub fn is_statement_boundary(kind: &TokenKind) -> bool {
    matches!(
        kind,
        TokenKind::Newline | TokenKind::Semicolon | TokenKind::RBrace | TokenKind::Eof
    )
}
