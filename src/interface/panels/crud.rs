//! Módulo para operaciones CRUD en los paneles.

use std::sync::{Arc, Mutex};

use crate::{
    client::{cli::Client, protocol_result::ProtocolResult},
    data::{airports::airp::Airport, flights::states::FlightState, utils::distances::distance_eta},
    protocol::{
        aliases::{
            results::Result,
            types::{Int, Long},
        },
        errors::error::Error,
    },
};

/// Inserta un nuevo vuelo.
pub fn insert_flight(
    client: Arc<Mutex<Client>>,
    timestamp: Long,
    cur_airport: &Airport,
    ex_airport: &Airport,
) -> Result<()> {
    let flight_id = cur_airport.id + ex_airport.id + timestamp as usize;

    let incoming_client = Arc::clone(&client);
    let flight_duration = distance_eta(&cur_airport.position, &ex_airport.position, None, None);
    let eta = (timestamp as u64 + flight_duration.as_secs()) as i64;
    let incoming_query = format!(
        "INSERT INTO vuelos_entrantes (id, orig, dest, llegada, estado) VALUES ({}, '{}', '{}', {}, '{}');",
        flight_id as Int, cur_airport.ident, ex_airport.ident, eta, FlightState::InCourse
    );

    let departing_client = Arc::clone(&client);
    let departing_query = format!(
        "INSERT INTO vuelos_salientes (id, orig, dest, salida, estado) VALUES ({}, '{}', '{}', {}, '{}');",
        flight_id as Int, cur_airport.ident, ex_airport.ident, timestamp, FlightState::InCourse
    );

    send_client_query(incoming_client, incoming_query.as_str())?;
    send_client_query(departing_client, departing_query.as_str())?;

    Ok(())
}

/// Manda una _query_ para borrar el vuelo por su ID.
pub fn delete_flight_by_id(client_lock: Arc<Mutex<Client>>, flight_id: Int) -> Result<()> {
    let inc_client = Arc::clone(&client_lock);
    let inc_delete = format!("DELETE FROM vuelos_entrantes WHERE id = {};", flight_id);

    let dep_client = Arc::clone(&client_lock);
    let dep_delete = format!("DELETE FROM vuelos_salientes WHERE id = {};", flight_id);

    send_client_query(inc_client, inc_delete.as_str())?;
    send_client_query(dep_client, dep_delete.as_str())?;

    Ok(())
}

/// Manda una _query_ para insertar un tipo de vuelo.
fn send_client_query(client_lock: Arc<Mutex<Client>>, query: &str) -> Result<()> {
    let mut client = match client_lock.lock() {
        Ok(cli) => cli,
        Err(poison_err) => {
            return Err(Error::ServerError(format!(
                "Error de lock envenenado tratando de leer un cliente:\n\n{}",
                poison_err
            )))
        }
    };

    let mut tcp_stream = client.connect()?;
    let protocol_result = client.send_query(query, &mut tcp_stream)?;

    if let ProtocolResult::QueryError(err) = protocol_result {
        println!("{}", err);
    }

    Ok(())
}
