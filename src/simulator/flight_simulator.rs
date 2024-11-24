use {
    crate::{
        client::{cli::Client, protocol_result::ProtocolResult},
        data::airports::airp::{Airport, AirportsMap},
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

/// Ejecuta el simulador de vuelos.
pub fn run_sim(client: Client) -> Result<(), Error> {
    match FlightSimulator::new(4, client) {
        Ok(simulator) => loop {
            println!("\nSimulador de Vuelos");
            println!("1. Añadir vuelo");
            println!("2. Ver estado de un vuelo");
            println!("3. Ver todos los vuelos");
            println!("4. Ver aeropuertos disponibles");
            println!("5. Salir");

            let mut input = String::new();
            std::io::stdin().read_line(&mut input).unwrap();

            match input.trim() {
                "1" => {
                    println!("Número de vuelo:");
                    let mut flight_num = String::new();
                    std::io::stdin().read_line(&mut flight_num).unwrap();

                    println!("Código del aeropuerto de origen:");
                    let mut origin = String::new();
                    std::io::stdin().read_line(&mut origin).unwrap();

                    println!("Código del aeropuerto de destino:");
                    let mut dest = String::new();
                    std::io::stdin().read_line(&mut dest).unwrap();

                    println!("Velocidad promedio (km/h):");
                    let mut speed = String::new();
                    std::io::stdin().read_line(&mut speed).unwrap();

                    if let Ok(speed) = speed.trim().parse::<f64>() {
                        match simulator.add_flight(
                            flight_num.trim().to_string(),
                            origin.trim().to_string(),
                            dest.trim().to_string(),
                            speed,
                        ) {
                            Ok(_) => println!("Vuelo añadido exitosamente"),
                            Err(e) => println!("Error al añadir vuelo: {}", e),
                        }
                    }
                }
                "4" => {
                    println!("Aeropuertos disponibles:");
                    for (code, airport) in simulator.airports.iter() {
                        println!("{}: {} ({})", code, airport.name, airport.municipality);
                    }
                }
                "5" => break Ok(()),
                _ => println!("Opción no válida"),
            }
        },
        Err(e) => Err(e),
    }
}

#[derive(Debug, Clone)]
struct FlightData {
    flight_number: String,
    origin_airport: Airport,
    destination_airport: Airport,
    avg_speed: f64,
    current_position: (f64, f64),
    altitude: f64,
    fuel_level: f64,
    current_speed: f64,
    status: FlightState,
}

struct FlightSimulator {
    flights: Arc<Mutex<Vec<FlightData>>>,
    thread_pool: ThreadPool,
    client: Client,
    airports: Arc<AirportsMap>,
}

#[derive(Debug, Clone, PartialEq)]
enum FlightState {
    InCourse,
    Finished,
    Preparing,
}

impl FlightSimulator {
    fn new(max_threads: usize, client: Client) -> Result<Self, Error> {
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
        flight_number: String,
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

        let flight = FlightData {
            flight_number,
            origin_airport,
            destination_airport,
            avg_speed,
            current_position: (0.0, 0.0),
            altitude: 0.0,
            fuel_level: 100.0,
            current_speed: 0.0,
            status: FlightState::Preparing,
        };

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
        flights: Arc<Mutex<Vec<FlightData>>>,
        mut flight: FlightData,
        client: Client,
    ) {
        let mut rng = rand::thread_rng();
        let fuel_consumption_rate = 0.1;
        thread::sleep(Duration::from_secs(2));

        flight.current_position = (
            flight.origin_airport.position.lat(),
            flight.origin_airport.position.lon(),
        );

        if let Ok(mut flight_list) = flights.lock() {
            if let Some(existing_flight) = flight_list
                .iter_mut()
                .find(|f| f.flight_number == flight.flight_number)
            {
                existing_flight.status = FlightState::InCourse;
                flight.status = FlightState::InCourse;
            }
        }

        let simulation_start = std::time::Instant::now();
        let simulation_limit = Duration::from_secs(15);

        let lat_diff =
            flight.destination_airport.position.lat() - flight.origin_airport.position.lat();
        let lon_diff =
            flight.destination_airport.position.lon() - flight.origin_airport.position.lon();
        let total_distance = (lat_diff.powi(2) + lon_diff.powi(2)).sqrt();
        let lat_step = lat_diff / total_distance * 0.1;
        let lon_step = lon_diff / total_distance * 0.1;

        while flight.fuel_level > 0.0 && simulation_start.elapsed() < simulation_limit {
            flight.current_position.0 += lat_step + rng.gen_range(-0.01..0.01);
            flight.current_position.1 += lon_step + rng.gen_range(-0.01..0.01);

            flight.current_speed = flight.avg_speed * (1.0 + rng.gen_range(-0.1..0.1));
            flight.altitude = 32500.0 + rng.gen_range(-2500.0..2500.0);
            flight.fuel_level = (flight.fuel_level - fuel_consumption_rate).max(0.0);

            if let Ok(mut flight_list) = flights.lock() {
                if let Some(existing_flight) = flight_list
                    .iter_mut()
                    .find(|f| f.flight_number == flight.flight_number)
                {
                    *existing_flight = flight.clone();
                }
            }

            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_else(|_| Duration::from_secs(0))
                .as_secs() as i64;
            let _ = FlightSimulator::send_flight_update(&flight, timestamp, &client);

            thread::sleep(Duration::from_secs(1));
        }

        flight.status = FlightState::Finished;
        flight.current_position = (
            flight.destination_airport.position.lat(),
            flight.destination_airport.position.lon(),
        );

        if let Ok(mut flight_list) = flights.lock() {
            if let Some(existing_flight) = flight_list
                .iter_mut()
                .find(|f| f.flight_number == flight.flight_number)
            {
                existing_flight.status = FlightState::Finished;
                existing_flight.current_position = flight.current_position;
            }
        }

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_else(|_| Duration::from_secs(0))
            .as_secs() as i64;

        let _ = FlightSimulator::send_flight_update(&flight, timestamp, &client);
    }

    fn send_flight_update(
        flight: &FlightData,
        timestamp: i64,
        client: &Client,
    ) -> Result<(), Error> {
        let incoming_query = format!(
            "INSERT INTO vuelos_entrantes_en_vivo (id, orig, dest, llegada, pos_lat, pos_lon, estado, velocidad, altitud, nivel_combustible) VALUES ({}, '{}', '{}', {}, {}, {}, '{}', {}, {}, {});",
            flight.flight_number.parse::<i64>().unwrap(), flight.origin_airport.ident, flight.destination_airport.ident, timestamp, flight.current_position.0, flight.current_position.1, flight.status.clone() as i32, flight.current_speed, flight.altitude, flight.fuel_level
        );

        let departing_query = format!(
            "INSERT INTO vuelos_salientes_en_vivo (id, orig, dest, salida, pos_lat, pos_lon, estado, velocidad, altitud, nivel_combustible) VALUES ({}, '{}', '{}', {}, {}, {}, '{}', {}, {}, {});",
            flight.flight_number.parse::<i64>().unwrap(), flight.origin_airport.ident, flight.destination_airport.ident, timestamp, flight.current_position.0, flight.current_position.1, flight.status.clone() as i32, flight.current_speed, flight.altitude, flight.fuel_level
        );

        FlightSimulator::send_insert_query(&incoming_query, &mut client.clone())?;
        FlightSimulator::send_insert_query(&departing_query, &mut client.clone())?;

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

    pub fn get_flight_data(&self, flight_number: &str) -> Option<FlightData> {
        if let Ok(flights) = self.flights.lock() {
            flights
                .iter()
                .find(|f| f.flight_number == flight_number)
                .cloned()
        } else {
            None
        }
    }

    pub fn get_all_flights(&self) -> Vec<FlightData> {
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

        simulator.add_flight(
            "123456".to_string(),
            "MAD".to_string(),
            "BCN".to_string(),
            800.0,
        )?;

        thread::sleep(Duration::from_secs(3));

        let flight_data = simulator.get_flight_data("123456");
        assert!(flight_data.is_some());

        if let Some(data) = flight_data {
            assert_eq!(data.status, FlightState::InCourse);
        }

        thread::sleep(Duration::from_secs(15));

        if let Some(data) = simulator.get_flight_data("123456") {
            assert_eq!(
                data.status,
                FlightState::Finished,
                "El estado del vuelo es {:?} cuando debería ser Landed",
                data.status
            );
        }

        Ok(())
    }
}
