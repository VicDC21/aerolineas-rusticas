//! Módulo para correr la interfaz.

use interface::run::run_app;

fn main() {
    if let Err(err) = run_app() {
        println!("{}", err);
    }
}
