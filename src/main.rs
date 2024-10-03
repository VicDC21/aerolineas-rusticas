use std::env::args;
use std::io::stdin;

use aerolineas::{client::cli, server::sv};

fn main() {
    let argv = args().collect::<Vec<String>>();
    if argv.len() != 2 {
        println!("ERROR: No hay sólo 2 argumentos...");
        return;
    }

    match argv[1].as_str() {
        "sv" => {
            let _ = sv::run();
        }

        "cli" => {
            let _ = cli::run(&mut stdin());
        }
        _ => {
            println!("Se debe elegir o 'sv' o 'cli', no '{}'...", argv[1]);
        }
    }
}
