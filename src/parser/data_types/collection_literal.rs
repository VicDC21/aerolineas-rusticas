use super::set_literal::SetLiteral;
use super::list_literal::ListLiteral;
use super::map_literal::MapLiteral;


pub enum CollectionLiteral {
    /// MAP '<' cql_type',' cql_type'>'
    MapLiteral(MapLiteral),

    /// SET '<' cql_type '>'
    SetLiteral(SetLiteral),

    /// LIST '<' cql_type'>'
    ListLiteral(ListLiteral),
}