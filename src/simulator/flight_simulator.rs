use {
    crate::{
        client::{
            cli::{get_client_connection, Client},
            protocol_result::ProtocolResult,
        },
        data::{
            airports::airp::{Airport, AirportsMap},
            flights::{states::FlightState, types::FlightType},
            tracking::live_flight_data::LiveFlightData,
        },
        protocol::{
            aliases::types::{Double, Int, Long},
            errors::error::Error,
        },
        server::pool::threadpool::ThreadPool,
        simulator::utils::FlightCalculations,
    },
    rand::thread_rng,
    std::{
        net::TcpStream,
        sync::{Arc, Mutex},
        thread,
        time::{Duration, Instant, SystemTime, UNIX_EPOCH},
    },
};

/// La duración de una simulación.
const FLIGHT_LIMIT_SECS: u64 = 60;

struct FlightSimulationParams {
    dest_coords: (Double, Double),
    dest_elevation: Double,
    simulation_start: Instant,
    simulation_limit: Duration,
    step_size: Double,
    initial_fuel: Double,
    _fuel_consumption_rate: Double,
}

/// Simulador de vuelos.
pub struct FlightSimulator {
    /// Aeropuertos disponibles.
    pub airports: Arc<AirportsMap>,
    flights: Arc<Mutex<Vec<LiveFlightData>>>,
    thread_pool: ThreadPool,
    client: Client,
}

impl FlightSimulator {
    /// Crea un nuevo simulador de vuelos con un número máximo de hilos y un cliente.
    pub fn new(max_threads: usize, client: Client) -> Result<Self, Error> {
        let airports = Airport::get_all()?;

        Ok(FlightSimulator {
            flights: Arc::new(Mutex::new(Vec::new())),
            thread_pool: ThreadPool::build(max_threads)?,
            client,
            airports: Arc::new(airports),
        })
    }

    /// Obtiene los datos específicos de un vuelo según el id solicitado.
    pub fn get_flight_data(&self, flight_id: Int) -> Option<LiveFlightData> {
        if let Ok(flights) = self.flights.lock() {
            flights.iter().find(|f| f.flight_id == flight_id).cloned()
        } else {
            None
        }
    }

    /// Obtiene datos principales de todos los vuelos cargados al simulador.
    pub fn get_all_flights(&self) -> Vec<LiveFlightData> {
        self.flights
            .lock()
            .map(|flights| flights.clone())
            .unwrap_or_default()
    }

    /// Agrega un vuelo al simulador con un id, aeropuerto de origen y destino, y velocidad promedio.
    pub fn add_flight(
        &self,
        flight_id: Int,
        origin: String,
        destination: String,
        avg_speed: Double,
    ) -> Result<(), Error> {
        let (flight, dest_coords, dest_elevation) =
            self.initialize_flight(flight_id, origin, destination, avg_speed)?;

        if let Ok(mut flight_list) = self.flights.lock() {
            flight_list.push(flight.clone());
        }

        let flights = Arc::clone(&self.flights);
        let client = self.client.clone();

        self.thread_pool.execute(move || {
            Self::simulate_flight(flights, flight, client, dest_coords, dest_elevation);
            Ok(())
        })
    }

    fn simulate_flight(
        flights: Arc<Mutex<Vec<LiveFlightData>>>,
        mut flight: LiveFlightData,
        client: Client,
        dest_coords: (Double, Double),
        dest_elevation: Double,
    ) {
        let mut rng = thread_rng();
        let _ = Self::prepare_flight(&flights, &mut flight, &client);

        let (total_distance, initial_fuel, _fuel_consumption_rate) =
            Self::initialize_flight_parameters(&flight, dest_coords);

        flight.state = FlightState::InCourse;
        Self::update_flight_in_list(&flights, &flight);

        thread::sleep(Duration::from_millis(100));

        let simulation_start = Instant::now();
        let simulation_limit = Duration::from_secs(FLIGHT_LIMIT_SECS);
        let step_size = total_distance / 50.0;

        let params = FlightSimulationParams {
            dest_coords,
            dest_elevation,
            simulation_start,
            simulation_limit,
            step_size,
            initial_fuel,
            _fuel_consumption_rate,
        };

        Self::run_flight_simulation(&flights, &mut flight, &client, &params, &mut rng);

        let _ = Self::finish_flight(
            &flights,
            &mut flight,
            dest_coords,
            dest_elevation,
            &client,
            params.simulation_start.elapsed().as_secs_f64(),
        );
    }

