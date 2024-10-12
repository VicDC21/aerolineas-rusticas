use std::env::args;

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
            let _ = server.listen();
        }

        "cli" => {
            let client = Client::new(Server::default_addr());
            let _ = client.echo();
        }
        _ => {
            println!("Se debe elegir o 'sv' o 'cli', no '{}'...", argv[1]);
        }
    }
}
