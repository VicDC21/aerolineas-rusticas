/// user_defined_type::= udt_name
/// udt_name::= [ keyspace_name '.' ] identifier

pub struct UserDefinedType {
    udt_name: String,
}
