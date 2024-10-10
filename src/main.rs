use std::env::args;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

use aerolineas::{client::cli::Client, server::sv};

fn main() {
    let argv = args().collect::<Vec<String>>();
    if argv.len() < 2 {
        println!("ERROR: Hay menos de 2 argumentos...");
        return;
    }

    match argv[1].as_str() {
        "sv" => {
            let _ = sv::run();
        }

        "cli" => {
            let addr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 8080));
            let client = Client::new(addr);
            let _ = client.echo();
        }
        _ => {
            println!("Se debe elegir o 'sv' o 'cli', no '{}'...", argv[1]);
        }
    }
}
