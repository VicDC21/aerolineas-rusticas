use super::{constant::Constant, identifier::Identifier, map_literal::MapLiteral};

// options::= option ( AND option )*
// option::= identifier '=' ( identifier
// 	| constant
// 	| map_literal )
pub enum Option{
    Identifier(Identifier),
    Constant(Constant),
    MapLiteral(MapLiteral)
}