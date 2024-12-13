use {
    crate::{
        data::{
            airports::airp::{Airport, AirportsMap},
            flights::states::FlightState,
            tracking::live_flight_data::LiveFlightData,
        },
        protocol::{
            aliases::{
                results::Result,
                types::{Double, Int, Ulong},
            },
            errors::error::Error,
        },
        server::pool::threadpool::ThreadPool,
        simulator::{initializer::initialize_flight, updater::simulate_flight},
    },
    std::{
        collections::HashMap,
        process::exit,
        sync::{Arc, RwLock},
    },
};

/// La duración de una simulación.
pub const FLIGHT_LIMIT_SECS: Ulong = 10;

/// Simulador de vuelos.
pub struct FlightSimulator {
    /// Aeropuertos disponibles en el simulador.
    pub airports: Arc<AirportsMap>,
    flights: Arc<RwLock<HashMap<Int, LiveFlightData>>>,
    thread_pool: ThreadPool,
    has_to_connect: bool,
}

impl FlightSimulator {
    /// Crea un nuevo simulador de vuelos con un número máximo de hilos y un cliente.
    pub fn new(max_threads: usize, has_to_connect: bool) -> Result<Self> {
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
        println!("Aeropuertos disponibles:");
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
    ) -> Result<()> {
        if self.get_flight_data(flight_id).is_some() {
            return Err(Error::ServerError(format!(
                "El vuelo con id {} ya existe",
                flight_id
            )));
        }

        let (flight, _, _) = initialize_flight(self, flight_id, &origin, &destination, avg_spd)?;

        if let Ok(mut flight_map) = self.flights.write() {
            flight_map.insert(flight_id, flight);
        }

        let (flight, dest_coords, dest_elevation) =
            initialize_flight(self, flight_id, &origin, &destination, avg_spd)?;

        let has_to_connect = self.has_to_connect;
        let flight_map_ref = Arc::downgrade(&self.flights);
        self.thread_pool.execute(move || {
            if let Some(flights) = flight_map_ref.upgrade() {
                simulate_flight(
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

    /// Obtiene la cantidad de vuelos activos en el simulador.
    pub fn count_active_flights(&self) -> usize {
        match self.flights.read() {
            Ok(flights) => flights
                .values()
                .filter(|flight| flight.state != FlightState::Finished)
                .count(),
            Err(_) => 0,
        }
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
    use {
        super::*,
        crate::{data::flights::states::FlightState, protocol::aliases::results::Result},
        std::{thread, time::Duration},
    };

    #[test]
    fn test_flight_simulator() -> Result<()> {
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
    fn test_concurrent_flights_simulation() -> Result<()> {
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
