//! Módulo para info de conexión.

use std::{
    net::TcpStream,
    sync::{Arc, Mutex},
};

use rustls::ClientConnection;

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

    /// La conexión.
    pub cli_con: ClientConnection,

    /// El _TCP Stream_.
    pub tcp_stream: TcpStream,
}

impl ConnectionHolder {
    /// Crea una nueva instancia a partir de un cliente.
    pub fn with_cli(mut client: Client) -> Result<Self> {
        client.set_consistency_level("Quorum")?;
        let cli_con = get_client_connection()?;
        let tcp_stream = client.connect()?;

        Ok(Self {
            client: Arc::new(Mutex::new(client)),
            cli_con,
            tcp_stream,
        })
    }

    /// Consigue una referencia al cliente.
    pub fn get_cli(&self) -> Arc<Mutex<Client>> {
        Arc::clone(&self.client)
    }

    /// Devuelve un _TLS Stream_ con los datos propios.
    pub fn get_tls(&mut self) -> Result<TlsStream> {
        match self.client.lock() {
            Err(poison_err) => {
                self.client.clear_poison();
                Err(Error::ServerError(format!(
                    "Error de lock envenenado:\n\n{}",
                    poison_err
                )))
            }
            Ok(client) => client.create_tls_connection(&mut self.cli_con, &mut self.tcp_stream),
        }
    }

    /// Devuelve un _TLS Stream_ con los datos ya logueados.
    pub fn get_tls_and_login(&mut self, user: &String, password: &String) -> Result<TlsStream> {
        match self.client.lock() {
            Err(poison_err) => {
                self.client.clear_poison();
                Err(Error::ServerError(format!(
                    "Error de lock envenenado:\n\n{}",
                    poison_err
                )))
            }
            Ok(mut client) => {
                let mut tls_stream =
                    client.create_tls_connection(&mut self.cli_con, &mut self.tcp_stream)?;
                let protocol_result = client.send_query(
                    format!("User: {} Password: {}", &user, &password,).as_str(),
                    &mut tls_stream,
                )?;

                match protocol_result {
                    ProtocolResult::AuthSuccess => (),
                    ProtocolResult::QueryError(auth_err) => {
                        return Err(Error::AuthenticationError(format!(
                            "La autenticación con usuario '{}' y contraseña '{}' ha fallado:\n\n{}",
                            &user, &password, auth_err,
                        )));
                    }
                    _ => {
                        return Err(Error::AuthenticationError(format!(
                            "La autenticación con usuario '{}' y contraseña '{}' ha fallado.\nSe reciibó un resultado de tipo {:?}.",
                            &user, &password, protocol_result,
                        )));
                    }
                }

                Ok(tls_stream)
            }
        }
    }
}
