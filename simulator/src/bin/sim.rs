//! Módulo para correr el simulador de vuelos.

use simulator::cli::run_sim;

fn main() {
    if let Err(err) = run_sim(&[]) {
        println!("{}", err);
    }
}
