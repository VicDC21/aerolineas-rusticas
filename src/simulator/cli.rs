// src/simulator/cli.rs
use super::flight_simulator::FlightSimulator;
use crate::client::cli::Client;
use crate::protocol::errors::error::Error;

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
                "1" => handle_add_flight(&simulator),
                "2" => handle_view_flight(&simulator),
                "3" => handle_view_all_flights(&simulator),
                "4" => handle_view_airports(&simulator),
                "5" => break Ok(()),
                _ => println!("Opción no válida"),
            }
        },
        Err(e) => Err(e),
    }
}

fn handle_add_flight(simulator: &FlightSimulator) {
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
    } else {
        println!("Error: La velocidad debe ser un número válido");
    }
}

fn handle_view_flight(simulator: &FlightSimulator) {
    println!("Ingrese el número de vuelo:");
    let mut flight_num = String::new();
    std::io::stdin().read_line(&mut flight_num).unwrap();

    match simulator.get_flight_data(flight_num.trim()) {
        Some(flight) => {
            println!("\nInformación del vuelo {}:", flight.flight_number);
            println!(
                "Origen: {} ({})",
                flight.origin_airport.name, flight.origin_airport.ident
            );
            println!(
                "Destino: {} ({})",
                flight.destination_airport.name, flight.destination_airport.ident
            );
            println!("Estado: {:?}", flight.status);
            println!(
                "Posición actual: ({:.2}, {:.2})",
                flight.current_position.0, flight.current_position.1
            );
            println!("Altitud: {:.2} pies", flight.altitude);
            println!("Velocidad actual: {:.2} km/h", flight.current_speed);
            println!("Nivel de combustible: {:.2}%", flight.fuel_level);
        }
        None => println!("Vuelo no encontrado"),
    }
}

fn handle_view_all_flights(simulator: &FlightSimulator) {
    let flights = simulator.get_all_flights();
    if flights.is_empty() {
        println!("No hay vuelos activos");
        return;
    }

    println!("\nVuelos activos:");
    for flight in flights {
        println!("\nVuelo {}:", flight.flight_number);
        println!(
            "  Origen: {} ({})",
            flight.origin_airport.name, flight.origin_airport.ident
        );
        println!(
            "  Destino: {} ({})",
            flight.destination_airport.name, flight.destination_airport.ident
        );
        println!("  Estado: {:?}", flight.status);
    }
}

fn handle_view_airports(simulator: &FlightSimulator) {
    println!("Aeropuertos disponibles:");
    for (code, airport) in simulator.airports.iter() {
        println!("{}: {} ({})", code, airport.name, airport.municipality);
    }
}
