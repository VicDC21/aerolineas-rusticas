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

pub struct FlightSimulator {
    flights: Arc<Mutex<Vec<LiveFlightData>>>,
    thread_pool: ThreadPool,
    client: Client,
    pub airports: Arc<AirportsMap>,
}

impl FlightSimulator {
    pub fn new(max_threads: usize, client: Client) -> Result<Self, Error> {
        let airports = Airport::get_all()?;

        Ok(FlightSimulator {
            flights: Arc::new(Mutex::new(Vec::new())),
            thread_pool: ThreadPool::build(max_threads)?,
            client,
            airports: Arc::new(airports),
        })
    }

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

        let flight = LiveFlightData::new(
            flight_id,
            (origin_airport.name, destination_airport.name),
            timestamp,
            avg_speed,
            (0.0, 0.0),
            0.0,
            (FlightType::Departing, FlightState::Preparing),
        );

        let flights = Arc::clone(&self.flights);
        let client = self.client.clone();

        if let Ok(mut flight_list) = flights.lock() {
            flight_list.push(flight.clone());
        }

        self.thread_pool.execute(move || {
            Self::simulate_flight(flights, flight, client);
            Ok(())
        })
    }

    fn simulate_flight(
        flights: Arc<Mutex<Vec<LiveFlightData>>>,
        mut flight: LiveFlightData,
        client: Client,
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

        while simulation_start.elapsed() < simulation_limit {
            let new_lat = flight.lat() + rng.gen_range(-0.01..0.01);
            let new_lon = flight.lon() + rng.gen_range(-0.01..0.01);
            flight.pos = (new_lat, new_lon);

            flight.spd = flight.avg_spd() * (1.0 + rng.gen_range(-0.1..0.1));
            flight.altitude_ft = 32500.0 + rng.gen_range(-2500.0..2500.0);

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

        if let Ok(mut flight_list) = flights.lock() {
            if let Some(existing_flight) = flight_list
                .iter_mut()
                .find(|f| f.flight_id == flight.flight_id)
            {
                existing_flight.state = FlightState::Finished;
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
            flight.flight_id, flight.orig, flight.dest, timestamp, flight.lat(), flight.lon(), flight.state.clone() as i32, flight.spd, flight.altitude_ft, 100.0);

        let departing_query = format!(
            "INSERT INTO vuelos_salientes_en_vivo (id, orig, dest, salida, pos_lat, pos_lon, estado, velocidad, altitud, nivel_combustible) VALUES ({}, '{}', '{}', {}, {}, {}, '{}', {}, {}, {});",
            flight.flight_id, flight.orig, flight.dest, timestamp, flight.lat(), flight.lon(), flight.state.clone() as i32, flight.spd, flight.altitude_ft, 100.0);

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

    pub fn get_flight_data(&self, flight_id: i32) -> Option<LiveFlightData> {
        if let Ok(flights) = self.flights.lock() {
            flights.iter().find(|f| f.flight_id == flight_id).cloned()
        } else {
            None
        }
    }

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

        simulator.add_flight(123456, "MAD".to_string(), "BCN".to_string(), 800.0)?;

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
