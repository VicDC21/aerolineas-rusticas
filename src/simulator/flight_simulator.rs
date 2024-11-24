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
        let mut rng = rand::thread_rng();

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

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_else(|_| Duration::from_secs(0))
            .as_secs() as i64;
        let _ = Self::send_flight_update(&flight, timestamp, &client, 100.0);

        thread::sleep(Duration::from_secs(2));

        let r = 6371.0;
        let d_lat = (dest_coords.0 - flight.lat()).to_radians();
        let d_lon = (dest_coords.1 - flight.lon()).to_radians();
        let lat1 = flight.lat().to_radians();
        let lat2 = dest_coords.0.to_radians();

        let a = (d_lat / 2.0).sin().powi(2) + lat1.cos() * lat2.cos() * (d_lon / 2.0).sin().powi(2);
        let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
        let total_distance = r * c;

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

        let simulation_start = std::time::Instant::now();
        let simulation_limit = Duration::from_secs(15);
        let step_size = total_distance / 15.0;

        while simulation_start.elapsed() < simulation_limit {
            let (new_lat, new_lon) = Self::calculate_next_position(
                flight.lat(),
                flight.lon(),
                dest_coords.0,
                dest_coords.1,
                step_size,
            );

            let d_lat = (new_lat - flight.lat()).to_radians();
            let d_lon = (new_lon - flight.lon()).to_radians();
            let lat1 = flight.lat().to_radians();
            let lat2 = new_lat.to_radians();

            let a =
                (d_lat / 2.0).sin().powi(2) + lat1.cos() * lat2.cos() * (d_lon / 2.0).sin().powi(2);
            let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
            let step_distance = r * c;

            distance_traveled += step_distance;
            _current_fuel =
                (initial_fuel - (distance_traveled * fuel_consumption_rate)).max(final_fuel);

            flight.pos = (new_lat, new_lon);

            let progress =
                simulation_start.elapsed().as_secs_f64() / simulation_limit.as_secs_f64();
            if progress < 0.1 {
                flight.spd = flight.avg_spd() * (progress * 10.0);
            } else if progress > 0.9 {
                flight.spd = flight.avg_spd() * (1.0 - ((progress - 0.9) * 10.0));
            } else {
                flight.spd = flight.avg_spd() * (1.0 + rng.gen_range(-0.05..0.05));
            }

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
                existing_flight.state = FlightState::Finished;
                existing_flight.pos = flight.pos;
                existing_flight.spd = 0.0;
                existing_flight.altitude_ft = dest_elevation;
            }
        }

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
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
            flight.flight_id, flight.dest, flight.orig, timestamp, flight.lat(), flight.lon(), flight.state.clone() as i32, flight.spd, flight.altitude_ft, fuel);

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

    fn calculate_next_position(
        current_lat: f64,
        current_lon: f64,
        dest_lat: f64,
        dest_lon: f64,
        step_size: f64,
    ) -> (f64, f64) {
        let r = 6371.0;

        let lat1 = current_lat.to_radians();
        let lon1 = current_lon.to_radians();
        let lat2 = dest_lat.to_radians();
        let lon2 = dest_lon.to_radians();

        let d_lon = lon2 - lon1;
        let d_lat = lat2 - lat1;
        let a = (d_lat / 2.0).sin().powi(2) + lat1.cos() * lat2.cos() * (d_lon / 2.0).sin().powi(2);
        let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
        let current_distance = r * c;

        if current_distance < step_size {
            return (dest_lat, dest_lon);
        }

        let y = d_lon.sin() * lat2.cos();
        let x = lat1.cos() * lat2.sin() - lat1.sin() * lat2.cos() * d_lon.cos();
        let bearing = y.atan2(x);

        let angular_distance = step_size / r;

        let new_lat = (lat1.sin() * angular_distance.cos()
            + lat1.cos() * angular_distance.sin() * bearing.cos())
        .asin();

        let new_lon = lon1
            + (bearing.sin() * angular_distance.sin() * lat1.cos())
                .atan2(angular_distance.cos() - lat1.sin() * new_lat.sin());

        (
            (new_lat.to_degrees() * 10000.0).round() / 10000.0,
            (new_lon.to_degrees() * 10000.0).round() / 10000.0,
        )
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
