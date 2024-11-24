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
    },
    rand::Rng,
    std::{
        sync::{Arc, Mutex},
        thread,
        time::Duration,
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

    /// Agrega un vuelo al simulador.
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

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
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
            (avg_speed, 1.0),
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

    fn calculate_next_position(
        current_lat: f64,
        current_lon: f64,
        dest_lat: f64,
        dest_lon: f64,
        step_size: f64,
    ) -> (f64, f64) {
        let delta_lat = dest_lat - current_lat;
        let delta_lon = dest_lon - current_lon;
        let distance = (delta_lat.powi(2) + delta_lon.powi(2)).sqrt();

        if distance < step_size {
            return (dest_lat, dest_lon);
        }

        let ratio = step_size / distance;
        let new_lat = current_lat + delta_lat * ratio;
        let new_lon = current_lon + delta_lon * ratio;

        (new_lat, new_lon)
    }

    fn calculate_cruise_altitude(origin_elevation: f64, dest_elevation: f64, progress: f64) -> f64 {
        const CRUISE_ALTITUDE: f64 = 35000.0;

        if progress < 0.1 {
            origin_elevation + (CRUISE_ALTITUDE - origin_elevation) * (progress * 10.0)
        } else if progress > 0.9 {
            CRUISE_ALTITUDE - (CRUISE_ALTITUDE - dest_elevation) * ((progress - 0.9) * 10.0)
        } else {
            CRUISE_ALTITUDE
        }
    }

    fn simulate_flight(
        flights: Arc<Mutex<Vec<LiveFlightData>>>,
        mut flight: LiveFlightData,
        client: Client,
        dest_coords: (f64, f64),
        dest_elevation: f64,
    ) {
        let mut rng = rand::thread_rng();
        thread::sleep(Duration::from_secs(2));

        if let Ok(mut flight_list) = flights.lock() {
            if let Some(existing_flight) = flight_list
                .iter_mut()
                .find(|f| f.flight_id == flight.flight_id)
            {
                existing_flight.state = FlightState::InCourse;
                flight.state = FlightState::InCourse;
            }
        }

        let simulation_start = std::time::Instant::now();
        let simulation_limit = Duration::from_secs(15);
        let step_size = 0.01;

        while simulation_start.elapsed() < simulation_limit {
            let progress =
                simulation_start.elapsed().as_secs_f64() / simulation_limit.as_secs_f64();

            let (new_lat, new_lon) = Self::calculate_next_position(
                flight.lat(),
                flight.lon(),
                dest_coords.0,
                dest_coords.1,
                step_size,
            );

            flight.pos = (
                new_lat + rng.gen_range(-0.001..0.001),
                new_lon + rng.gen_range(-0.001..0.001),
            );

            flight.set_spd(flight.avg_spd() * (1.0 + rng.gen_range(-0.05..0.05)));

            flight.altitude_ft =
                Self::calculate_cruise_altitude(flight.altitude_ft, dest_elevation, progress)
                    + rng.gen_range(-100.0..100.0);

            if let Ok(mut flight_list) = flights.lock() {
                if let Some(existing_flight) = flight_list
                    .iter_mut()
                    .find(|f| f.flight_id == flight.flight_id)
                {
                    *existing_flight = flight.clone();
                }
            }

            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_else(|_| Duration::from_secs(0))
                .as_secs() as i64;

            let _ = Self::send_flight_update(&flight, timestamp, &client);

            thread::sleep(Duration::from_secs(1));
        }

        flight.state = FlightState::Finished;
        flight.pos = dest_coords;

        if let Ok(mut flight_list) = flights.lock() {
            if let Some(existing_flight) = flight_list
                .iter_mut()
                .find(|f| f.flight_id == flight.flight_id)
            {
                existing_flight.state = FlightState::Finished;
                existing_flight.pos = flight.pos;
            }
        }

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_else(|_| Duration::from_secs(0))
            .as_secs() as i64;

        let _ = Self::send_flight_update(&flight, timestamp, &client);
    }

    fn send_flight_update(
        flight: &LiveFlightData,
        timestamp: i64,
        client: &Client,
    ) -> Result<(), Error> {
        let incoming_query = format!(
            "INSERT INTO vuelos_entrantes_en_vivo (id, orig, dest, llegada, pos_lat, pos_lon, estado, velocidad, altitud, nivel_combustible) VALUES ({}, '{}', '{}', {}, {}, {}, '{}', {}, {}, {});",
            flight.flight_id, flight.orig, flight.dest, timestamp, flight.lat(), flight.lon(), flight.state.clone() as i32, flight.get_spd(), flight.altitude_ft, 100.0);

        let departing_query = format!(
            "INSERT INTO vuelos_salientes_en_vivo (id, orig, dest, salida, pos_lat, pos_lon, estado, velocidad, altitud, nivel_combustible) VALUES ({}, '{}', '{}', {}, {}, {}, '{}', {}, {}, {});",
            flight.flight_id, flight.orig, flight.dest, timestamp, flight.lat(), flight.lon(), flight.state.clone() as i32, flight.get_spd(), flight.altitude_ft, 100.0);

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
