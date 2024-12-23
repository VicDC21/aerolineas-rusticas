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
    ///
    /// # Ejemplo
    ///
    /// ```
    /// # use aerolineas_rusticas::simulator::flight_simulator::FlightSimulator;
    /// let simulator = FlightSimulator::new(4, false).expect("Error al crear el simulador");
    /// assert!(simulator.get_all_flights().is_empty());
    /// ```    
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
    ///
    /// # Ejemplo
    ///
    /// ```
    /// # use aerolineas_rusticas::simulator::flight_simulator::FlightSimulator;
    /// let simulator = FlightSimulator::new(4, false).expect("Error al crear el simulador");
    /// assert!(simulator.get_flight_data(123).is_none());
    /// ```    
    pub fn get_flight_data(&self, flight_id: Int) -> Option<LiveFlightData> {
        match self.flights.read() {
            Ok(flights) => flights.get(&flight_id).cloned(),
            Err(_) => None,
        }
    }

    /// Obtiene datos principales de todos los vuelos cargados al simulador.
    ///
    /// # Ejemplo
    ///
    /// ```
    /// # use aerolineas_rusticas::simulator::flight_simulator::FlightSimulator;
    /// let simulator = FlightSimulator::new(4, false).expect("Error al crear el simulador");
    /// assert!(simulator.get_all_flights().is_empty());
    /// ```    
    pub fn get_all_flights(&self) -> Vec<LiveFlightData> {
        match self.flights.read() {
            Ok(flights) => flights.values().cloned().collect(),
            Err(_) => Vec::new(),
        }
    }

    /// Obtiene los aeropuertos disponibles en el simulador.
    ///
    /// # Ejemplo
    ///
    /// ```
    /// # use aerolineas_rusticas::simulator::flight_simulator::FlightSimulator;
    /// let simulator = FlightSimulator::new(4, false).expect("Error al crear el simulador");
    /// simulator.get_airports();
    /// ```
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
    ///
    /// # Ejemplo
    ///
    /// ```
    /// # use aerolineas_rusticas::simulator::flight_simulator::FlightSimulator;
    /// # use aerolineas_rusticas::protocol::{aliases::results::Result,errors::error::Error};
    /// # fn main() -> Result<()> {
    /// let simulator = FlightSimulator::new(4, false)?;
    ///
    ///
    /// simulator.add_flight(
    ///    1,
    ///    "MAD".to_string(),  // Aeropuerto de Madrid
    ///    "BCN".to_string(),  // Aeropuerto de Barcelona
    ///    500.0               // Velocidad promedio en km/h
    /// )?;
    /// assert_eq!(simulator.count_active_flights(), 1);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errores
    ///
    /// Devuelve un error si:
    /// - El vuelo ya existe
    /// - Los aeropuertos de origen o destino no son válidos
    /// - Hay problemas al inicializar el vuelo
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
    /// Después de añadir un vuelo, cambiará el número de vuelos activos.
    ///
    /// # Ejemplo
    ///
    /// ```
    /// # use aerolineas_rusticas::simulator::flight_simulator::FlightSimulator;
    /// # use aerolineas_rusticas::protocol::{aliases::results::Result,errors::error::Error};
    /// # fn main() -> Result<()> {
    /// let simulator = FlightSimulator::new(4, false)?;
    /// assert_eq!(simulator.count_active_flights(), 0);
    ///
    /// # Ok(())
    /// # }
    /// ```
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
    /// Crea un simulador con configuración predeterminada.
    ///
    /// # Ejemplo
    ///
    /// ```
    /// # use aerolineas_rusticas::simulator::flight_simulator::FlightSimulator;
    /// let simulator = FlightSimulator::default();
    /// assert_eq!(simulator.count_active_flights(), 0);
    /// ```
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
