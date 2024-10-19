use crate::protocol::errors::error::Error;

#[allow(dead_code)]
/// user_defined_type::= udt_name
/// udt_name::= [ keyspace_name '.' ] identifier
#[derive(Debug)]
pub struct UserDefinedType {
    /// TODO: Desc básica
    udt_name: String,
}

impl UserDefinedType {
    // TO CHECK
    /// TODO: Desc básica
    pub fn parse_user_defined_type(list: &mut Vec<String>) -> Result<Option<Self>, Error> {
        if let Some(udt_name) = list.pop() {
            Ok(Some(UserDefinedType { udt_name }))
        } else {
            Ok(None)
        }
    }
}
