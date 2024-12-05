use aerolineas_rusticas::{
    client::cli::Client,
    simulator::cli::{run_sim, FlightConfig},
};

/// Vuelos de ejemplo.
pub const FLIGHT_CONFIGS: [FlightConfig; 6] = [
    FlightConfig {
        flight_id: 123456,
        origin: "SAEZ",
        destination: "LEMD",
        spd: 850.0,
    },
    FlightConfig {
        flight_id: 234567,
        origin: "SAEZ",
        destination: "KJFK",
        spd: 900.0,
    },
    FlightConfig {
        flight_id: 345678,
        origin: "SAEZ",
        destination: "RJAA",
        spd: 950.0,
    },
    FlightConfig {
        flight_id: 456789,
        origin: "LFPG",
        destination: "SAEZ",
        spd: 800.0,
    },
    FlightConfig {
        flight_id: 567890,
        origin: "HKJK",
        destination: "SAEZ",
        spd: 825.0,
    },
    FlightConfig {
        flight_id: 678901,
        origin: "YMML",
        destination: "SAEZ",
        spd: 875.0,
    },
];

fn main() {
    let client = Client::default();
    if let Err(err) = run_sim(client, &FLIGHT_CONFIGS) {
        println!("{}", err);
    }
}