    fn run_flight_simulation(
        flights: &Arc<Mutex<Vec<LiveFlightData>>>,
        flight: &mut LiveFlightData,
        client: &Client,
        params: &FlightSimulationParams,
        rng: &mut rand::rngs::ThreadRng,
    ) {
        let mut _distance_traveled = 0.0;
        let total_distance = FlightCalculations::calculate_distance(
            flight.lat(),
            flight.lon(),
            params.dest_coords.0,
            params.dest_coords.1,
        );

        while params.simulation_start.elapsed() < params.simulation_limit {
            let progress = params.simulation_start.elapsed().as_secs_f64()
                / params.simulation_limit.as_secs_f64();

            Self::update_flight_position(
                flight,
                params.dest_coords,
                params.step_size,
                progress,
                params.dest_elevation,
                rng,
            );

            let step_distance = FlightCalculations::calculate_distance(
                flight.lat(),
                flight.lon(),
                params.dest_coords.0,
                params.dest_coords.1,
            );

            _distance_traveled = total_distance - step_distance;

            let progress_factor = _distance_traveled / total_distance;
            let current_fuel =
                params.initial_fuel - ((params.initial_fuel - 60.0) * progress_factor);

            Self::update_flight_in_list(flights, flight);

            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_else(|_| Duration::from_secs(0))
                .as_secs() as Long;

            let _ = Self::send_flight_update(
                flight,
                timestamp,
                client,
                current_fuel,
                params.simulation_start.elapsed().as_secs_f64(),
            );
            thread::sleep(Duration::from_secs(1));
        }
    }

    fn send_flight_update(
        flight: &LiveFlightData,
        timestamp: Long,
        client: &Client,
        fuel: Double,
        elapsed: Double,
    ) -> Result<(), Error> {
        let incoming_query = format!(
            "INSERT INTO vuelos_entrantes_en_vivo (id, orig, dest, llegada, pos_lat, pos_lon, estado, velocidad, altitud, nivel_combustible, duracion) VALUES ({}, '{}', '{}', {}, {}, {}, '{}', {}, {}, {}, {});",
            flight.flight_id, flight.orig, flight.dest, timestamp, flight.lat(), flight.lon(), flight.state, flight.get_spd(), flight.altitude_ft, fuel, elapsed);

        let departing_query = format!(
            "INSERT INTO vuelos_salientes_en_vivo (id, orig, dest, salida, pos_lat, pos_lon, estado, velocidad, altitud, nivel_combustible, duracion) VALUES ({}, '{}', '{}', {}, {}, {}, '{}', {}, {}, {}, {});",
            flight.flight_id, flight.orig, flight.dest, timestamp, flight.lat(), flight.lon(), flight.state, flight.get_spd(), flight.altitude_ft, fuel, elapsed);

        Self::send_insert_query(&incoming_query, &mut client.clone())?;
        Self::send_insert_query(&departing_query, &mut client.clone())?;

        Ok(())
    }

    fn send_insert_query(query: &str, client: &mut Client) -> Result<(), Error> {
        let mut tcp_stream = client.connect()?;
        let mut client_connection = get_client_connection()?;
        let mut tls_stream: rustls::Stream<'_, rustls::ClientConnection, TcpStream> =
            rustls::Stream::new(&mut client_connection, &mut tcp_stream);
        let protocol_result = client.send_query(query, &mut tls_stream)?;

        if let ProtocolResult::QueryError(err) = protocol_result {
            println!("{}", err);
        }

        Ok(())
    }

    fn initialize_flight(
        &self,
        flight_id: Int,
        origin: String,
        destination: String,
        avg_speed: Double,
    ) -> Result<(LiveFlightData, (Double, Double), Double), Error> {
        let (origin_airport, destination_airport) =
            self.validate_airports(&origin, &destination)?;

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_else(|_| Duration::from_secs(0))
            .as_secs() as Long;

        let origin_coords = (origin_airport.position.0, origin_airport.position.1);
        let dest_coords = (
            destination_airport.position.0,
            destination_airport.position.1,
        );

        let flight = LiveFlightData::new(
            flight_id,
            (origin_airport.ident, destination_airport.ident),
            (timestamp, 0.0),
            (avg_speed, 100.0),
            origin_coords,
            origin_airport.elevation_ft.unwrap_or(0) as Double,
            (FlightType::Departing, FlightState::Preparing),
        );

        Ok((
            flight,
            dest_coords,
            destination_airport.elevation_ft.unwrap_or(0) as Double,
        ))
    }

