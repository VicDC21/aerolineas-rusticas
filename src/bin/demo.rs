use {
    aerolineas_rusticas::simulator::cli::{run_sim, FlightConfig},
    serde::Deserialize,
    std::{fs::File, io::BufReader, path::Path},
};

#[derive(Deserialize)]
struct FlightConfigs {
    flight_configs: Vec<FlightConfig>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = Path::new("media/flights/flights_configs.json");
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let flight_configs: FlightConfigs = serde_json::from_reader(reader)?;

    if let Err(err) = run_sim(&flight_configs.flight_configs) {
        println!("{}", err);
    }

    Ok(())
}
