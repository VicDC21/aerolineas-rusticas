//! Módulo para correr los nodos.

use std::env::args;

use aerolineas::server::nodes::graph::NodesGraph;

fn main() {
    let argv = args().collect::<Vec<String>>();

    let mut graph = if argv.len() >= 2 && argv[1].eq_ignore_ascii_case("echo") {
        NodesGraph::echo_mode()
    } else {
        NodesGraph::parsing_mode()
    };
    if let Err(err) = graph.init() {
        println!("{}", err);
    }
}
