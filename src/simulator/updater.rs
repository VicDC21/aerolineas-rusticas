use {
    crate::{
        client::cli::{Client, TlsStream},
        data::{flights::states::FlightState, tracking::live_flight_data::LiveFlightData},
        protocol::aliases::{
            results::Result,
            types::{Double, Int, Ulong},
        },
        simulator::{
            connection::set_client_and_connection,
            flight_simulator::FLIGHT_LIMIT_SECS,
            initializer::{initialize_flight_parameters, prepare_flight},
            sender::send_flight_update,
            utils::FlightCalculations,
        },
    },
    rand::thread_rng,
    std::{
        collections::HashMap,
        sync::{Arc, RwLock},
        thread,
        time::{Duration, Instant},
    },
};

struct FlightSimulationParams {
    origin_coords: (Double, Double),
    dest_coords: (Double, Double),
    dest_elevation: Double,
    simulation_start: Instant,
    simulation_limit: Duration,
    fuel_consumption_rate: Double,
}

/// Simula un vuelo con los datos de un vuelo en curso.
pub fn simulate_flight(
    flights: &Arc<RwLock<HashMap<Int, LiveFlightData>>>,
    mut flight: LiveFlightData,
    dest_coords: (Double, Double),
    dest_elevation: Double,
    has_to_connect: bool,
) {
    let (mut client, mut tls_stream) = match set_client_and_connection(has_to_connect) {
        Ok((client, tls_stream)) => (client, tls_stream),
        Err(err) => {
            eprintln!("Error en la conexión del cliente: {}", err);
            return;
        }
    };

    let mut rng = thread_rng();
    let _ = prepare_flight(flights, &mut flight, &mut client, &mut tls_stream);

    let (total_distance, fuel_consumption_rate) =
        initialize_flight_parameters(&flight, dest_coords);

    thread::sleep(Duration::from_secs(2));

    flight.state = FlightState::InCourse;
    update_flight_in_list(flights, &mut flight);

    let simulation_start = Instant::now();
    let simulation_limit = if tls_stream.is_some() {
        Duration::from_secs(
            ((total_distance * (FLIGHT_LIMIT_SECS as Double)) / flight.get_spd()) as Ulong,
        )
    } else {
        Duration::from_secs(FLIGHT_LIMIT_SECS)
    };

    let params = FlightSimulationParams {
        origin_coords: flight.pos,
        dest_coords,
        dest_elevation,
        simulation_start,
        simulation_limit,
        fuel_consumption_rate,
    };

    let _ = run_flight_simulation(
        flights,
        &mut flight,
        &mut client,
        &params,
        &mut rng,
        &mut tls_stream,
    );
    let _ = finish_flight(
        flights,
        &mut flight,
        &params,
        &mut client,
        params.simulation_start.elapsed().as_secs_f64(),
        &mut tls_stream,
    );
}

/// Actualiza los datos de un vuelo en la lista de vuelos.
pub fn update_flight_in_list(
    flights: &Arc<RwLock<HashMap<Int, LiveFlightData>>>,
    flight: &mut LiveFlightData,
) {
    if let Ok(mut flight_map) = flights.write() {
        if let Some(existing_flight) = flight_map.get_mut(&flight.flight_id) {
            existing_flight.set_spd(*flight.get_spd());
            existing_flight.fuel = flight.fuel;
            existing_flight.pos = flight.pos;
            existing_flight.altitude_ft = flight.altitude_ft;
            existing_flight.state = match flight.state {
                FlightState::Finished => FlightState::Finished,
                _ => FlightState::InCourse,
            };
            existing_flight.elapsed = flight.elapsed;
        }
    }
}

fn run_flight_simulation(
    flights: &Arc<RwLock<HashMap<Int, LiveFlightData>>>,
    flight: &mut LiveFlightData,
    client: &mut Client,
    params: &FlightSimulationParams,
    rng: &mut rand::rngs::ThreadRng,
    tls_stream: &mut Option<TlsStream>,
) -> Result<()> {
    while params.simulation_start.elapsed().as_secs_f64() < params.simulation_limit.as_secs_f64() {
        let progress =
            params.simulation_start.elapsed().as_secs_f64() / params.simulation_limit.as_secs_f64();
        update_flight_position(flight, params, progress, rng);

        flight.fuel = (flight.fuel - params.fuel_consumption_rate).max(0.0);
        update_flight_in_list(flights, flight);
        if tls_stream.is_some() {
            let _ = send_flight_update(
                flight,
                client,
                flight.fuel,
                params.simulation_start.elapsed().as_secs_f64(),
                tls_stream,
            );
        }

        thread::sleep(Duration::from_secs(1));
    }
    Ok(())
}

fn update_flight_position(
    flight: &mut LiveFlightData,
    params: &FlightSimulationParams,
    progress: Double,
    rng: &mut rand::rngs::ThreadRng,
) {
    let (new_lat, new_lon) = FlightCalculations::calculate_next_position(
        params.origin_coords.0,
        params.origin_coords.1,
        params.dest_coords.0,
        params.dest_coords.1,
        progress,
    );

    flight.pos = (new_lat, new_lon);
    flight.set_spd(FlightCalculations::calculate_current_speed(
        flight.avg_spd(),
        rng,
    ));
    flight.altitude_ft = FlightCalculations::calculate_current_altitude(
        flight.altitude_ft,
        params.dest_elevation,
        params.simulation_limit.as_secs_f64(),
        params.simulation_start.elapsed().as_secs_f64(),
        rng,
    );
}

fn finish_flight(
    flights: &Arc<RwLock<HashMap<Int, LiveFlightData>>>,
    flight: &mut LiveFlightData,
    params: &FlightSimulationParams,
    client: &mut Client,
    elapsed: Double,
    tls_stream: &mut Option<TlsStream>,
) -> Result<()> {
    flight.state = FlightState::Finished;
    flight.pos = params.dest_coords;
    flight.set_spd(0.0);
    flight.altitude_ft = params.dest_elevation;

    update_flight_in_list(flights, flight);

    if tls_stream.is_some() {
        let _ = send_flight_update(flight, client, flight.fuel, elapsed, tls_stream);
    }
    Ok(())
}
