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
    },
    FlightConfig {
        flight_id: 234567,
        origin: "SBGR",
        destination: "KJFK",
    },
    FlightConfig {
        flight_id: 345678,
        origin: "KLAX",
        destination: "RJAA",
    },
    FlightConfig {
        flight_id: 456789,
        origin: "LFPG",
        destination: "SVMI",
    },
    FlightConfig {
        flight_id: 567890,
        origin: "HKJK",
        destination: "EGLL",
    },
    FlightConfig {
        flight_id: 678901,
        origin: "YMML",
        destination: "WSSS",
    },
];

fn main() {
    let client = Client::default();
    if let Err(err) = run_sim(client, &FLIGHT_CONFIGS) {
        println!("{}", err);
    }
}
