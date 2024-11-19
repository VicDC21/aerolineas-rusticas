use crate::protocol::errors::error::Error;

#[allow(dead_code)]
/// Tipo definido por el usuario.
///
/// user_defined_type::= udt_name
///
/// udt_name::= [ keyspace_name '.' ] identifier
#[derive(Debug, PartialEq)]
pub struct UserDefinedType {
    /// Nombre del tipo de usuario.
    udt_name: String,
}

impl UserDefinedType {
    // TODO: TO CHECK
    /// Verifica si la lista de tokens es un tipo de usuario. Si lo es, lo retorna.
    /// Si no lo es, retorna None.
    pub fn parse_user_defined_type(list: &mut Vec<String>) -> Result<Option<Self>, Error> {
        if let Some(udt_name) = list.pop() {
            Ok(Some(UserDefinedType { udt_name }))
        } else {
            Ok(None)
        }
    }
}
