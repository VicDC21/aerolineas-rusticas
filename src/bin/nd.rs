//! Módulo para correr un nodo.

use std::env::args;

use aerolineas_rusticas::{protocol::aliases::types::Byte, server::nodes::node::Node};

fn main() {
    let argv = args().collect::<Vec<String>>();

    if argv.len() >= 2 {
        match argv[1].parse::<Byte>() {
            Ok(id) => {
                if argv.len() == 3 && argv[2].to_ascii_lowercase() == "echo" {
                    if let Err(err) = Node::init_in_echo_mode(id) {
                        println!("{}", err);
                    }
                } else if let Err(err) = Node::init_in_parsing_mode(id) {
                    println!("{}", err);
                }
            }
            Err(_) => {
                println!("El id debe ser un número entero entre 0 y 255.");
            }
        }
    } else {
        println!("Uso:\n\ncargo run nd <id> [echo]\n");
    };
}
