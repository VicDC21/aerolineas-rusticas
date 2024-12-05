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
        avg_speed: 850.0,
    },
    FlightConfig {
        flight_id: 234567,
        origin: "SBGR",
        destination: "KJFK",
        avg_speed: 900.0,
    },
    FlightConfig {
        flight_id: 345678,
        origin: "KLAX",
        destination: "RJAA",
        avg_speed: 950.0,
    },
    FlightConfig {
        flight_id: 456789,
        origin: "LFPG",
        destination: "SVMI",
        avg_speed: 800.0,
    },
    FlightConfig {
        flight_id: 567890,
        origin: "HKJK",
        destination: "EGLL",
        avg_speed: 825.0,
    },
    FlightConfig {
        flight_id: 678901,
        origin: "YMML",
        destination: "WSSS",
        avg_speed: 875.0,
    },
];

fn main() {
    let client = Client::default();
    if let Err(err) = run_sim(client, &FLIGHT_CONFIGS) {
        println!("{}", err);
    }
}
