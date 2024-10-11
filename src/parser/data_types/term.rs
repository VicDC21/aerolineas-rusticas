use super::constant::Constant;
use super::literal::Literal;
use super::type_hint::TypeHint;

/// constant | literal | arithmetic_operation | type_hint | bind_marker

pub enum Term {
    /// string | integer | float | boolean | uuid | blob | NULL
    Constant(Constant),

    // collection_literal | vector_literal | udt_literal | tuple_literal
    // Literal(Literal),

    // '-' term | term ('+' | '-' | '*' | '/' | '%') term
    // AritmethicOperation,

    // '(' cql_type ')' term
    // TypeHint(TypeHint),

    // '?' | ':' identifier
    // BindMarker,
}
