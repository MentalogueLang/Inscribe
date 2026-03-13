use inscribe_lexer::token::TokenKind;

use crate::item::starts_item;
use crate::stmt::is_statement_boundary;

// TODO: Replace this with richer synchronization sets when the parser starts collecting errors.

pub fn is_recovery_boundary(kind: &TokenKind) -> bool {
    starts_item(kind) || is_statement_boundary(kind)
}
