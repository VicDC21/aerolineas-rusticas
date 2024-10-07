pub enum Term {
    /// constant | literal | function_call | arithmetic_operation | type_hint | bind_marker
    Term,

    /// collection_literal | vector_literal | udt_literal | tuple_literal
    Literal,

    /// identifier '(' [ term (',' term)* ] ')'
    FunctionCall,

    /// '-' term | term ('+' | '-' | '*' | '/' | '%') term
    AritmethicOperation,

    /// '(' cql_type ')' term
    TypeHint,

    /// '?' | ':' identifier
    BindMaker,
}
