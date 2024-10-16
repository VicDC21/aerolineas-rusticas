use super::super::udt_literal::UdtLiteral;
use super::collection_literal::CollectionLiteral;
use super::tuple_literal::TupleLiteral;
use super::vector_literal::VectorLiteral;

/// collection_literal | vector_literal | udt_literal | tuple_literal
pub enum Literal {
    /// TODO: Desc básica
    CollectionLiteral(CollectionLiteral),
    /// TODO: Desc básica
    VectorLiteral(VectorLiteral),
    /// TODO: Desc básica
    UdtLiteral(UdtLiteral),
    /// TODO: Desc básica
    TupleLiteral(TupleLiteral),
}
