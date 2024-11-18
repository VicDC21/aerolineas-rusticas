use aerolineas::{
    client::cli::Client, protocol::aliases::results::Result, server::nodes::graph::NodesGraph,
};

#[cfg(feature = "gui")]
use aerolineas::interface::run::run_app;

use std::env::args;

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
            #[cfg(feature = "gui")]
            print_err(run_app());

            #[cfg(not(feature = "gui"))]
            println!("Se quizo ejecutar 'gui', pero la feature relevante no está activada. Prueba con:\n\ncargo run --features \"gui\" gui\n")
        }
        _ => {
            println!(
                "Se debe elegir o 'sv', 'cli' o 'gui', pero no '{}'...",
                argv[1]
            );
        }
    }
}
