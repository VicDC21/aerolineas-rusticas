use crate::data_types::cql_type::{
    collection_type::CollectionType, native_types::NativeType, tuple_type::TupleType,
};
use protocol::aliases::results::Result;

/// Tipo de dato de CQL.
#[derive(Debug, PartialEq)]
pub enum CQLType {
    /// Tipo de dato nativo.
    NativeType(NativeType),
    /// Tipo de colección.
    CollectionType(CollectionType),
    /// Tipo de tupla.
    TupleType(TupleType),
    /// Tipo de dato personalizado.
    CustomType(String),
}

impl CQLType {
    /// Verifica si la lista de tokens es un tipo de dato de CQL. Si lo es, lo retorna.
    /// Si no lo es, retorna None, o Error en caso de no poder parsearla.
    pub fn check_kind_of_type(list: &mut Vec<String>) -> Result<Option<Self>> {
        if let Some(value) = NativeType::parse_data_type(list)? {
            return Ok(Some(CQLType::NativeType(value)));
        } else if let Some(value) = CollectionType::parse_collection_type(list)? {
            return Ok(Some(CQLType::CollectionType(value)));
        } else if let Some(value) = TupleType::parse_tuple_type(list)? {
            return Ok(Some(CQLType::TupleType(value)));
        } else if let Some(custom_type) = list.first() {
            return Ok(Some(CQLType::CustomType(custom_type.to_string())));
        }
        Ok(None)
    }
}
