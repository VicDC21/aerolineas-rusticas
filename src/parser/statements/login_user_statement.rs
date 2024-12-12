use crate::{
    parser::statements::ddl_statement::ddl_statement_parser::check_words,
    protocol::{aliases::results::Result, errors::error::Error},
};

/// Representa una sentencia de un login de usuario.
#[derive(Debug)]
pub struct LoginUserStatement {
    /// El usuario del login
    pub user: String,
    /// La contraseña del usuario
    pub password: String,
}

/// Verifica si la lista dada es una sentencia de un login de usuario. Si lo es, lo retorna, si no, retorna None.
/// Devuelve error si hay campos faltantes o errores de sintaxis.
pub fn login_statement(lista: &mut Vec<String>) -> Result<Option<LoginUserStatement>> {
    let mut login = LoginUserStatement {
        user: "".to_string(),
        password: "".to_string(),
    };
    if !check_words(lista, "User :") {
        return Ok(None);
    }
    if !lista.is_empty() {
        let user = lista.remove(0);
        login.user = user;
    }
    if !check_words(lista, "Password :") {
        return Err(Error::Invalid(
            "Falta la contraseña al momento de loguearse".to_string(),
        ));
    }
    if !lista.is_empty() {
        let password = lista.remove(0);
        login.password = password;
    } else {
        return Err(Error::Invalid(
            "Falta la contraseña al momento de loguearse".to_string(),
        ));
    }
    Ok(Some(login))
}
