use aerolineas_rusticas::simulator::cli::{run_sim, FlightConfig};

/// Vuelos de ejemplo.
pub const FLIGHT_CONFIGS: [FlightConfig; 38] = [
    FlightConfig {
        flight_id: 123456,
        origin: "SABE",      // Aeroparque Jorge Newbery. Buenos Aires, Argentina
        destination: "EGAA", // Belfast International Airport. Belfast, Northern Ireland
        spd: 850.0,
    },
    FlightConfig {
        flight_id: 234567,
        origin: "SABE",      // Aeroparque Jorge Newbery. Buenos Aires, Argentina
        destination: "FACT", // Cape Town International Airport. Cape Town, South Africa
        spd: 900.0,
    },
    FlightConfig {
        flight_id: 345678,
        origin: "UNKL",      // Yemelyanovo International Airport. Krasnoyarsk, Russia
        destination: "SABE", // Aeroparque Jorge Newbery. Buenos Aires, Argentina
        spd: 950.0,
    },
    FlightConfig {
        flight_id: 456789,
        origin: "UNKL",      // Yemelyanovo International Airport. Krasnoyarsk, Russia
        destination: "EGAA", // Belfast International Airport. Belfast, Northern Ireland
        spd: 800.0,
    },
    FlightConfig {
        flight_id: 567890,
        origin: "UNKL",      // Yemelyanovo International Airport. Krasnoyarsk, Russia
        destination: "FACT", // Cape Town International Airport. Cape Town, South Africa
        spd: 825.0,
    },
    FlightConfig {
        flight_id: 678901,
        origin: "SABE",      // Aeroparque Jorge Newbery. Buenos Aires, Argentina
        destination: "UNKL", // Yemelyanovo International Airport. Krasnoyarsk, Russia
        spd: 875.0,
    },
    FlightConfig {
        flight_id: 789012,
        origin: "SABE",      // Aeroparque Jorge Newbery. Buenos Aires, Argentina
        destination: "LEMD", // Adolfo Suárez Madrid–Barajas Airport. Madrid, Spain
        spd: 900.0,
    },
    FlightConfig {
        flight_id: 890123,
        origin: "SABE",      // Aeroparque Jorge Newbery. Buenos Aires, Argentina
        destination: "RJAA", // Narita International Airport. Tokyo, Japan
        spd: 900.0,
    },
    FlightConfig {
        flight_id: 923456,
        origin: "SABE",      // Aeroparque Jorge Newbery. Buenos Aires, Argentina
        destination: "KMSP", // Minneapolis-Saint Paul International Airport. Minneapolis, Minnesota, USA
        spd: 860.0,
    },
    FlightConfig {
        flight_id: 934567,
        origin: "KMSP", // Minneapolis-Saint Paul International Airport. Minneapolis, Minnesota, USA
        destination: "SABE", // Aeroparque Jorge Newbery. Buenos Aires, Argentina
        spd: 890.0,
    },
    FlightConfig {
        flight_id: 945678,
        origin: "SABE",      // Aeroparque Jorge Newbery. Buenos Aires, Argentina
        destination: "SBBR", // Brasília International Airport. Brasília, Brazil
        spd: 920.0,
    },
    FlightConfig {
        flight_id: 956789,
        origin: "SBBR",      // Brasília International Airport. Brasília, Brazil
        destination: "SABE", // Aeroparque Jorge Newbery. Buenos Aires, Argentina
        spd: 930.0,
    },
    FlightConfig {
        flight_id: 967890,
        origin: "SABE",      // Aeroparque Jorge Newbery. Buenos Aires, Argentina
        destination: "SBCF", // Tancredo Neves International Airport. Belo Horizonte, Brazil
        spd: 910.0,
    },
    FlightConfig {
        flight_id: 978901,
        origin: "SBCF", // Tancredo Neves International Airport. Belo Horizonte, Brazil
        destination: "SABE", // Aeroparque Jorge Newbery. Buenos Aires, Argentina
        spd: 940.0,
    },
    FlightConfig {
        flight_id: 989012,
        origin: "SABE",      // Aeroparque Jorge Newbery. Buenos Aires, Argentina
        destination: "SBEG", // Eduardo Gomes International Airport. Manaus, Brazil
        spd: 915.0,
    },
    FlightConfig {
        flight_id: 990123,
        origin: "SBEG",      // Eduardo Gomes International Airport. Manaus, Brazil
        destination: "SABE", // Aeroparque Jorge Newbery. Buenos Aires, Argentina
        spd: 935.0,
    },
    FlightConfig {
        flight_id: 991234,
        origin: "SABE",      // Aeroparque Jorge Newbery. Buenos Aires, Argentina
        destination: "EGLL", // London Heathrow Airport. London, England
        spd: 950.0,
    },
    FlightConfig {
        flight_id: 992345,
        origin: "EGLL",      // London Heathrow Airport. London, England
        destination: "SABE", // Aeroparque Jorge Newbery. Buenos Aires, Argentina
        spd: 945.0,
    },
    FlightConfig {
        flight_id: 993456,
        origin: "SABE",      // Aeroparque Jorge Newbery. Buenos Aires, Argentina
        destination: "KLAX", // Los Angeles International Airport. Los Angeles, California, USA
        spd: 960.0,
    },
    FlightConfig {
        flight_id: 994567,
        origin: "KLAX", // Los Angeles International Airport. Los Angeles, California, USA
        destination: "SABE", // Aeroparque Jorge Newbery. Buenos Aires, Argentina
        spd: 970.0,
    },
    FlightConfig {
        flight_id: 100001,
        origin: "KMSP", // Minneapolis-Saint Paul International Airport. Minneapolis, Minnesota, USA
        destination: "SABE", // Aeroparque Jorge Newbery. Buenos Aires, Argentina
        spd: 870.0,
    },
    FlightConfig {
        flight_id: 100002,
        origin: "KMSP", // Minneapolis-Saint Paul International Airport. Minneapolis, Minnesota, USA
        destination: "EGLL", // London Heathrow Airport. London, England
        spd: 880.0,
    },
    FlightConfig {
        flight_id: 100003,
        origin: "KMSP", // Minneapolis-Saint Paul International Airport. Minneapolis, Minnesota, USA
        destination: "RJAA", // Narita International Airport. Tokyo, Japan
        spd: 860.0,
    },
    FlightConfig {
        flight_id: 100004,
        origin: "KMSP", // Minneapolis-Saint Paul International Airport. Minneapolis, Minnesota, USA
        destination: "KLAX", // Los Angeles International Airport. Los Angeles, California, USA
        spd: 840.0,
    },
    FlightConfig {
        flight_id: 100005,
        origin: "KMSP", // Minneapolis-Saint Paul International Airport. Minneapolis, Minnesota, USA
        destination: "LEMD", // Adolfo Suárez Madrid–Barajas Airport. Madrid, Spain
        spd: 850.0,
    },
    FlightConfig {
        flight_id: 100007,
        origin: "KMSP", // Minneapolis-Saint Paul International Airport. Minneapolis, Minnesota, USA
        destination: "SBBR", // Brasília International Airport. Brasília, Brazil
        spd: 910.0,
    },
    FlightConfig {
        flight_id: 100008,
        origin: "KMSP", // Minneapolis-Saint Paul International Airport. Minneapolis, Minnesota, USA
        destination: "SBCF", // Tancredo Neves International Airport. Belo Horizonte, Brazil
        spd: 895.0,
    },
    FlightConfig {
        flight_id: 100009,
        origin: "KMSP", // Minneapolis-Saint Paul International Airport. Minneapolis, Minnesota, USA
        destination: "SBEG", // Eduardo Gomes International Airport. Manaus, Brazil
        spd: 930.0,
    },
    FlightConfig {
        flight_id: 100010,
        origin: "KMSP", // Minneapolis-Saint Paul International Airport. Minneapolis, Minnesota, USA
        destination: "UNKL", // Yemelyanovo International Airport. Krasnoyarsk, Russia
        spd: 870.0,
    },
    FlightConfig {
        flight_id: 100011,
        origin: "SABE",      // Aeroparque Jorge Newbery. Buenos Aires, Argentina
        destination: "KMSP", // Minneapolis-Saint Paul International Airport. Minneapolis, Minnesota, USA
        spd: 860.0,
    },
    FlightConfig {
        flight_id: 100012,
        origin: "EGAA", // Belfast International Airport. Belfast, Northern Ireland
        destination: "KMSP", // Minneapolis-Saint Paul International Airport. Minneapolis, Minnesota, USA
        spd: 850.0,
    },
    FlightConfig {
        flight_id: 100013,
        origin: "FACT", // Cape Town International Airport. Cape Town, South Africa
        destination: "KMSP", // Minneapolis-Saint Paul International Airport. Minneapolis, Minnesota, USA
        spd: 880.0,
    },
    FlightConfig {
        flight_id: 100014,
        origin: "SBBR",      // Brasília International Airport. Brasília, Brazil
        destination: "KMSP", // Minneapolis-Saint Paul International Airport. Minneapolis, Minnesota, USA
        spd: 895.0,
    },
    FlightConfig {
        flight_id: 100015,
        origin: "SBCF", // Tancredo Neves International Airport. Belo Horizonte, Brazil
        destination: "KMSP", // Minneapolis-Saint Paul International Airport. Minneapolis, Minnesota, USA
        spd: 910.0,
    },
    FlightConfig {
        flight_id: 100016,
        origin: "SBEG",      // Eduardo Gomes International Airport. Manaus, Brazil
        destination: "KMSP", // Minneapolis-Saint Paul International Airport. Minneapolis, Minnesota, USA
        spd: 920.0,
    },
    FlightConfig {
        flight_id: 100017,
        origin: "EGLL",      // London Heathrow Airport. London, England
        destination: "KMSP", // Minneapolis-Saint Paul International Airport. Minneapolis, Minnesota, USA
        spd: 870.0,
    },
    FlightConfig {
        flight_id: 100018,
        origin: "KLAX", // Los Angeles International Airport. Los Angeles, California, USA
        destination: "KMSP", // Minneapolis-Saint Paul International Airport. Minneapolis, Minnesota, USA
        spd: 860.0,
    },
    FlightConfig {
        flight_id: 100019,
        origin: "LEMD",      // Adolfo Suárez Madrid–Barajas Airport. Madrid, Spain
        destination: "KMSP", // Minneapolis-Saint Paul International Airport. Minneapolis, Minnesota, USA
        spd: 840.0,
    },
];

fn main() {
    if let Err(err) = run_sim(&FLIGHT_CONFIGS) {
        println!("{}", err);
    }
}
