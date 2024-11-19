use super::super::udt_literal::UdtLiteral;
use super::collection_literal::CollectionLiteral;
use super::tuple_literal::TupleLiteral;
use super::vector_literal::VectorLiteral;

/// Literal de CQL.
///
/// collection_literal | vector_literal | udt_literal | tuple_literal
pub enum Literal {
    /// Literal de una colección.
    CollectionLiteral(CollectionLiteral),
    /// Literal de un vector.
    VectorLiteral(VectorLiteral),
    /// Literal de tipo _UDT_.
    UdtLiteral(UdtLiteral),
    /// Literal de una tupla.
    TupleLiteral(TupleLiteral),
}
