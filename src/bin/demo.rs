use {
    aerolineas_rusticas::simulator::cli::{run_sim, FlightConfig},
    std::{fs::File, io::BufReader, path::Path},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let flight_configs: Vec<FlightConfig> = serde_json::from_reader(BufReader::new(File::open(
        Path::new("media/flights/flights_configs.json"),
    )?))?;
    if let Err(err) = run_sim(&flight_configs) {
        println!("{}", err);
    }

    Ok(())
}
