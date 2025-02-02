use crate::{
    client::cli::{get_client_connection, Client, TlsStream},
    data::login_info::LoginInfo,
    protocol::{aliases::results::Result, errors::error::Error},
};

/// Establece la conexión con el servidor y el cliente.
pub fn set_client_and_connection(has_to_connect: bool) -> Result<(Client, Option<TlsStream>)> {
    let mut client = Client::default();
    client.set_consistency_level("One")?;
    let tls_stream = create_connection(&mut client, has_to_connect)?;

    Ok((client, tls_stream))
}

fn create_connection(client: &mut Client, has_to_connect: bool) -> Result<Option<TlsStream>> {
    if has_to_connect {
        let client_connection = get_client_connection()?;
        let tcp_stream = client.connect()?;
        let mut tls_stream =
            match Some(client.create_tls_connection(client_connection, tcp_stream)?) {
                Some(tls_stream) => tls_stream,
                None => {
                    return Err(Error::ServerError(
                        "No se pudo crear el stream TLS".to_string(),
                    ))
                }
            };
        client.login(LoginInfo::new_str("juan", "1234"), &mut tls_stream)?;
        Ok(Some(tls_stream))
    } else {
        Ok(None)
    }
}
