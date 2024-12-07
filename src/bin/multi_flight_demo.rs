use aerolineas_rusticas::simulator::cli::{run_sim, FlightConfig};

/// Vuelos de ejemplo.
pub const FLIGHT_CONFIGS: [FlightConfig; 8] = [
    FlightConfig {
        flight_id: 123456,
        origin: "SABE",
        destination: "EGAA",
        spd: 850.0,
    },
    FlightConfig {
        flight_id: 234567,
        origin: "SABE",
        destination: "FACT",
        spd: 900.0,
    },
    FlightConfig {
        flight_id: 345678,
        origin: "UNKL",
        destination: "SABE",
        spd: 950.0,
    },
    FlightConfig {
        flight_id: 456789,
        origin: "UNKL",
        destination: "EGAA",
        spd: 800.0,
    },
    FlightConfig {
        flight_id: 567890,
        origin: "UNKL",
        destination: "FACT",
        spd: 825.0,
    },
    FlightConfig {
        flight_id: 678901,
        origin: "SABE",
        destination: "UNKL",
        spd: 875.0,
    },
    FlightConfig {
        flight_id: 789012,
        origin: "SABE",
        destination: "LEMD",
        spd: 900.0,
    },
    FlightConfig {
        flight_id: 890123,
        origin: "SABE",
        destination: "RJAA",
        spd: 900.0,
    },
];

fn main() {
    if let Err(err) = run_sim(&FLIGHT_CONFIGS) {
        println!("{}", err);
    }
}
