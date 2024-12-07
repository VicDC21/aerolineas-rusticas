//! Módulo para la información de acceso.

/// La información de logueo.
#[derive(Clone, Debug, PartialEq)]
pub struct LoginInfo {
    /// El usuario.
    pub user: String,

    /// La contraseña.
    pub pass: String,
}

impl LoginInfo {
    /// Crea una nueva instancia.
    pub fn new(user: String, pass: String) -> Self {
        Self { user, pass }
    }

    /// Crea una nueva instancia a partir de [str]s.
    pub fn new_str(user: &str, pass: &str) -> Self {
        Self::new(user.to_string(), pass.to_string())
    }

    /// Chequea si la info está vacía.
    pub fn is_empty(&self) -> bool {
        self.user.is_empty() && self.pass.is_empty()
    }
}

impl Default for LoginInfo {
    fn default() -> Self {
        Self::new("".to_string(), "".to_string())
    }
}
