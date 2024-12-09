//! Módulo para info de conexión.

use std::sync::{Arc, Mutex};

use crate::{
    client::cli::{get_client_connection, Client, TlsStream},
    data::login_info::LoginInfo,
    protocol::{aliases::results::Result, errors::error::Error},
};
/// Estructura que guarda información de la conexión a un nodo.
pub struct ConnectionHolder {
    /// El cliente.
    client: Arc<Mutex<Client>>,

    /// El _TLS Stream_.
    pub tls_stream: TlsStream,
}

impl ConnectionHolder {
    /// Crea una nueva instancia a partir de un cliente.
    pub fn with_cli(mut client: Client, consistency_lvl: &str) -> Result<Self> {
        client.set_consistency_level(consistency_lvl)?;
        let cli_con = get_client_connection()?;
        let tcp_stream = client.connect()?;
        let tls_stream = client.create_tls_connection(cli_con, tcp_stream)?;

        Ok(Self {
            client: Arc::new(Mutex::new(client)),
            tls_stream,
        })
    }

    /// Consigue una referencia al cliente.
    pub fn get_cli(&self) -> Arc<Mutex<Client>> {
        Arc::clone(&self.client)
    }

    /// Se loguea con el usuario y contraseña dados.
    pub fn login(&mut self, login_info: &LoginInfo) -> Result<()> {
        match self.client.lock() {
            Err(poison_err) => {
                self.client.clear_poison();
                Err(Error::ServerError(format!(
                    "Error de lock envenenado:\n\n{}",
                    poison_err
                )))
            }
            Ok(mut client) => {
                let mut new_tls_opt = client.login(login_info.to_owned(), &mut self.tls_stream)?;
                if let Some(new_tls) = new_tls_opt.take() {
                    self.tls_stream = new_tls;
                }
                Ok(())
            }
        }
    }
}
