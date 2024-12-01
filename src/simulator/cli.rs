use crate::{
    client::cli::Client,
    protocol::aliases::{
        results::Result,
        types::{Double, Int},
    },
    protocol::errors::error::Error,
    simulator::flight_simulator::FlightSimulator,
};

const MAX_THREADS: usize = 16;

/// Ejecuta el simulador de vuelos.
pub fn run_sim(client: Client) -> Result<()> {
    match FlightSimulator::new(MAX_THREADS, client) {
        Ok(simulator) => loop {
            println!("\nSimulador de Vuelos");
            println!("1. Añadir vuelo");
            println!("2. Ver estado de un vuelo");
            println!("3. Ver todos los vuelos");
            println!("4. Ver aeropuertos disponibles");
            println!("5. Salir");

            let mut input = String::new();
            if let Err(err) = std::io::stdin().read_line(&mut input) {
                break Err(Error::ServerError(format!(
                    "Error al leer la entrada: {}",
                    err
                )));
            }

            match input.trim() {
                "1" => handle_add_flight(&simulator)?,
                "2" => handle_view_flight(&simulator)?,
                "3" => handle_view_all_flights(&simulator),
                "4" => handle_view_airports(&simulator),
                "5" => break Ok(()),
                _ => println!("Opción no válida"),
            }
        },
        Err(e) => Err(e),
    }
}

fn handle_add_flight(simulator: &FlightSimulator) -> Result<()> {
    println!("ID de vuelo:");
    let mut flight_id = String::new();
    if let Err(err) = std::io::stdin().read_line(&mut flight_id) {
        return Err(Error::ServerError(format!(
            "Error al leer la entrada: {}",
            err
        )));
    }

    println!("Código del aeropuerto de origen:");
    let mut origin = String::new();
    if let Err(err) = std::io::stdin().read_line(&mut origin) {
        return Err(Error::ServerError(format!(
            "Error al leer la entrada: {}",
            err
        )));
    }

    println!("Código del aeropuerto de destino:");
    let mut dest = String::new();
    if let Err(err) = std::io::stdin().read_line(&mut dest) {
        return Err(Error::ServerError(format!(
            "Error al leer la entrada: {}",
            err
        )));
    }

    println!("Velocidad promedio (km/h):");
    let mut speed = String::new();
    if let Err(err) = std::io::stdin().read_line(&mut speed) {
        return Err(Error::ServerError(format!(
            "Error al leer la entrada: {}",
            err
        )));
    }

    match speed.trim().parse::<Double>() {
        Ok(speed) => match flight_id.trim().parse::<Int>() {
            Ok(id) => match simulator.add_flight(
                id,
                origin.trim().to_string(),
                dest.trim().to_string(),
                speed,
            ) {
                Ok(_) => println!("Vuelo añadido exitosamente"),
                Err(e) => return Err(Error::ServerError(format!("Error al añadir vuelo: {}", e))),
            },
            Err(_) => {
                return Err(Error::ServerError(
                    "El ID de vuelo debe ser un entero válido".to_string(),
                ))
            }
        },
        Err(_) => {
            return Err(Error::ServerError(
                "La velocidad debe ser un número válido".to_string(),
            ))
        }
    }
    Ok(())
}

fn handle_view_flight(simulator: &FlightSimulator) -> Result<()> {
    println!("Ingrese el ID de vuelo:");
    let mut flight_id = String::new();
    if let Err(err) = std::io::stdin().read_line(&mut flight_id) {
        return Err(Error::ServerError(format!(
            "Error al leer la entrada: {}",
            err
        )));
    }

    match flight_id.trim().parse::<Int>() {
        Ok(id) => match simulator.get_flight_data(id) {
            Some(flight) => {
                println!("\nInformación del vuelo {}:", flight.flight_id);
                println!("Origen: {}", flight.orig);
                println!("Destino: {}", flight.dest);
                println!("Estado: {:?}", flight.state);
                println!(
                    "Posición actual: ({:.4}, {:.4})",
                    flight.lat(),
                    flight.lon()
                );
                println!("Altitud: {:.2} msnm", flight.altitude_ft);
                println!("Velocidad actual: {:.2} km/h", flight.get_spd());
                println!("Velocidad promedio: {:.2} km/h", flight.avg_spd());
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
    let flights = simulator.get_all_flights();
    if flights.is_empty() {
        println!("No hay vuelos activos");
        return;
    }

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
