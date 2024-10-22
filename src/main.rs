use std::env::args;

use aerolineas::interface::run::run_app;
use aerolineas::{client::cli::Client, server::sv::Server};

fn main() {
    let argv = args().collect::<Vec<String>>();
    if argv.len() < 2 {
        println!("ERROR: Hay menos de 2 argumentos...");
        return;
    }

    match argv[1].as_str() {
        "sv" => {
            let mut server = Server::echo_mode();
            if let Err(err) = server.listen() {
                println!("{}", err);
            }
        }

        "cli" => {
            let mut client = Client::new(Server::default_addr());
            if let Err(err) = client.echo() {
                println!("{}", err);
            }
        }
        "gui" => {
            if let Err(err) = run_app() {
                println!("{}", err);
            }
        }
        _ => {
            println!("Se debe elegir o 'sv' o 'cli', no '{}'...", argv[1]);
        }
    }
}
