use crate::parser::data_types::literal::{
    collection_literal::CollectionLiteral, tuple_literal::TupleLiteral,
};

/// Literal de CQL.
///
/// collection_literal | tuple_literal
pub enum Literal {
    /// Literal de una colección.
    CollectionLiteral(CollectionLiteral),
    /// Literal de una tupla.
    TupleLiteral(TupleLiteral),
}
