//! Módulo para funciones auxiliares de la interfaz.

use crate::{
    client::{conn_holder::ConnectionHolder, protocol_result::ProtocolResult},
    protocol::{aliases::results::Result, errors::error::Error},
};

/// Manda una _query_ para insertar un tipo de vuelo.
///
/// Se asume que en la conexión, uno ya se encuentra logueado.
pub fn send_client_query(con_info: &mut ConnectionHolder, query: &str) -> Result<()> {
    let client_lock = con_info.get_cli();

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

    let protocol_result = client.send_query(query, &mut con_info.tls_stream)?;

    if let ProtocolResult::QueryError(err) = protocol_result {
        println!("{}", err);
    }

    Ok(())
}
