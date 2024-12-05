use {
    crate::{
        client::cli::Client,
        data::tracking::live_flight_data::LiveFlightData,
        protocol::{
            aliases::{
                results::Result,
                types::{Double, Int},
            },
            errors::error::Error,
        },
        simulator::flight_simulator::FlightSimulator,
    },
    std::{thread, time::Duration},
};

/// Configuración de un vuelo.
pub struct FlightConfig {
    /// ID del vuelo.
    pub flight_id: Int,
    /// Código del aeropuerto de origen.
    pub origin: &'static str,
    /// Código del aeropuerto de destino.
    pub destination: &'static str,
    /// Velocidad inicial del vuelo.
    pub spd: Double,
}

const MAX_THREADS: usize = 16;

/// Ejecuta el simulador de vuelos.
pub fn run_sim(mut client: Client, flights: &[FlightConfig]) -> Result<()> {
    client.set_consistency_level("One")?;
    match FlightSimulator::new(MAX_THREADS, client, true) {
        Ok(simulator) => {
            script_loop(&simulator, flights);
            app_loop(&simulator)?;
            Ok(())
        }
        Err(e) => Err(e),
    }
}

fn read_line() -> Result<String> {
    let mut input = String::new();
    if let Err(err) = std::io::stdin().read_line(&mut input) {
        return Err(Error::ServerError(format!(
            "Error al leer la entrada: {}",
            err
        )));
    }
    Ok(input)
}

fn script_loop(simulator: &FlightSimulator, flights: &[FlightConfig]) {
    if !flights.is_empty() {
        for flight in flights {
            println!(
                "Añadiendo vuelo {} con origen {} y {}",
                flight.flight_id, flight.origin, flight.destination
            );

            match simulator.add_flight(
                flight.flight_id,
                flight.origin.to_string(),
                flight.destination.to_string(),
                flight.spd,
            ) {
                Ok(_) => println!("Vuelo añadido exitosamente!"),
                Err(e) => println!("Error al añadir vuelo: {}", e),
            }
            thread::sleep(Duration::from_millis(500));
        }
    }
}

fn app_loop(simulator: &FlightSimulator) -> Result<()> {
    loop {
        println!("\nSimulador de Vuelos");
        println!("1. Añadir vuelo");
        println!("2. Ver estado de un vuelo");
        println!("3. Ver todos los vuelos");
        println!("4. Ver aeropuertos disponibles");
        println!("5. Salir");

        let input = read_line()?;

        match input.trim() {
            "1" => handle_add_flight(simulator)?,
            "2" => handle_view_flight(simulator)?,
            "3" => handle_view_all_flights(simulator),
            "4" => handle_view_airports(simulator),
            "5" => break Ok(()),
            _ => println!("Opción no válida"),
        }
    }
}

fn get_initial_data() -> Result<(String, String, String, String)> {
    println!("ID de vuelo:");
    let flight_id = read_line()?;

    println!("Código del aeropuerto de origen:");
    let origin = read_line()?;

    println!("Código del aeropuerto de destino:");
    let dest = read_line()?;

    if origin.trim() == dest.trim() {
        println!("Error: El origen y destino no pueden ser iguales");
        return get_initial_data();
    }

    println!("Velocidad inicial:");
    let spd = read_line()?;

    Ok((flight_id, origin, dest, spd))
}

fn handle_add_flight(simulator: &FlightSimulator) -> Result<()> {
    match get_initial_data() {
        Ok((flight_id, origin, dest, spd)) => match flight_id.trim().parse::<Int>() {
            Ok(id) => {
                match simulator.add_flight(
                    id,
                    origin.trim().to_string(),
                    dest.trim().to_string(),
                    match spd.trim().parse::<Double>() {
                        Ok(speed) => speed,
                        Err(err) => {
                            return Err(Error::ServerError(format!(
                                "Error al parsear la velocidad: {}",
                                err
                            )))
                        }
                    },
                ) {
                    Ok(_) => println!("Vuelo añadido exitosamente"),
                    Err(e) => println!("Error al añadir vuelo: {}", e),
                }
            }
            Err(_) => println!("Error: El ID de vuelo debe ser un entero válido"),
        },
        Err(e) => return Err(e),
    }
    Ok(())
}

fn check_if_there_are_flights(simulator: &FlightSimulator) -> bool {
    if simulator.get_all_flights().is_empty() {
        println!("No hay vuelos activos");
        return false;
    }
    true
}

fn print_flight_data(flight: LiveFlightData) {
    println!("\nInformación del vuelo {}:", flight.flight_id);
    println!("Origen: {}", flight.orig);
    println!("Destino: {}", flight.dest);
    println!("Estado: {:?}", flight.state);
    println!(
        "Posición actual: ({:.4}, {:.4})",
        flight.lat(),
        flight.lon()
    );
    println!("Altitud: {:.2} ft", flight.altitude_ft);
    println!("Velocidad actual: {:.2} km/h", flight.get_spd());
    println!("Velocidad promedio: {:.2} km/h", flight.avg_spd());
    println!("Combustible restante: {:.2} %", flight.fuel);
}

fn handle_view_flight(simulator: &FlightSimulator) -> Result<()> {
    if !check_if_there_are_flights(simulator) {
        return Ok(());
    }

    println!("Ingrese el ID de vuelo:");
    let flight_id = read_line()?;

    match flight_id.trim().parse::<Int>() {
        Ok(id) => match simulator.get_flight_data(id) {
            Some(flight) => {
                print_flight_data(flight);
            }
            None => println!("Vuelo no encontrado"),
        },
        Err(_) => {
            return Err(Error::ServerError(
                "El ID de vuelo debe ser un número entero válido".to_string(),
            ))
        }
    }
    Ok(())
}

fn handle_view_all_flights(simulator: &FlightSimulator) {
    if !check_if_there_are_flights(simulator) {
        return;
    }

    let flights = simulator.get_all_flights();
    println!("\nVuelos activos:");
    for flight in flights {
        println!("\nVuelo {}:", flight.flight_id);
        println!("  Origen: {}", flight.orig);
        println!("  Destino: {}", flight.dest);
        println!("  Estado: {:?}", flight.state);
    }
}

fn handle_view_airports(simulator: &FlightSimulator) {
    println!("Aeropuertos disponibles:");
    for (code, airport) in simulator.airports.iter() {
        println!(
            "{}: {} ({}, {})",
            code, airport.name, airport.municipality, airport.country.name
        );
    }
}
