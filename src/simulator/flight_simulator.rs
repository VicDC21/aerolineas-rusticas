use {
    crate::{
        client::{
            cli::{get_client_connection, Client, TlsStream},
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
        process::exit,
        sync::{Arc, Mutex},
        thread,
        time::{Duration, Instant, SystemTime, UNIX_EPOCH},
    },
};

/// La duración de una simulación.
const FLIGHT_LIMIT_SECS: u64 = 10;

struct FlightSimulationParams {
    origin_coords: (Double, Double),
    dest_coords: (Double, Double),
    dest_elevation: Double,
    simulation_start: Instant,
    simulation_limit: Duration,
    fuel_consumption_rate: Double,
}

/// Simulador de vuelos.
pub struct FlightSimulator {
    /// Aeropuertos disponibles.
    pub airports: Arc<AirportsMap>,
    flights: Arc<Mutex<Vec<LiveFlightData>>>,
    thread_pool: ThreadPool,
    client: Client,
    has_to_connect: bool,
}

impl FlightSimulator {
    /// Crea un nuevo simulador de vuelos con un número máximo de hilos y un cliente.
    pub fn new(max_threads: usize, client: Client, has_to_connect: bool) -> Result<Self, Error> {
        let airports = Airport::get_all()?;

        Ok(FlightSimulator {
            flights: Arc::new(Mutex::new(Vec::new())),
            thread_pool: ThreadPool::build(max_threads)?,
            client,
            airports: Arc::new(airports),
            has_to_connect,
        })
    }

    /// Obtiene los datos específicos de un vuelo según el id solicitado.
    pub fn get_flight_data(&self, flight_id: Int) -> Option<LiveFlightData> {
        match self.flights.lock() {
            Ok(flights) => flights.iter().find(|f| f.flight_id == flight_id).cloned(),
            Err(_) => None,
        }
    }

    /// Obtiene datos principales de todos los vuelos cargados al simulador.
    pub fn get_all_flights(&self) -> Vec<LiveFlightData> {
        match self.flights.lock() {
            Ok(flights) => flights.clone(),
            Err(_) => Vec::new(),
        }
    }

    /// Agrega un vuelo al simulador con un id, aeropuerto de origen y destino, y velocidad promedio.
    pub fn add_flight(
        &self,
        flight_id: Int,
        origin: String,
        destination: String,
        avg_spd: Double,
    ) -> Result<(), Error> {
        if self.get_flight_data(flight_id).is_some() {
            return Err(Error::ServerError(format!(
                "El vuelo con id {} ya existe",
                flight_id
            )));
        }

        let (flight, dest_coords, dest_elevation) =
            self.initialize_flight(flight_id, origin, destination, avg_spd)?;

        if let Ok(mut flight_list) = self.flights.lock() {
            flight_list.push(flight.clone());
        }

        let flights = Arc::clone(&self.flights);
        let mut client = self.client.clone();

        if self.has_to_connect {
            let client_connection = get_client_connection()?;
            let tcp_stream = client.connect()?;
            let tls_stream = Arc::new(Mutex::new(
                client.create_tls_connection(client_connection, tcp_stream)?,
            ));
            if let Ok(mut tls_stream) = tls_stream.lock() {
                client.send_query("User: juan Password: 1234", &mut tls_stream)?;
            }
            self.thread_pool.execute(move || {
                thread::spawn(move || {
                    Self::simulate_flight(
                        flights,
                        flight,
                        client,
                        dest_coords,
                        dest_elevation,
                        Some(tls_stream),
                        true,
                    );
                });
                Ok(())
            })
        } else {
            self.thread_pool.execute(move || {
                thread::spawn(move || {
                    Self::simulate_flight(
                        flights,
                        flight,
                        client,
                        dest_coords,
                        dest_elevation,
                        None,
                        false,
                    );
                });
                Ok(())
            })
        }
    }

    fn simulate_flight(
        flights: Arc<Mutex<Vec<LiveFlightData>>>,
        mut flight: LiveFlightData,
        client: Client,
        dest_coords: (Double, Double),
        dest_elevation: Double,
        tls_stream: Option<Arc<Mutex<TlsStream>>>,
        _has_to_connect: bool,
    ) {
        let mut rng = thread_rng();
        if let Some(ref tls_stream) = tls_stream {
            let _ =
                Self::prepare_flight(&flights, &mut flight, &client, Some(Arc::clone(tls_stream)));
        } else {
            let _ = Self::prepare_flight(&flights, &mut flight, &client, None);
        }

        let (total_distance, fuel_consumption_rate) =
            Self::initialize_flight_parameters(&flight, dest_coords);

        thread::sleep(Duration::from_secs(2));

        flight.state = FlightState::InCourse;
        Self::update_flight_in_list(&flights, &flight);

        let simulation_start = Instant::now();
        let simulation_limit = if _has_to_connect {
            Duration::from_secs(
                ((total_distance * (FLIGHT_LIMIT_SECS as Double)) / flight.get_spd()) as u64,
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

        if tls_stream.is_some() {
            Self::run_flight_simulation(
                &flights,
                &mut flight,
                &client,
                &params,
                &mut rng,
                tls_stream.clone(),
            );
            let _ = Self::finish_flight(
                &flights,
                &mut flight,
                dest_coords,
                dest_elevation,
                &client,
                params.simulation_start.elapsed().as_secs_f64(),
                tls_stream,
            );
        } else {
            Self::run_flight_simulation(&flights, &mut flight, &client, &params, &mut rng, None);
            let _ = Self::finish_flight(
                &flights,
                &mut flight,
                dest_coords,
                dest_elevation,
                &client,
                params.simulation_start.elapsed().as_secs_f64(),
                None,
            );
        }
    }

    fn run_flight_simulation(
        flights: &Arc<Mutex<Vec<LiveFlightData>>>,
        flight: &mut LiveFlightData,
        client: &Client,
        params: &FlightSimulationParams,
        rng: &mut rand::rngs::ThreadRng,
        tls_stream: Option<Arc<Mutex<TlsStream>>>,
    ) {
        while params.simulation_start.elapsed().as_secs_f64()
            < params.simulation_limit.as_secs_f64()
        {
            let progress = params.simulation_start.elapsed().as_secs_f64()
                / params.simulation_limit.as_secs_f64();
            Self::update_flight_position(
                flight,
                params.origin_coords,
                params.dest_coords,
                params.dest_elevation,
                progress,
                rng,
            );

            flight.fuel = (flight.fuel - params.fuel_consumption_rate).max(0.0);
            Self::update_flight_in_list(flights, flight);
            let timestamp = match Self::get_current_timestamp() {
                Ok(ts) => ts,
                Err(err) => {
                    eprintln!("Error obteniendo timestamp actual: {}", err);
                    return;
                }
            };

            if let Some(ref tls_stream) = tls_stream {
                let _ = Self::send_flight_update(
                    flight,
                    timestamp,
                    client,
                    flight.fuel,
                    params.simulation_start.elapsed().as_secs_f64(),
                    Some(Arc::clone(tls_stream)),
                );
            }
            thread::sleep(Duration::from_secs(1));
        }
    }

    fn send_flight_update(
        flight: &LiveFlightData,
        timestamp: Long,
        client: &Client,
        fuel: Double,
        elapsed: Double,
        tls_stream: Option<Arc<Mutex<TlsStream>>>,
    ) -> Result<(), Error> {
        let incoming_query = format!(
            "INSERT INTO vuelos_entrantes_en_vivo (id, orig, dest, llegada, pos_lat, pos_lon, estado, velocidad, altitud, nivel_combustible, duracion) VALUES ({}, '{}', '{}', {}, {}, {}, '{}', {}, {}, {}, {});",
            flight.flight_id, flight.orig, flight.dest, timestamp, flight.lat(), flight.lon(), flight.state, flight.get_spd(), flight.altitude_ft, fuel, elapsed);

        let departing_query = format!(
            "INSERT INTO vuelos_salientes_en_vivo (id, orig, dest, salida, pos_lat, pos_lon, estado, velocidad, altitud, nivel_combustible, duracion) VALUES ({}, '{}', '{}', {}, {}, {}, '{}', {}, {}, {}, {});",
            flight.flight_id, flight.orig, flight.dest, timestamp, flight.lat(), flight.lon(), flight.state, flight.get_spd(), flight.altitude_ft, fuel, elapsed);

        Self::send_insert_query(&incoming_query, &mut client.clone(), tls_stream.clone())?;
        Self::send_insert_query(&departing_query, &mut client.clone(), tls_stream)?;

        Ok(())
    }

    fn send_insert_query(
        query: &str,
        client: &mut Client,
        tls_stream: Option<Arc<Mutex<TlsStream>>>,
    ) -> Result<(), Error> {
        let protocol_result = {
            let tls_stream = tls_stream.unwrap();
            let mut tls_stream = tls_stream.lock().unwrap();
            client.send_query(query, &mut tls_stream)?
        };

        if let ProtocolResult::QueryError(err) = protocol_result {
            eprintln!("{}", err);
        }

        Ok(())
    }

    fn get_current_timestamp() -> Result<Long, Error> {
        match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(time) => Ok(time.as_secs() as Long),
            Err(_) => Err(Error::ServerError(
                "No se pudo obtener el timestamp actual".to_string(),
            )),
        }
    }

    fn initialize_flight(
        &self,
        flight_id: Int,
        origin: String,
        destination: String,
        avg_spd: Double,
    ) -> Result<(LiveFlightData, (Double, Double), Double), Error> {
        let (origin_airport, destination_airport) =
            self.validate_airports(&origin, &destination)?;

        let timestamp = Self::get_current_timestamp()?;

        match (
            origin_airport.elevation_ft,
            destination_airport.elevation_ft,
        ) {
            (Some(origin_elevation), Some(dest_elevation)) => {
                let flight = LiveFlightData::new(
                    flight_id,
                    (origin_airport.ident, destination_airport.ident),
                    (timestamp, 0.0),
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
            (_, _) => Err(Error::ServerError(
                "No se pudo obtener la elevación de los aeropuertos".to_string(),
            )),
        }
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
        tls_stream: Option<Arc<Mutex<TlsStream>>>,
    ) -> Result<(), Error> {
        flight.state = FlightState::Preparing;

        Self::update_flight_in_list(flights, flight);

        let timestamp = Self::get_current_timestamp()?;
        if tls_stream.is_some() {
            let _ =
                Self::send_flight_update(flight, timestamp, client, flight.fuel, 0.0, tls_stream);
        }
        Ok(())
    }

    fn initialize_flight_parameters(
        flight: &LiveFlightData,
        dest_coords: (Double, Double),
    ) -> (Double, Double) {
        let total_distance = FlightCalculations::calculate_distance(
            flight.lat(),
            flight.lon(),
            dest_coords.0,
            dest_coords.1,
        );
        (total_distance, flight.fuel * 10.0 / total_distance)
    }

    fn update_flight_position(
        flight: &mut LiveFlightData,
        origin_coords: (Double, Double),
        dest_coords: (Double, Double),
        dest_elevation: Double,
        progress: Double,
        rng: &mut rand::rngs::ThreadRng,
    ) {
        let (new_lat, new_lon) = FlightCalculations::calculate_next_position(
            origin_coords.0,
            origin_coords.1,
            dest_coords.0,
            dest_coords.1,
            progress,
        );

        flight.pos = (new_lat, new_lon);
        flight.set_spd(FlightCalculations::calculate_current_speed(
            flight.avg_spd(),
            rng,
        ));

        flight.altitude_ft =
            FlightCalculations::calculate_current_altitude(flight.altitude_ft, rng, progress)
                .max(dest_elevation);
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
        tls_stream: Option<Arc<Mutex<TlsStream>>>,
    ) -> Result<(), Error> {
        flight.state = FlightState::Finished;
        flight.pos = dest_coords;
        flight.set_spd(0.0);
        flight.altitude_ft = dest_elevation;

        Self::update_flight_in_list(flights, flight);
        let timestamp = Self::get_current_timestamp()?;

        if tls_stream.is_some() {
            let _ = Self::send_flight_update(
                flight,
                timestamp,
                client,
                flight.fuel,
                elapsed,
                tls_stream,
            );
        }
        Ok(())
    }
}

impl Default for FlightSimulator {
    fn default() -> Self {
        match Self::new(8, Client::default(), false) {
            Ok(simulator) => simulator,
            Err(err) => {
                eprintln!("{}", err);
                exit(1);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flight_simulator() -> Result<(), Error> {
        let simulator = FlightSimulator::default();

        simulator.add_flight(123456, "SAEZ".to_string(), "LEMD".to_string(), 900.0)?;
        assert!(simulator.get_flight_data(123456).is_some());

        if let Some(data) = simulator.get_flight_data(123456) {
            assert_eq!(data.state, FlightState::Preparing);
        }

        thread::sleep(Duration::from_secs(3));

        if let Some(data) = simulator.get_flight_data(123456) {
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

    #[test]
    fn test_concurrent_flights_simulation() -> Result<(), Error> {
        let simulator = FlightSimulator::default();

        let flight_configs = vec![
            (234567, "SBGR", "KJFK", 900.0),
            (345678, "KLAX", "RJAA", 950.0),
            (456789, "LFPG", "SVMI", 850.0),
        ];

        for &(flight_id, origin, destination, avg_spd) in &flight_configs {
            simulator.add_flight(
                flight_id,
                origin.to_string(),
                destination.to_string(),
                avg_spd as Double,
            )?;
        }

        let check_intervals = 5;
        let total_wait_time = FLIGHT_LIMIT_SECS + check_intervals;
        let check_interval_duration = total_wait_time / check_intervals;

        for _ in 0..check_intervals {
            thread::sleep(Duration::from_secs(check_interval_duration));

            for &(flight_id, _, _, _) in &flight_configs {
                let flight_data = simulator.get_flight_data(flight_id);
                assert!(flight_data.is_some(), "Vuelo {} no encontrado", flight_id);
            }
        }

        for &(flight_id, _, _, _) in &flight_configs {
            let flight_data = simulator.get_flight_data(flight_id);
            assert!(flight_data.is_some(), "Vuelo {} no encontrado", flight_id);

            if let Some(data) = flight_data {
                assert_eq!(
                    data.state,
                    FlightState::Finished,
                    "El vuelo {} no ha finalizado como se esperaba. Estado actual: {:?}",
                    flight_id,
                    data.state
                );
            }
        }

        let all_flights = simulator.get_all_flights();
        assert_eq!(
            all_flights.len(),
            flight_configs.len(),
            "No se registraron todos los vuelos"
        );

        Ok(())
    }
}
