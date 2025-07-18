use crate::{connection::set_client_and_connection, utils::get_current_timestamp};
use client::cli::{Client, TlsStream};
use data::tracking::live_flight_data::LiveFlightData;
use protocol::{
    aliases::{results::Result, types::Double},
    errors::error::Error,
};

/// Arma la query y envía la actualización de vuelo a la base de datos.
pub fn send_flight_update(
    flight: &LiveFlightData,
    client: &mut Client,
    fuel: Double,
    elapsed: Double,
    tls_stream: &mut Option<TlsStream>,
) -> Result<()> {
    let timestamp = get_current_timestamp()?;

    let incoming_query = format!(
        "INSERT INTO vuelos_entrantes_en_vivo (id, orig, dest, llegada, pos_lat, pos_lon, estado, velocidad, altitud, nivel_combustible, duracion) VALUES ({}, '{}', '{}', {}, {}, {}, '{}', {}, {}, {:.2}, {:.2});",
        flight.flight_id, flight.orig, flight.dest, timestamp, flight.lat(), flight.lon(), flight.state, flight.get_spd(), flight.altitude_ft, fuel, elapsed);

    let departing_query = format!(
        "INSERT INTO vuelos_salientes_en_vivo (id, orig, dest, salida, pos_lat, pos_lon, estado, velocidad, altitud, nivel_combustible, duracion) VALUES ({}, '{}', '{}', {}, {}, {}, '{}', {}, {}, {:.2}, {:.2});",
        flight.flight_id, flight.orig, flight.dest, timestamp, flight.lat(), flight.lon(), flight.state, flight.get_spd(), flight.altitude_ft, fuel, elapsed);

    send_insert_query(&incoming_query, client, tls_stream)?;
    send_insert_query(&departing_query, client, tls_stream)?;

    Ok(())
}

fn send_insert_query(
    query: &str,
    client: &mut Client,
    tls_stream: &mut Option<TlsStream>,
) -> Result<()> {
    if let Some(tls_stream) = tls_stream {
        match client.send_query(query, tls_stream) {
            Ok(_) => (),
            Err(_) => {
                let (new_client, new_tls_stream) = match set_client_and_connection(true) {
                    Ok((new_client, new_tls_stream)) => (new_client, new_tls_stream),
                    Err(reconnect_err) => {
                        eprintln!("Error en la reconexión del cliente: {reconnect_err}");
                        return Err(reconnect_err);
                    }
                };
                *client = new_client;
                *tls_stream = match new_tls_stream {
                    Some(tls_stream) => tls_stream,
                    None => {
                        return Err(Error::ServerError(
                            "No se pudo crear el stream TLS".to_string(),
                        ))
                    }
                };
                client.send_query(query, tls_stream)?;
            }
        }
    }
    Ok(())
}
