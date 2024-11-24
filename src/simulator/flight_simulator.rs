use {
    crate::{
        client::{cli::Client, protocol_result::ProtocolResult},
        data::{
            airports::airp::{Airport, AirportsMap},
            flights::{states::FlightState, types::FlightType},
            tracking::live_flight_data::LiveFlightData,
        },
        protocol::errors::error::Error,
        server::pool::threadpool::ThreadPool,
        simulator::utils::FlightCalculations,
    },
    rand::thread_rng,
    std::{
        sync::{Arc, Mutex},
        thread,
        time::{Duration, Instant, SystemTime, UNIX_EPOCH},
    },
};

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
    pub fn get_flight_data(&self, flight_id: i32) -> Option<LiveFlightData> {
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
        flight_id: i32,
        origin: String,
        destination: String,
        avg_speed: f64,
    ) -> Result<(), Error> {
        let origin_airport = self
            .airports
            .get(&origin)
            .ok_or_else(|| {
                Error::ServerError(format!("Aeropuerto de origen '{}' no encontrado", origin))
            })?
            .clone();

        let destination_airport = self
            .airports
            .get(&destination)
            .ok_or_else(|| {
                Error::ServerError(format!(
                    "Aeropuerto de destino '{}' no encontrado",
                    destination
                ))
            })?
            .clone();

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_else(|_| Duration::from_secs(0))
            .as_secs() as i64;

        let origin_coords = (origin_airport.position.lat(), origin_airport.position.lon());
        let dest_coords = (
            destination_airport.position.lat(),
            destination_airport.position.lon(),
        );

        let flight = LiveFlightData::new(
            flight_id,
            (origin_airport.name, destination_airport.name),
            timestamp,
            avg_speed,
            origin_coords,
            origin_airport.elevation_ft.unwrap_or(0) as f64,
            (FlightType::Departing, FlightState::Preparing),
        );

        let flights = Arc::clone(&self.flights);
        let client = self.client.clone();
        let dest_elevation = destination_airport.elevation_ft.unwrap_or(0) as f64;

        if let Ok(mut flight_list) = flights.lock() {
            flight_list.push(flight.clone());
        }

        self.thread_pool.execute(move || {
            Self::simulate_flight(flights, flight, client, dest_coords, dest_elevation);
            Ok(())
        })
    }

    fn simulate_flight(
        flights: Arc<Mutex<Vec<LiveFlightData>>>,
        mut flight: LiveFlightData,
        client: Client,
        dest_coords: (f64, f64),
        dest_elevation: f64,
    ) {
        let mut rng = thread_rng();
        flight.spd = 0.0;
        flight.state = FlightState::Preparing;

        if let Ok(mut flight_list) = flights.lock() {
            if let Some(existing_flight) = flight_list
                .iter_mut()
                .find(|f| f.flight_id == flight.flight_id)
            {
                *existing_flight = flight.clone();
            }
        }

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_else(|_| Duration::from_secs(0))
            .as_secs() as i64;
        let _ = Self::send_flight_update(&flight, timestamp, &client, 100.0);

        thread::sleep(Duration::from_secs(2));

        let total_distance = FlightCalculations::calculate_distance(
            flight.lat(),
            flight.lon(),
            dest_coords.0,
            dest_coords.1,
        );

        let initial_fuel = 100.0;
        let final_fuel = 60.0;
        let fuel_consumption_rate = (initial_fuel - final_fuel) / total_distance;
        let mut _current_fuel = initial_fuel;
        let mut distance_traveled = 0.0;

        flight.state = FlightState::InCourse;
        if let Ok(mut flight_list) = flights.lock() {
            if let Some(existing_flight) = flight_list
                .iter_mut()
                .find(|f| f.flight_id == flight.flight_id)
            {
                existing_flight.state = FlightState::InCourse;
            }
        }

        let simulation_start = Instant::now();
        let simulation_limit = Duration::from_secs(15);
        let step_size = total_distance / 15.0;

        while simulation_start.elapsed() < simulation_limit {
            let (new_lat, new_lon) = FlightCalculations::calculate_next_position(
                flight.lat(),
                flight.lon(),
                dest_coords.0,
                dest_coords.1,
                step_size,
            );

            let step_distance = FlightCalculations::calculate_distance(
                flight.lat(),
                flight.lon(),
                new_lat,
                new_lon,
            );

            distance_traveled += step_distance;
            _current_fuel =
                (initial_fuel - (distance_traveled * fuel_consumption_rate)).max(final_fuel);

            flight.pos = (new_lat, new_lon);

            let progress =
                simulation_start.elapsed().as_secs_f64() / simulation_limit.as_secs_f64();

            flight.spd =
                FlightCalculations::calculate_current_speed(flight.avg_spd(), progress, &mut rng);

            let base_altitude = FlightCalculations::calculate_cruise_altitude(
                flight.altitude_ft,
                dest_elevation,
                progress,
            );
            flight.altitude_ft =
                FlightCalculations::calculate_current_altitude(base_altitude, &mut rng);

            if let Ok(mut flight_list) = flights.lock() {
                if let Some(existing_flight) = flight_list
                    .iter_mut()
                    .find(|f| f.flight_id == flight.flight_id)
                {
                    *existing_flight = flight.clone();
                }
            }

            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_else(|_| Duration::from_secs(0))
                .as_secs() as i64;

            let _ = Self::send_flight_update(&flight, timestamp, &client, _current_fuel);

            thread::sleep(Duration::from_secs(1));
        }

        flight.state = FlightState::Finished;
        flight.pos = dest_coords;
        flight.spd = 0.0;
        flight.altitude_ft = dest_elevation;

        if let Ok(mut flight_list) = flights.lock() {
            if let Some(existing_flight) = flight_list
                .iter_mut()
                .find(|f| f.flight_id == flight.flight_id)
            {
                *existing_flight = flight.clone();
            }
        }

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_else(|_| Duration::from_secs(0))
            .as_secs() as i64;

        let _ = Self::send_flight_update(&flight, timestamp, &client, final_fuel);
    }

    fn send_flight_update(
        flight: &LiveFlightData,
        timestamp: i64,
        client: &Client,
        fuel: f64,
    ) -> Result<(), Error> {
        let incoming_query = format!(
            "INSERT INTO vuelos_entrantes_en_vivo (id, orig, dest, llegada, pos_lat, pos_lon, estado, velocidad, altitud, nivel_combustible) VALUES ({}, '{}', '{}', {}, {:.4}, {:.4}, '{}', {}, {}, {});",
            flight.flight_id, flight.orig, flight.dest, timestamp, flight.lat(), flight.lon(), flight.state.clone() as i32, flight.spd, flight.altitude_ft, fuel);

        let departing_query = format!(
            "INSERT INTO vuelos_salientes_en_vivo (id, orig, dest, salida, pos_lat, pos_lon, estado, velocidad, altitud, nivel_combustible) VALUES ({}, '{}', '{}', {}, {:.4}, {:.4}, '{}', {}, {}, {});",
            flight.flight_id, flight.orig, flight.dest, timestamp, flight.lat(), flight.lon(), flight.state.clone() as i32, flight.spd, flight.altitude_ft, fuel);

        Self::send_insert_query(&incoming_query, &mut client.clone())?;
        Self::send_insert_query(&departing_query, &mut client.clone())?;

        Ok(())
    }

    fn send_insert_query(query: &str, client: &mut Client) -> Result<(), Error> {
        let mut tcp_stream = client.connect()?;
        let protocol_result = client.send_query(query, &mut tcp_stream)?;

        if let ProtocolResult::QueryError(err) = protocol_result {
            println!("{}", err);
        }

        Ok(())
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

        thread::sleep(Duration::from_secs(15));

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