    fn validate_airports(
        &self,
        origin: &str,
        destination: &str,
    ) -> Result<(Airport, Airport), Error> {
        let origin_airport = self
            .airports
            .get(origin)
            .ok_or_else(|| {
                Error::ServerError(format!("Aeropuerto de origen '{}' no encontrado", origin))
            })?
            .clone();

        let destination_airport = self
            .airports
            .get(destination)
            .ok_or_else(|| {
                Error::ServerError(format!(
                    "Aeropuerto de destino '{}' no encontrado",
                    destination
                ))
            })?
            .clone();

        Ok((origin_airport, destination_airport))
    }

    fn prepare_flight(
        flights: &Arc<Mutex<Vec<LiveFlightData>>>,
        flight: &mut LiveFlightData,
        client: &Client,
    ) -> Result<(), Error> {
        flight.set_spd(0.0);
        flight.state = FlightState::Preparing;

        Self::update_flight_in_list(flights, flight);

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_else(|_| Duration::from_secs(0))
            .as_secs() as Long;

        Self::send_flight_update(flight, timestamp, client, 100.0, 0.0)?;
        thread::sleep(Duration::from_secs(2));

        Ok(())
    }

    fn initialize_flight_parameters(
        flight: &LiveFlightData,
        dest_coords: (Double, Double),
    ) -> (Double, Double, Double) {
        let total_distance = FlightCalculations::calculate_distance(
            flight.lat(),
            flight.lon(),
            dest_coords.0,
            dest_coords.1,
        );

        let initial_fuel = 100.0;
        let final_fuel = 60.0;
        let _fuel_consumption_rate = (initial_fuel - final_fuel) / total_distance;

        (total_distance, initial_fuel, _fuel_consumption_rate)
    }

    fn update_flight_position(
        flight: &mut LiveFlightData,
        dest_coords: (Double, Double),
        step_size: Double,
        progress: Double,
        dest_elevation: Double,
        rng: &mut rand::rngs::ThreadRng,
    ) {
        let (new_lat, new_lon) = FlightCalculations::calculate_next_position(
            flight.lat(),
            flight.lon(),
            dest_coords.0,
            dest_coords.1,
            step_size,
        );

        flight.pos = (new_lat, new_lon);
        flight.set_spd(FlightCalculations::calculate_current_speed(
            flight.avg_spd(),
            progress,
            rng,
        ));

        let base_altitude = FlightCalculations::calculate_cruise_altitude(
            flight.altitude_ft,
            dest_elevation,
            progress,
        );
        flight.altitude_ft = FlightCalculations::calculate_current_altitude(base_altitude, rng);
    }

    fn update_flight_in_list(flights: &Arc<Mutex<Vec<LiveFlightData>>>, flight: &LiveFlightData) {
        if let Ok(mut flight_list) = flights.lock() {
            if let Some(existing_flight) = flight_list
                .iter_mut()
                .find(|f| f.flight_id == flight.flight_id)
            {
                *existing_flight = flight.clone();
            }
        }
    }

    fn finish_flight(
        flights: &Arc<Mutex<Vec<LiveFlightData>>>,
        flight: &mut LiveFlightData,
        dest_coords: (Double, Double),
        dest_elevation: Double,
        client: &Client,
        elapsed: Double,
    ) -> Result<(), Error> {
        flight.state = FlightState::Finished;
        flight.pos = dest_coords;
        flight.set_spd(0.0);
        flight.altitude_ft = dest_elevation;

        Self::update_flight_in_list(flights, flight);

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_else(|_| Duration::from_secs(0))
            .as_secs() as Long;

        Self::send_flight_update(flight, timestamp, client, 60.0, elapsed)
    }
}

impl Default for FlightSimulator {
    fn default() -> Self {
        Self::new(4, Client::default()).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flight_simulator() -> Result<(), Error> {
        let simulator = FlightSimulator::default();

        simulator.add_flight(123456, "SAEZ".to_string(), "LEMD".to_string(), 900.0)?;

        thread::sleep(Duration::from_secs(3));

        let flight_data = simulator.get_flight_data(123456);
        assert!(flight_data.is_some());

        if let Some(data) = flight_data {
            assert_eq!(data.state, FlightState::InCourse);
        }

        thread::sleep(Duration::from_secs(FLIGHT_LIMIT_SECS));

        if let Some(data) = simulator.get_flight_data(123456) {
            assert_eq!(
                data.state,
                FlightState::Finished,
                "El estado del vuelo es {:?} cuando debería ser Finished",
                data.state
            );
        }

        Ok(())
    }
}
