use crate::parser::data_types::literal::{list_literal::ListLiteral, map_literal::MapLiteral};

/// Literal de una colección.
pub enum CollectionLiteral {
    /// MAP '<' cql_type',' cql_type'>'
    MapLiteral(MapLiteral),

    /// LIST '<' cql_type'>'
    ListLiteral(ListLiteral),
}
