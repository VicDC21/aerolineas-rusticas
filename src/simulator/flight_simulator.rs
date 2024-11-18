use {
    crate::{
        client::{cli::Client, protocol_result::ProtocolResult},
        data::flights::states::FlightState,
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

/// Ejecuta el simulador de vuelos
pub fn run_sim(client: Client) -> Result<(), Error> {
    match FlightSimulator::new(4, client) {
        Ok(simulator) => loop {
            println!("\nSimulador de Vuelos");
            println!("1. Añadir vuelo");
            println!("2. Ver estado de un vuelo");
            println!("3. Ver todos los vuelos");
            println!("4. Salir");

            let mut input = String::new();
            std::io::stdin().read_line(&mut input).unwrap();

            match input.trim() {
                "1" => {
                    println!("Número de vuelo:");
                    let mut flight_num = String::new();
                    std::io::stdin().read_line(&mut flight_num).unwrap();

                    println!("Origen:");
                    let mut origin = String::new();
                    std::io::stdin().read_line(&mut origin).unwrap();

                    println!("Destino:");
                    let mut dest = String::new();
                    std::io::stdin().read_line(&mut dest).unwrap();

                    println!("Velocidad promedio (km/h):");
                    let mut speed = String::new();
                    std::io::stdin().read_line(&mut speed).unwrap();

                    if let Ok(speed) = speed.trim().parse::<f64>() {
                        if let Err(e) = simulator.add_flight(
                            flight_num.trim().to_string(),
                            origin.trim().to_string(),
                            dest.trim().to_string(),
                            speed,
                        ) {
                            println!("Error al añadir vuelo: {}", e);
                        }
                    }
                }
                "2" => {
                    println!("Número de vuelo:");
                    let mut flight_num = String::new();
                    std::io::stdin().read_line(&mut flight_num).unwrap();

                    if let Some(data) = simulator.get_flight_data(flight_num.trim()) {
                        println!("{:#?}", data);
                    } else {
                        println!("Vuelo no encontrado");
                    }
                }
                "3" => {
                    for flight in simulator.get_all_flights() {
                        println!("{:#?}", flight);
                    }
                }
                "4" => break Ok(()),
                _ => println!("Opción no válida"),
            }
        },
        Err(e) => Err(e),
    }
}

#[derive(Debug, Clone)]
struct FlightData {
    flight_number: String,
    origin: String,
    destination: String,
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
}

impl FlightSimulator {
    fn new(max_threads: usize, client: Client) -> Result<Self, Error> {
        Ok(FlightSimulator {
            flights: Arc::new(Mutex::new(Vec::new())),
            thread_pool: ThreadPool::build(max_threads)?,
            client,
        })
    }

    pub fn add_flight(
        &self,
        flight_number: String,
        origin: String,
        destination: String,
        avg_speed: f64,
    ) -> Result<(), Error> {
        let flight = FlightData {
            flight_number,
            origin,
            destination,
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

        while flight.fuel_level > 0.0 && simulation_start.elapsed() < simulation_limit {
            flight.current_position.0 += rng.gen_range(-0.1..0.1);
            flight.current_position.1 += rng.gen_range(-0.1..0.1);
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
        if let Ok(mut flight_list) = flights.lock() {
            if let Some(existing_flight) = flight_list
                .iter_mut()
                .find(|f| f.flight_number == flight.flight_number)
            {
                existing_flight.status = FlightState::Finished;
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
            "INSERT INTO vuelos_entrantes_en_vivo (id, dest, llegada, pos_lat, pos_lon, estado, velocidad, altitud, nivel_combustible) VALUES ({}, '{}', {}, {}, {}, '{}', {}, {}, {});",
            flight.flight_number.parse::<i64>().unwrap(), flight.destination, timestamp, flight.current_position.0, flight.current_position.1, flight.status.clone() as i32, flight.current_speed, flight.altitude, flight.fuel_level
        );

        let departing_query = format!(
            "INSERT INTO vuelos_salientes_en_vivo (id, orig, salida, pos_lat, pos_lon, estado, velocidad, altitud, nivel_combustible) VALUES ({}, '{}', {}, {}, {}, '{}', {}, {}, {});",
            flight.flight_number.parse::<i64>().unwrap(), flight.origin, timestamp, flight.current_position.0, flight.current_position.1, flight.status.clone() as i32, flight.current_speed, flight.altitude, flight.fuel_level
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
