use super::collection_type::CollectionType;
use super::native_types::NativeType;
use super::tuple_type::TupleType;
use crate::cassandra::errors::error::Error;
use crate::parser::data_types::user_defined_type::UserDefinedType;

pub enum CQLType {
    NativeType(NativeType),
    CollectionType(CollectionType),
    UserDefinedType(UserDefinedType),
    TupleType(TupleType),
    CustomType(String),
}

impl CQLType{
    pub fn check_kind_of_type(list: &mut Vec<String>) -> Result<Option<Self>, Error>{
        if let Some(value) = NativeType::parse_data_type(list)?{
            return Ok(Some(CQLType::NativeType(value)))
        };

        Ok(None)
    }
}