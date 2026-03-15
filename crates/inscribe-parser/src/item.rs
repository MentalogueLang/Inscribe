use inscribe_lexer::token::TokenKind;

// TODO: Expand item starts once traits and impl blocks land.

pub fn starts_item(kind: &TokenKind) -> bool {
    matches!(
        kind,
        TokenKind::Import
            | TokenKind::Struct
            | TokenKind::Enum
            | TokenKind::Fn
            | TokenKind::Private
    )
}
