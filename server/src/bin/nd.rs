//! Módulo para correr un nodo.

use {
    protocol::aliases::{results::Result, types::Byte},
    server::nodes::node::Node,
    std::{env::args, net::IpAddr, path::Path},
};

fn main() {
    let argv = args().collect::<Vec<String>>();

    if argv.len() >= 2 {
        if argv[1] == "new" && argv.len() >= 3 {
            // "target/debug/nd.exe" new <id> [<ip>] [echo]
            match argv[2].parse::<Byte>() {
                Ok(id) => {
                    let (ip, echo_idx) = if argv.len() >= 4 && argv[3].parse::<IpAddr>().is_ok() {
                        let ip_str = argv[3].clone();
                        let ip_addr: IpAddr = match ip_str.parse() {
                            Ok(ip) => ip,
                            Err(_) => {
                                println!("La IP no es válida para la ejecución local.");
                                return;
                            }
                        };
                        if running_in_docker() || ip_addr.is_loopback() {
                            (ip_str, 4)
                        } else {
                            println!("La IP no es válida para la ejecución local.");
                            return;
                        }
                    } else {
                        if argv.len() >= 4 && !argv[3].eq_ignore_ascii_case("echo") {
                            println!("La IP no es válida.");
                            return;
                        }
                        (format!("127.0.0.{id}"), 3)
                    };
                    println!("Nodo nuevo con id {id} y dirección IP {ip}.");
                    if argv
                        .get(echo_idx)
                        .is_some_and(|s| s.eq_ignore_ascii_case("echo"))
                    {
                        // "target/debug/nd.exe" new <id> [<ip>] echo
                        print_err(Node::init_new_in_echo_mode(id, &ip))
                    } else {
                        // "target/debug/nd.exe" new <id> [<ip>]
                        print_err(Node::init_new_in_parsing_mode(id, &ip))
                    }
                }
                Err(_) => {
                    println!("El id debe ser un número entero entre 0 y 255.");
                }
            }
        } else if argv[1] == "delete" && argv.len() == 3 {
            // "target/debug/nd.exe" delete <id>
            match argv[2].parse::<Byte>() {
                Ok(id) => {
                    println!("Nodo a eliminar: {}", argv[2]);
                    print_err(Node::delete_node(id));
                }
                Err(_) => {
                    println!("El id debe ser un número entero entre 0 y 255.");
                }
            }
        } else {
            // "target/debug/nd.exe" <id> [echo]
            match argv[1].parse::<Byte>() {
                Ok(id) => {
                    if argv.len() == 3 && argv[2].eq_ignore_ascii_case("echo") {
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
        println!("Uso:\n\ncargo run -p server --bin nd [new]/[delete] <id> [<ip>] [echo]\n");
    };
}

fn print_err(res: Result<()>) {
    if let Err(err) = res {
        println!("{err}");
    }
}

fn running_in_docker() -> bool {
    Path::new("/.dockerenv").exists()
}
