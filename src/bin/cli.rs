//! Módulo para correr el cliente.

use aerolineas::client::cli::Client;

fn main() {
    let mut client = Client::default();
    if let Err(err) = client.echo() {
        println!("{}", err);
    }
}
