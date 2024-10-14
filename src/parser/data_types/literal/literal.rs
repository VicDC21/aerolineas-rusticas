use super::collection_literal::CollectionLiteral;
use super::tuple_literal::TupleLiteral;
use super::super::udt_literal::UdtLiteral;
use super::vector_literal::VectorLiteral;

/// collection_literal | vector_literal | udt_literal | tuple_literal
pub enum Literal {
    CollectionLiteral(CollectionLiteral),
    VectorLiteral(VectorLiteral),
    UdtLiteral(UdtLiteral),
    TupleLiteral(TupleLiteral),
}
