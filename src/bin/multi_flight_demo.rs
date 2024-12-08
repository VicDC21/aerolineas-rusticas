use aerolineas_rusticas::simulator::cli::{run_sim, FlightConfig};

/// Vuelos de ejemplo.
pub const FLIGHT_CONFIGS: [FlightConfig; 38] = [
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
    FlightConfig {
        flight_id: 923456,
        origin: "SABE",
        destination: "KMSP",
        spd: 860.0,
    },
    FlightConfig {
        flight_id: 934567,
        origin: "KMSP",
        destination: "SABE",
        spd: 890.0,
    },
    FlightConfig {
        flight_id: 945678,
        origin: "SABE",
        destination: "SBBR",
        spd: 920.0,
    },
    FlightConfig {
        flight_id: 956789,
        origin: "SBBR",
        destination: "SABE",
        spd: 930.0,
    },
    FlightConfig {
        flight_id: 967890,
        origin: "SABE",
        destination: "SBCF",
        spd: 910.0,
    },
    FlightConfig {
        flight_id: 978901,
        origin: "SBCF",
        destination: "SABE",
        spd: 940.0,
    },
    FlightConfig {
        flight_id: 989012,
        origin: "SABE",
        destination: "SBEG",
        spd: 915.0,
    },
    FlightConfig {
        flight_id: 990123,
        origin: "SBEG",
        destination: "SABE",
        spd: 935.0,
    },
    FlightConfig {
        flight_id: 991234,
        origin: "SABE",
        destination: "EGLL",
        spd: 950.0,
    },
    FlightConfig {
        flight_id: 992345,
        origin: "EGLL",
        destination: "SABE",
        spd: 945.0,
    },
    FlightConfig {
        flight_id: 993456,
        origin: "SABE",
        destination: "KLAX",
        spd: 960.0,
    },
    FlightConfig {
        flight_id: 994567,
        origin: "KLAX",
        destination: "SABE",
        spd: 970.0,
    },
    // Aca empiezan los vuelos de KMSP
    FlightConfig {
        flight_id: 100001,
        origin: "KMSP",
        destination: "SABE",
        spd: 870.0,
    },
    FlightConfig {
        flight_id: 100002,
        origin: "KMSP",
        destination: "EGLL",
        spd: 880.0,
    },
    FlightConfig {
        flight_id: 100003,
        origin: "KMSP",
        destination: "RJAA",
        spd: 860.0,
    },
    FlightConfig {
        flight_id: 100004,
        origin: "KMSP",
        destination: "KLAX",
        spd: 840.0,
    },
    FlightConfig {
        flight_id: 100005,
        origin: "KMSP",
        destination: "LEMD",
        spd: 850.0,
    },
    FlightConfig {
        flight_id: 100007,
        origin: "KMSP",
        destination: "SBBR",
        spd: 910.0,
    },
    FlightConfig {
        flight_id: 100008,
        origin: "KMSP",
        destination: "SBCF",
        spd: 895.0,
    },
    FlightConfig {
        flight_id: 100009,
        origin: "KMSP",
        destination: "SBEG",
        spd: 930.0,
    },
    FlightConfig {
        flight_id: 100010,
        origin: "KMSP",
        destination: "UNKL",
        spd: 870.0,
    },
    FlightConfig {
        flight_id: 100011,
        origin: "SABE",
        destination: "KMSP",
        spd: 860.0,
    },
    FlightConfig {
        flight_id: 100012,
        origin: "EGAA",
        destination: "KMSP",
        spd: 850.0,
    },
    FlightConfig {
        flight_id: 100013,
        origin: "FACT",
        destination: "KMSP",
        spd: 880.0,
    },
    FlightConfig {
        flight_id: 100014,
        origin: "SBBR",
        destination: "KMSP",
        spd: 895.0,
    },
    FlightConfig {
        flight_id: 100015,
        origin: "SBCF",
        destination: "KMSP",
        spd: 910.0,
    },
    FlightConfig {
        flight_id: 100016,
        origin: "SBEG",
        destination: "KMSP",
        spd: 920.0,
    },
    FlightConfig {
        flight_id: 100017,
        origin: "EGLL",
        destination: "KMSP",
        spd: 870.0,
    },
    FlightConfig {
        flight_id: 100018,
        origin: "KLAX",
        destination: "KMSP",
        spd: 860.0,
    },
    FlightConfig {
        flight_id: 100019,
        origin: "LEMD",
        destination: "KMSP",
        spd: 840.0,
    },
];

fn main() {
    if let Err(err) = run_sim(&FLIGHT_CONFIGS) {
        println!("{}", err);
    }
}
