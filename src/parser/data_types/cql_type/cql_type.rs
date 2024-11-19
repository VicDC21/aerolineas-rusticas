use super::collection_type::CollectionType;
use super::native_types::NativeType;
use super::tuple_type::TupleType;
use crate::parser::data_types::user_defined_type::UserDefinedType;
use crate::protocol::errors::error::Error;

/// Tipo de dato de CQL.
#[derive(Debug, PartialEq)]
pub enum CQLType {
    /// Tipo de dato nativo.
    NativeType(NativeType),
    /// Tipo de colección.
    CollectionType(CollectionType),
    /// Tipo de dato definido por el usuario.
    UserDefinedType(UserDefinedType),
    /// Tipo de tupla.
    TupleType(TupleType),
    /// Tipo de dato personalizado.
    CustomType(String),
}

impl CQLType {
    /// Verifica si la lista de tokens es un tipo de dato de CQL. Si lo es, lo retorna.
    /// Si no lo es, retorna None, o Error en caso de no poder parsearla.
    pub fn check_kind_of_type(list: &mut Vec<String>) -> Result<Option<Self>, Error> {
        if let Some(value) = NativeType::parse_data_type(list)? {
            return Ok(Some(CQLType::NativeType(value)));
        } else if let Some(value) = CollectionType::parse_collection_type(list)? {
            return Ok(Some(CQLType::CollectionType(value)));
        } else if let Some(value) = UserDefinedType::parse_user_defined_type(list)? {
            return Ok(Some(CQLType::UserDefinedType(value)));
        } else if let Some(value) = TupleType::parse_tuple_type(list)? {
            return Ok(Some(CQLType::TupleType(value)));
        } else if let Some(custom_type) = list.first() {
            return Ok(Some(CQLType::CustomType(custom_type.to_string())));
        }
        Ok(None)
    }
}
