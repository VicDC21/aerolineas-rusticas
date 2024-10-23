use std::env::args;

use aerolineas::interface::run::run_app;
use aerolineas::{client::cli::Client, server::sv::Server};
use aerolineas::protocol::aliases::results::Result;

/// Imprime por pantalla el error
fn print_err(res: Result<_>) {
    if let Err(err) = res {
        println!("{}", err);
    }
}

fn main() {
    let argv = args().collect::<Vec<String>>();
    if argv.len() < 2 {
        println!("ERROR: Hay menos de 2 argumentos...");
        return;
    }

    match argv[1].as_str() {
        "sv" => {
            let mut server = Server::echo_mode();
            print_err(server.listen());
        }

        "cli" => {
            let mut client = Client::new(Server::default_addr());
            print_err(client.echo());
        }
        "gui" => {
            print_err(run_app());
        }
        _ => {
            println!("Se debe elegir o 'sv' o 'cli', no '{}'...", argv[1]);
        }
    }
}
