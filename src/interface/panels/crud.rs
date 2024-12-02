//! Módulo para operaciones CRUD en los paneles.

use walkers::Position;

use crate::{
    client::conn_holder::ConnectionHolder,
    data::{airports::airp::Airport, flights::states::FlightState, utils::distances::distance_eta},
    interface::{data::login_info::LoginInfo, utils::send_client_query},
    protocol::aliases::{
        results::Result,
        types::{Int, Long},
    },
};

/// Inserta un nuevo vuelo.
pub fn insert_flight(
    con_info: &mut ConnectionHolder,
    login_info: &LoginInfo,
    timestamp: Long,
    cur_airport: &Airport,
    ex_airport: &Airport,
) -> Result<()> {
    let flight_id = cur_airport.id + ex_airport.id + timestamp as usize;

    let (cur_lat, cur_lon) = cur_airport.position;
    let (ex_lat, ex_lon) = ex_airport.position;
    let flight_duration = distance_eta(
        &Position::from_lat_lon(cur_lat, cur_lon),
        &Position::from_lat_lon(ex_lat, ex_lon),
        None,
        None,
    );
    let eta = (timestamp as u64 + flight_duration.as_secs()) as i64;
    let inc_fl_query = format!(
        "INSERT INTO vuelos_entrantes (id, orig, dest, llegada, estado) VALUES ({}, '{}', '{}', {}, '{}');",
        flight_id as Int, cur_airport.ident, ex_airport.ident, eta, FlightState::Preparing
    );

    let dep_fl_query = format!(
        "INSERT INTO vuelos_salientes (id, orig, dest, salida, estado) VALUES ({}, '{}', '{}', {}, '{}');",
        flight_id as Int, cur_airport.ident, ex_airport.ident, timestamp, FlightState::Preparing
    );

    send_client_query(con_info, login_info, inc_fl_query.as_str())?;
    send_client_query(con_info, login_info, dep_fl_query.as_str())?;

    Ok(())
}

/// Manda una _query_ para borrar el vuelo por su ID.
pub fn delete_flight_by_id(
    con_info: &mut ConnectionHolder,
    login_info: &LoginInfo,
    flight_id: Int,
) -> Result<()> {
    let inc_delete = format!("DELETE FROM vuelos_entrantes WHERE id = {};", flight_id);
    let dep_delete = format!("DELETE FROM vuelos_salientes WHERE id = {};", flight_id);

    send_client_query(con_info, login_info, inc_delete.as_str())?;
    send_client_query(con_info, login_info, dep_delete.as_str())?;

    Ok(())
}
