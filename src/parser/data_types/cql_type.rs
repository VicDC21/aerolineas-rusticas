use super::collection_type::CollectionType;
use super::native_types::NativeType;
use super::tuple_type::TupleType;
use super::user_defined_type::UserDefinedType;

pub enum CQLType {
    NativeType(NativeType),
    CollectionType(CollectionType),
    UserDefinedType(UserDefinedType),
    TupleType(TupleType),
    CustomType(String),
}
