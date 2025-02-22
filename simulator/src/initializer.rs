use crate::{
    flight_simulator::{FlightSimulator, FLIGHT_LIMIT_SECS},
    sender::send_flight_update,
    updater::update_flight_in_list,
    utils::{get_current_timestamp, FlightCalculations},
};
use client::cli::{Client, TlsStream};
use data::{
    airports::airp::Airport,
    flights::{states::FlightState, types::FlightType},
    tracking::live_flight_data::LiveFlightData,
};
use protocol::{
    aliases::{
        results::Result,
        types::{Double, Int},
    },
    errors::error::Error,
};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

/// Inicializa un vuelo con los datos necesarios.
pub fn initialize_flight(
    simulator: &FlightSimulator,
    flight_id: Int,
    origin: &str,
    destination: &str,
    avg_spd: Double,
) -> Result<(LiveFlightData, (Double, Double), Double)> {
    let (origin_airport, destination_airport) = validate_airports(simulator, origin, destination)?;

    match (
        origin_airport.elevation_ft,
        destination_airport.elevation_ft,
        origin_airport.iata_code.as_ref(),
        destination_airport.iata_code.as_ref(),
    ) {
        (Some(origin_elevation), Some(dest_elevation), Some(origin_iata), Some(dest_iata)) => {
            let flight = LiveFlightData::new(
                flight_id,
                (origin_iata.to_string(), dest_iata.to_string()),
                (get_current_timestamp()?, 0.0),
                (avg_spd, 100.0),
                origin_airport.position,
                origin_elevation as Double,
                (FlightType::Departing, FlightState::Preparing),
            );

            Ok((
                flight,
                destination_airport.position,
                dest_elevation as Double,
            ))
        }
        (_, _, _, _) => Err(Error::ServerError(
            "No se pudieron inicializar los datos del vuelo".to_string(),
        )),
    }
}

/// Prepara un vuelo para despegar.
pub fn prepare_flight(
    flights: &Arc<RwLock<HashMap<Int, LiveFlightData>>>,
    flight: &mut LiveFlightData,
    client: &mut Client,
    tls_stream: &mut Option<TlsStream>,
) -> Result<()> {
    flight.state = FlightState::Preparing;

    update_flight_in_list(flights, flight);
    if tls_stream.is_some() {
        let _ = send_flight_update(flight, client, flight.fuel, 0.0, tls_stream);
    }
    Ok(())
}

/// Inicializa datos de trayectoria de vuelo.
pub fn initialize_flight_parameters(
    flight: &LiveFlightData,
    dest_coords: (Double, Double),
) -> (Double, Double) {
    let total_distance = FlightCalculations::calculate_distance(
        flight.lat(),
        flight.lon(),
        dest_coords.0,
        dest_coords.1,
    );
    (total_distance, (1.0 / FLIGHT_LIMIT_SECS as Double))
}

fn validate_airports<'a>(
    simulator: &'a FlightSimulator,
    origin: &str,
    destination: &str,
) -> Result<(&'a Airport, &'a Airport)> {
    let origin_airport = simulator.airports.get(origin).ok_or_else(|| {
        Error::ServerError(format!("Aeropuerto de origen '{}' no encontrado", origin))
    })?;

    let destination_airport = simulator.airports.get(destination).ok_or_else(|| {
        Error::ServerError(format!(
            "Aeropuerto de destino '{}' no encontrado",
            destination
        ))
    })?;

    Ok((origin_airport, destination_airport))
}
