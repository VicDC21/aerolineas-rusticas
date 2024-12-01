//! Módulo para funciones auxiliares de la interfaz.

use std::sync::{Arc, Mutex};

use crate::{
    client::{
        cli::{get_client_connection, Client},
        protocol_result::ProtocolResult,
    },
    protocol::{aliases::results::Result, errors::error::Error},
};

/// Manda una _query_ para insertar un tipo de vuelo.
pub fn send_client_query(client_lock: Arc<Mutex<Client>>, query: &str) -> Result<()> {
    let mut client = match client_lock.lock() {
        Ok(cli) => cli,
        Err(poison_err) => {
            client_lock.clear_poison();
            return Err(Error::ServerError(format!(
                "Error de lock envenenado tratando de leer un cliente:\n\n{}",
                poison_err
            )));
        }
    };

    let mut client_connection = get_client_connection()?;
    let mut tcp_stream = client.connect()?;
    let mut tls_stream = client.create_tls_connection(&mut client_connection, &mut tcp_stream)?;
    let protocol_result = client.send_query(query, &mut tls_stream)?;

    if let ProtocolResult::QueryError(err) = protocol_result {
        println!("{}", err);
    }

    Ok(())
}
