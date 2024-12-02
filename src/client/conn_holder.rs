//! Módulo para info de conexión.

use std::sync::{Arc, Mutex};

use crate::{
    client::{
        cli::{get_client_connection, Client, TlsStream},
        protocol_result::ProtocolResult,
    },
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
    pub fn with_cli(mut client: Client) -> Result<Self> {
        client.set_consistency_level("Quorum")?;
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
    pub fn login(&mut self, user: &str, password: &str) -> Result<()> {
        match self.client.lock() {
            Err(poison_err) => {
                self.client.clear_poison();
                Err(Error::ServerError(format!(
                    "Error de lock envenenado:\n\n{}",
                    poison_err
                )))
            }
            Ok(mut client) => {
                let protocol_result = client.send_query(
                    format!("User: {} Password: {}", &user, &password,).as_str(),
                    &mut self.tls_stream,
                )?;

                match protocol_result {
                    ProtocolResult::AuthSuccess => Ok(()),
                    ProtocolResult::QueryError(auth_err) => {
                        Err(Error::AuthenticationError(format!(
                            "La autenticación con usuario '{}' y contraseña '{}' ha fallado:\n\n{}",
                            &user, &password, auth_err,
                        )))
                    },
                    _ => {
                        Err(Error::AuthenticationError(format!(
                            "La autenticación con usuario '{}' y contraseña '{}' ha fallado.\nSe reciibó un resultado de tipo {:?}.",
                            &user, &password, protocol_result,
                        )))
                    }
                }
            }
        }
    }
}
