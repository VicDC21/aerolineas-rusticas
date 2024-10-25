use std::env::args;

use aerolineas::interface::run::run_app;
use aerolineas::protocol::aliases::results::Result;
use aerolineas::{client::cli::Client, server::nodes::graph::NodesGraph};

/// Imprime por pantalla el error
fn print_err(res: Result<()>) {
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
            let mut graph = if argv.len() == 3 && argv[2] == "echo" {
                NodesGraph::echo_mode()
            } else {
                NodesGraph::parsing_mode()
            };
            print_err(graph.init());
        }

        "cli" => {
            let mut client = Client::default();
            print_err(client.echo());
        }
        "gui" => {
            print_err(run_app());
        }
        _ => {
            println!(
                "Se debe elegir o 'sv', 'cli' o 'gui', pero no '{}'...",
                argv[1]
            );
        }
    }
}
