use {
    crate::{
        client::cli::{get_client_connection, Client, TlsStream},
        data::{
            airports::airp::{Airport, AirportsMap},
            flights::{states::FlightState, types::FlightType},
            login_info::LoginInfo,
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
        collections::HashMap,
        process::exit,
        sync::{Arc, RwLock},
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
    airports: Arc<AirportsMap>,
    flights: Arc<RwLock<HashMap<Int, LiveFlightData>>>,
    thread_pool: ThreadPool,
    has_to_connect: bool,
}

impl FlightSimulator {
    /// Crea un nuevo simulador de vuelos con un número máximo de hilos y un cliente.
    pub fn new(max_threads: usize, has_to_connect: bool) -> Result<Self, Error> {
        let airports = Airport::get_all()?;

        Ok(FlightSimulator {
            flights: Arc::new(RwLock::new(HashMap::new())),
            thread_pool: ThreadPool::build(max_threads)?,
            airports: Arc::new(airports),
            has_to_connect,
        })
    }

    /// Obtiene los datos específicos de un vuelo según el id solicitado.
    pub fn get_flight_data(&self, flight_id: Int) -> Option<LiveFlightData> {
        match self.flights.read() {
            Ok(flights) => flights.get(&flight_id).cloned(),
            Err(_) => None,
        }
    }

    /// Obtiene datos principales de todos los vuelos cargados al simulador.
    pub fn get_all_flights(&self) -> Vec<LiveFlightData> {
        match self.flights.read() {
            Ok(flights) => flights.values().cloned().collect(),
            Err(_) => Vec::new(),
        }
    }

    /// Obtiene los aeropuertos disponibles en el simulador.
    pub fn get_airports(&self) {
        for (code, airport) in self.airports.iter() {
            println!(
                "{}: {} ({}, {})",
                code, airport.name, airport.municipality, airport.country.name
            );
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

        let (flight, _, _) = self.initialize_flight(flight_id, &origin, &destination, avg_spd)?;

        if let Ok(mut flight_map) = self.flights.write() {
            flight_map.insert(flight_id, flight);
        }

        let (flight, dest_coords, dest_elevation) =
            self.initialize_flight(flight_id, &origin, &destination, avg_spd)?;

        let has_to_connect = self.has_to_connect;
        let flight_map_ref = Arc::downgrade(&self.flights);
        self.thread_pool.execute(move || {
            if let Some(flights) = flight_map_ref.upgrade() {
                Self::simulate_flight(
                    &flights,
                    flight,
                    dest_coords,
                    dest_elevation,
                    has_to_connect,
                );
            }
            Ok(())
        })
    }

    fn create_connection(
        client: &mut Client,
        has_to_connect: bool,
    ) -> Result<Option<TlsStream>, Error> {
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

    fn set_client_and_connection(
        has_to_connect: bool,
    ) -> Result<(Client, Option<TlsStream>), Error> {
        let mut client = Client::default();
        client.set_consistency_level("One")?;
        let tls_stream = match Self::create_connection(&mut client, has_to_connect) {
            Ok(tls_stream) => tls_stream,
            Err(err) => return Err(err),
        };
        Ok((client, tls_stream))
    }

    fn simulate_flight(
        flights: &Arc<RwLock<HashMap<Int, LiveFlightData>>>,
        mut flight: LiveFlightData,
        dest_coords: (Double, Double),
        dest_elevation: Double,
        has_to_connect: bool,
    ) {
        let (mut client, mut tls_stream) = match Self::set_client_and_connection(has_to_connect) {
            Ok((client, tls_stream)) => (client, tls_stream),
            Err(err) => {
                eprintln!("Error en la conexión del cliente: {}", err);
                return;
            }
        };

        let mut rng = thread_rng();
        let _ = Self::prepare_flight(flights, &mut flight, &mut client, &mut tls_stream);

        let (total_distance, fuel_consumption_rate) =
            Self::initialize_flight_parameters(&flight, dest_coords);

        thread::sleep(Duration::from_secs(2));

        flight.state = FlightState::InCourse;
        Self::update_flight_in_list(flights, &mut flight);

        let simulation_start = Instant::now();
        let simulation_limit = if tls_stream.is_some() {
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

        let _ = Self::run_flight_simulation(
            flights,
            &mut flight,
            &mut client,
            &params,
            &mut rng,
            &mut tls_stream,
        );
        let _ = Self::finish_flight(
            flights,
            &mut flight,
            &params,
            &mut client,
            params.simulation_start.elapsed().as_secs_f64(),
            &mut tls_stream,
        );
    }

    fn run_flight_simulation(
        flights: &Arc<RwLock<HashMap<Int, LiveFlightData>>>,
        flight: &mut LiveFlightData,
        client: &mut Client,
        params: &FlightSimulationParams,
        rng: &mut rand::rngs::ThreadRng,
        tls_stream: &mut Option<TlsStream>,
    ) -> Result<(), Error> {
        while params.simulation_start.elapsed().as_secs_f64()
            < params.simulation_limit.as_secs_f64()
        {
            let progress = params.simulation_start.elapsed().as_secs_f64()
                / params.simulation_limit.as_secs_f64();
            Self::update_flight_position(flight, params, progress, rng);

            flight.fuel = (flight.fuel - params.fuel_consumption_rate).max(0.0);
            Self::update_flight_in_list(flights, flight);
            if tls_stream.is_some() {
                let _ = Self::send_flight_update(
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

    fn send_flight_update(
        flight: &LiveFlightData,
        client: &mut Client,
        fuel: Double,
        elapsed: Double,
        tls_stream: &mut Option<TlsStream>,
    ) -> Result<(), Error> {
        let timestamp = Self::get_current_timestamp()?;

        let incoming_query = format!(
            "INSERT INTO vuelos_entrantes_en_vivo (id, orig, dest, llegada, pos_lat, pos_lon, estado, velocidad, altitud, nivel_combustible, duracion) VALUES ({}, '{}', '{}', {}, {}, {}, '{}', {}, {}, {:.2}, {:.2});",
            flight.flight_id, flight.orig, flight.dest, timestamp, flight.lat(), flight.lon(), flight.state, flight.get_spd(), flight.altitude_ft, fuel, elapsed);

        let departing_query = format!(
            "INSERT INTO vuelos_salientes_en_vivo (id, orig, dest, salida, pos_lat, pos_lon, estado, velocidad, altitud, nivel_combustible, duracion) VALUES ({}, '{}', '{}', {}, {}, {}, '{}', {}, {}, {:.2}, {:.2});",
            flight.flight_id, flight.orig, flight.dest, timestamp, flight.lat(), flight.lon(), flight.state, flight.get_spd(), flight.altitude_ft, fuel, elapsed);

        Self::send_insert_query(&incoming_query, client, tls_stream)?;
        Self::send_insert_query(&departing_query, client, tls_stream)?;

        Ok(())
    }

    fn send_insert_query(
        query: &str,
        client: &mut Client,
        tls_stream: &mut Option<TlsStream>,
    ) -> Result<(), Error> {
        if let Some(tls_stream) = tls_stream {
            match client.send_query(query, tls_stream) {
                Ok(_) => (),
                Err(_) => {
                    let (new_client, new_tls_stream) = match Self::set_client_and_connection(true) {
                        Ok((new_client, new_tls_stream)) => (new_client, new_tls_stream),
                        Err(reconnect_err) => {
                            eprintln!("Error en la reconexión del cliente: {}", reconnect_err);
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
        origin: &str,
        destination: &str,
        avg_spd: Double,
    ) -> Result<(LiveFlightData, (Double, Double), Double), Error> {
        let (origin_airport, destination_airport) = self.validate_airports(origin, destination)?;

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
                    (Self::get_current_timestamp()?, 0.0),
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

    fn validate_airports(
        &self,
        origin: &str,
        destination: &str,
    ) -> Result<(&Airport, &Airport), Error> {
        let origin_airport = self.airports.get(origin).ok_or_else(|| {
            Error::ServerError(format!("Aeropuerto de origen '{}' no encontrado", origin))
        })?;

        let destination_airport = self.airports.get(destination).ok_or_else(|| {
            Error::ServerError(format!(
                "Aeropuerto de destino '{}' no encontrado",
                destination
            ))
        })?;

        Ok((origin_airport, destination_airport))
    }

    fn prepare_flight(
        flights: &Arc<RwLock<HashMap<Int, LiveFlightData>>>,
        flight: &mut LiveFlightData,
        client: &mut Client,
        tls_stream: &mut Option<TlsStream>,
    ) -> Result<(), Error> {
        flight.state = FlightState::Preparing;

        Self::update_flight_in_list(flights, flight);
        if tls_stream.is_some() {
            let _ = Self::send_flight_update(flight, client, flight.fuel, 0.0, tls_stream);
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
        (total_distance, (1.0 / FLIGHT_LIMIT_SECS as Double))
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

    fn update_flight_in_list(
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

    fn finish_flight(
        flights: &Arc<RwLock<HashMap<Int, LiveFlightData>>>,
        flight: &mut LiveFlightData,
        params: &FlightSimulationParams,
        client: &mut Client,
        elapsed: Double,
        tls_stream: &mut Option<TlsStream>,
    ) -> Result<(), Error> {
        flight.state = FlightState::Finished;
        flight.pos = params.dest_coords;
        flight.set_spd(0.0);
        flight.altitude_ft = params.dest_elevation;

        Self::update_flight_in_list(flights, flight);

        if tls_stream.is_some() {
            let _ = Self::send_flight_update(flight, client, flight.fuel, elapsed, tls_stream);
        }
        Ok(())
    }
}

impl Default for FlightSimulator {
    fn default() -> Self {
        match Self::new(8, false) {
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

        simulator.add_flight(123456, "EZE".to_string(), "MAD".to_string(), 900.0)?;
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
            (234567, "EZE", "MAD", 900.0),
            (345678, "MAD", "EZE", 800.0),
            (456789, "EZE", "CDG", 1000.0),
            (567890, "CDG", "EZE", 950.0),
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
