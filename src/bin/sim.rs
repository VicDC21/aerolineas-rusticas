//! Módulo para correr el simulador de vuelos.

use aerolineas_rusticas::{client::cli::Client, simulator::cli::run_sim};

fn main() {
    let client = Client::default();
    if let Err(err) = run_sim(client, &[]) {
        println!("{}", err);
    }
}
