use aerolineas_rusticas::simulator::cli::{run_sim, FlightConfig};

/// Vuelos de ejemplo.
pub const FLIGHT_CONFIGS: [FlightConfig; 38] = [
    FlightConfig {
        flight_id: 123456,
        origin: "AEP",      // Aeroparque Jorge Newbery. Buenos Aires, Argentina
        destination: "BFS", // Belfast International Airport. Belfast, Northern Ireland
        spd: 850.0,
    },
    FlightConfig {
        flight_id: 234567,
        origin: "AEP",      // Aeroparque Jorge Newbery. Buenos Aires, Argentina
        destination: "CPT", // Cape Town International Airport. Cape Town, South Africa
        spd: 900.0,
    },
    FlightConfig {
        flight_id: 345678,
        origin: "KJA",      // Yemelyanovo International Airport. Krasnoyarsk, Russia
        destination: "AEP", // Aeroparque Jorge Newbery. Buenos Aires, Argentina
        spd: 950.0,
    },
    FlightConfig {
        flight_id: 456789,
        origin: "KJA",      // Yemelyanovo International Airport. Krasnoyarsk, Russia
        destination: "BFS", // Belfast International Airport. Belfast, Northern Ireland
        spd: 800.0,
    },
    FlightConfig {
        flight_id: 567890,
        origin: "KJA",      // Yemelyanovo International Airport. Krasnoyarsk, Russia
        destination: "CPT", // Cape Town International Airport. Cape Town, South Africa
        spd: 825.0,
    },
    FlightConfig {
        flight_id: 678901,
        origin: "AEP",      // Aeroparque Jorge Newbery. Buenos Aires, Argentina
        destination: "KJA", // Yemelyanovo International Airport. Krasnoyarsk, Russia
        spd: 875.0,
    },
    FlightConfig {
        flight_id: 789012,
        origin: "AEP",      // Aeroparque Jorge Newbery. Buenos Aires, Argentina
        destination: "MAD", // Adolfo Suárez Madrid–Barajas Airport. Madrid, Spain
        spd: 900.0,
    },
    FlightConfig {
        flight_id: 890123,
        origin: "AEP",      // Aeroparque Jorge Newbery. Buenos Aires, Argentina
        destination: "NRT", // Narita International Airport. Tokyo, Japan
        spd: 900.0,
    },
    FlightConfig {
        flight_id: 923456,
        origin: "AEP",      // Aeroparque Jorge Newbery. Buenos Aires, Argentina
        destination: "MSP", // Minneapolis-Saint Paul International Airport. Minneapolis, Minnesota, USA
        spd: 860.0,
    },
    FlightConfig {
        flight_id: 934567,
        origin: "MSP", // Minneapolis-Saint Paul International Airport. Minneapolis, Minnesota, USA
        destination: "AEP", // Aeroparque Jorge Newbery. Buenos Aires, Argentina
        spd: 890.0,
    },
    FlightConfig {
        flight_id: 945678,
        origin: "AEP",      // Aeroparque Jorge Newbery. Buenos Aires, Argentina
        destination: "BSB", // Brasília International Airport. Brasília, Brazil
        spd: 920.0,
    },
    FlightConfig {
        flight_id: 956789,
        origin: "BSB",      // Brasília International Airport. Brasília, Brazil
        destination: "AEP", // Aeroparque Jorge Newbery. Buenos Aires, Argentina
        spd: 930.0,
    },
    FlightConfig {
        flight_id: 967890,
        origin: "AEP",      // Aeroparque Jorge Newbery. Buenos Aires, Argentina
        destination: "CNF", // Tancredo Neves International Airport. Belo Horizonte, Brazil
        spd: 910.0,
    },
    FlightConfig {
        flight_id: 978901,
        origin: "CNF", // Tancredo Neves International Airport. Belo Horizonte, Brazil
        destination: "AEP", // Aeroparque Jorge Newbery. Buenos Aires, Argentina
        spd: 940.0,
    },
    FlightConfig {
        flight_id: 989012,
        origin: "AEP",      // Aeroparque Jorge Newbery. Buenos Aires, Argentina
        destination: "MAO", // Eduardo Gomes International Airport. Manaus, Brazil
        spd: 915.0,
    },
    FlightConfig {
        flight_id: 990123,
        origin: "MAO",      // Eduardo Gomes International Airport. Manaus, Brazil
        destination: "AEP", // Aeroparque Jorge Newbery. Buenos Aires, Argentina
        spd: 935.0,
    },
    FlightConfig {
        flight_id: 991234,
        origin: "AEP",      // Aeroparque Jorge Newbery. Buenos Aires, Argentina
        destination: "LHR", // London Heathrow Airport. London, England
        spd: 950.0,
    },
    FlightConfig {
        flight_id: 992345,
        origin: "LHR",      // London Heathrow Airport. London, England
        destination: "AEP", // Aeroparque Jorge Newbery. Buenos Aires, Argentina
        spd: 945.0,
    },
    FlightConfig {
        flight_id: 993456,
        origin: "AEP",      // Aeroparque Jorge Newbery. Buenos Aires, Argentina
        destination: "LAX", // Los Angeles International Airport. Los Angeles, California, USA
        spd: 960.0,
    },
    FlightConfig {
        flight_id: 994567,
        origin: "LAX", // Los Angeles International Airport. Los Angeles, California, USA
        destination: "AEP", // Aeroparque Jorge Newbery. Buenos Aires, Argentina
        spd: 970.0,
    },
    FlightConfig {
        flight_id: 100001,
        origin: "MSP", // Minneapolis-Saint Paul International Airport. Minneapolis, Minnesota, USA
        destination: "AEP", // Aeroparque Jorge Newbery. Buenos Aires, Argentina
        spd: 870.0,
    },
    FlightConfig {
        flight_id: 100002,
        origin: "MSP", // Minneapolis-Saint Paul International Airport. Minneapolis, Minnesota, USA
        destination: "LHR", // London Heathrow Airport. London, England
        spd: 880.0,
    },
    FlightConfig {
        flight_id: 100003,
        origin: "MSP", // Minneapolis-Saint Paul International Airport. Minneapolis, Minnesota, USA
        destination: "NRT", // Narita International Airport. Tokyo, Japan
        spd: 860.0,
    },
    FlightConfig {
        flight_id: 100004,
        origin: "MSP", // Minneapolis-Saint Paul International Airport. Minneapolis, Minnesota, USA
        destination: "LAX", // Los Angeles International Airport. Los Angeles, California, USA
        spd: 840.0,
    },
    FlightConfig {
        flight_id: 100005,
        origin: "MSP", // Minneapolis-Saint Paul International Airport. Minneapolis, Minnesota, USA
        destination: "MAD", // Adolfo Suárez Madrid–Barajas Airport. Madrid, Spain
        spd: 850.0,
    },
    FlightConfig {
        flight_id: 100007,
        origin: "MSP", // Minneapolis-Saint Paul International Airport. Minneapolis, Minnesota, USA
        destination: "BSB", // Brasília International Airport. Brasília, Brazil
        spd: 910.0,
    },
    FlightConfig {
        flight_id: 100008,
        origin: "MSP", // Minneapolis-Saint Paul International Airport. Minneapolis, Minnesota, USA
        destination: "CNF", // Tancredo Neves International Airport. Belo Horizonte, Brazil
        spd: 895.0,
    },
    FlightConfig {
        flight_id: 100009,
        origin: "MSP", // Minneapolis-Saint Paul International Airport. Minneapolis, Minnesota, USA
        destination: "MAO", // Eduardo Gomes International Airport. Manaus, Brazil
        spd: 930.0,
    },
    FlightConfig {
        flight_id: 100010,
        origin: "MSP", // Minneapolis-Saint Paul International Airport. Minneapolis, Minnesota, USA
        destination: "KJA", // Yemelyanovo International Airport. Krasnoyarsk, Russia
        spd: 870.0,
    },
    FlightConfig {
        flight_id: 100011,
        origin: "AEP",      // Aeroparque Jorge Newbery. Buenos Aires, Argentina
        destination: "MSP", // Minneapolis-Saint Paul International Airport. Minneapolis, Minnesota, USA
        spd: 860.0,
    },
    FlightConfig {
        flight_id: 100012,
        origin: "BFS",      // Belfast International Airport. Belfast, Northern Ireland
        destination: "MSP", // Minneapolis-Saint Paul International Airport. Minneapolis, Minnesota, USA
        spd: 850.0,
    },
    FlightConfig {
        flight_id: 100013,
        origin: "CPT",      // Cape Town International Airport. Cape Town, South Africa
        destination: "MSP", // Minneapolis-Saint Paul International Airport. Minneapolis, Minnesota, USA
        spd: 880.0,
    },
    FlightConfig {
        flight_id: 100014,
        origin: "BSB",      // Brasília International Airport. Brasília, Brazil
        destination: "MSP", // Minneapolis-Saint Paul International Airport. Minneapolis, Minnesota, USA
        spd: 895.0,
    },
    FlightConfig {
        flight_id: 100015,
        origin: "CNF", // Tancredo Neves International Airport. Belo Horizonte, Brazil
        destination: "MSP", // Minneapolis-Saint Paul International Airport. Minneapolis, Minnesota, USA
        spd: 910.0,
    },
    FlightConfig {
        flight_id: 100016,
        origin: "MAO",      // Eduardo Gomes International Airport. Manaus, Brazil
        destination: "MSP", // Minneapolis-Saint Paul International Airport. Minneapolis, Minnesota, USA
        spd: 920.0,
    },
    FlightConfig {
        flight_id: 100017,
        origin: "LHR",      // London Heathrow Airport. London, England
        destination: "MSP", // Minneapolis-Saint Paul International Airport. Minneapolis, Minnesota, USA
        spd: 870.0,
    },
    FlightConfig {
        flight_id: 100018,
        origin: "LAX", // Los Angeles International Airport. Los Angeles, California, USA
        destination: "MSP", // Minneapolis-Saint Paul International Airport. Minneapolis, Minnesota, USA
        spd: 860.0,
    },
    FlightConfig {
        flight_id: 100019,
        origin: "MAD",      // Adolfo Suárez Madrid–Barajas Airport. Madrid, Spain
        destination: "MSP", // Minneapolis-Saint Paul International Airport. Minneapolis, Minnesota, USA
        spd: 840.0,
    },
];

fn main() {
    if let Err(err) = run_sim(&FLIGHT_CONFIGS) {
        println!("{}", err);
    }
}
