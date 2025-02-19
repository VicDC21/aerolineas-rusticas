//! Módulo para correr un nodo.

use {
    protocol::aliases::{results::Result, types::Byte},
    server::nodes::node::Node,
    std::{env::args, net::IpAddr},
};

fn main() {
    let argv = args().collect::<Vec<String>>();

    if argv.len() >= 2 {
        if argv[1] == "new" && argv.len() >= 4 {
            // "target/debug/nd.exe" new <id> <ip> [echo]
            match argv[2].parse::<Byte>() {
                Ok(id) => {
                    if argv[3].parse::<IpAddr>().is_ok() {
                        println!("Nodo nuevo con id {} y dirección IP {}.", id, argv[3]);
                        if argv.len() == 5 && argv[4].to_ascii_lowercase() == "echo" {
                            // "target/debug/nd.exe" new <id> <ip> echo
                            print_err(Node::init_new_in_echo_mode(id, &argv[4]))
                        } else {
                            // "target/debug/nd.exe" new <id> <ip>
                            print_err(Node::init_new_in_parsing_mode(id, &argv[4]))
                        }
                    } else {
                        println!("La IP no es válida.");
                    }
                }
                Err(_) => {
                    println!("El id debe ser un número entero entre 0 y 255.");
                }
            }
        } else {
            // "target/debug/nd.exe" <id> [echo]
            match argv[1].parse::<Byte>() {
                Ok(id) => {
                    if argv.len() == 3 && argv[2].to_ascii_lowercase() == "echo" {
                        // "target/debug/nd.exe" <id> echo
                        print_err(Node::init_in_echo_mode(id))
                    } else {
                        // "target/debug/nd.exe" <id>
                        print_err(Node::init_in_parsing_mode(id))
                    }
                }
                Err(_) => {
                    println!("El id debe ser un número entero entre 0 y 255.");
                }
            }
        }
    } else {
        println!("Uso:\n\ncargo run -p server --bin nd [new] <id> <ip> [echo]\n");
    };
}

fn print_err(res: Result<()>) {
    if let Err(err) = res {
        println!("{}", err);
    }
}
