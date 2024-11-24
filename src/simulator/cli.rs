use crate::{
    client::cli::Client, protocol::errors::error::Error,
    simulator::flight_simulator::FlightSimulator,
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
    println!("ID de vuelo:");
    let mut flight_id = String::new();
    std::io::stdin().read_line(&mut flight_id).unwrap();

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
        match flight_id.trim().parse::<i32>() {
            Ok(id) => match simulator.add_flight(
                id,
                origin.trim().to_string(),
                dest.trim().to_string(),
                speed,
            ) {
                Ok(_) => println!("Vuelo añadido exitosamente"),
                Err(e) => println!("Error al añadir vuelo: {}", e),
            },
            Err(_) => println!("Error: El ID de vuelo debe ser un entero válido"),
        }
    } else {
        println!("Error: La velocidad debe ser un número válido");
    }
}

fn handle_view_flight(simulator: &FlightSimulator) {
    println!("Ingrese el ID de vuelo:");
    let mut flight_id = String::new();
    std::io::stdin().read_line(&mut flight_id).unwrap();

    match flight_id.trim().parse::<i32>() {
        Ok(id) => match simulator.get_flight_data(id) {
            Some(flight) => {
                println!("\nInformación del vuelo {}:", flight.flight_id);
                println!("Origen: {}", flight.orig);
                println!("Destino: {}", flight.dest);
                println!("Estado: {:?}", flight.state);
                println!(
                    "Posición actual: ({:.2}, {:.2})",
                    flight.lat(),
                    flight.lon()
                );
                println!("Altitud: {:.2} pies", flight.altitude_ft);
                println!("Velocidad actual: {:.2} km/h", flight.spd);
                println!("Velocidad promedio: {:.2} km/h", flight.avg_spd());
            }
            None => println!("Vuelo no encontrado"),
        },
        Err(_) => println!("Error: El ID de vuelo debe ser un número entero válido"),
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
        println!("\nVuelo {}:", flight.flight_id);
        println!("  Origen: {}", flight.orig);
        println!("  Destino: {}", flight.dest);
        println!("  Estado: {:?}", flight.state);
        println!("  Tipo: {:?}", flight.flight_type);
    }
}

fn handle_view_airports(simulator: &FlightSimulator) {
    println!("Aeropuertos disponibles:");
    for (code, airport) in simulator.airports.iter() {
        println!("{}: {} ({})", code, airport.name, airport.municipality);
    }
}
