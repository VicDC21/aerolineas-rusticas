//! Módulo para correr la interfaz.

#[cfg(feature = "gui")]
use aerolineas::interface::run::run_app;

fn main() {
    #[cfg(feature = "gui")]
    if let Err(err) = run_app() {
        println!("{}", err);
    }

    #[cfg(not(feature = "gui"))]
    println!("Se quizo ejecutar 'gui', pero la feature relevante no está activada. Prueba con:\n\ncargo run --features \"gui\" gui\n")
}
