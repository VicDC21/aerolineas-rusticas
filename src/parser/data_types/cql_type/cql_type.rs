use crate::parser::data_types::user_defined_type::UserDefinedType;
use super::collection_type::CollectionType;
use super::native_types::NativeType;
use super::tuple_type::TupleType;

pub enum CQLType {
    NativeType(NativeType),
    CollectionType(CollectionType),
    UserDefinedType(UserDefinedType),
    TupleType(TupleType),
    CustomType(String),
}
