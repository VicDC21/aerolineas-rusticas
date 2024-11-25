use std::env::args;

use aerolineas_rusticas::{
    client::cli::Client,
    protocol::aliases::results::Result,
    server::nodes::{graph::NodesGraph, node::Node},
    simulator::flight_simulator::run_sim,
};

#[cfg(feature = "gui")]
use aerolineas_rusticas::interface::run::run_app;

/// Imprime por pantalla el error
fn print_err(res: Result<()>) {
    if let Err(err) = res {
        println!("{}", err);
    }
}

fn main() {
    let argv = args().collect::<Vec<String>>();
    let how_to_use = "Uso:\n\ncargo run [cli | --features \"gui\" gui | sim | sv | nd [echo]]\n";
    if argv.len() < 2 {
        println!("{}", how_to_use);
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
        "nd" => {
            let id = if argv.len() >= 3 {
                argv[2].parse::<u8>().unwrap_or(0)
            } else {
                0
            };
            if argv.len() == 4 && argv[3] == "echo" {
                print_err(Node::init_in_echo_mode(id))
            } else {
                print_err(Node::init_in_parsing_mode(id))
            }
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
        "sim" => {
            let client = Client::default();
            print_err(run_sim(client));
        }
        _ => {
            println!("{}", how_to_use);
        }
    }
}
