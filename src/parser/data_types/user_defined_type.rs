/// user_defined_type::= udt_name
/// udt_name::= [ keyspace_name '.' ] identifier
use crate::cassandra::errors::error::Error;
pub struct UserDefinedType {
    udt_name: String,
}

impl UserDefinedType {
    // TO CHECK
    pub fn parse_user_defined_type(list: &mut Vec<String>) -> Result<Option<Self>, Error> {
        if let Some(udt_name) = list.pop() {
            Ok(Some(UserDefinedType { udt_name }))
        } else {
            Ok(None)
        }
    }
}
