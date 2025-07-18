use {
    aerolineas_rusticas::{
        client::cli::Client,
        protocol::aliases::{results::Result, types::Byte},
        server::nodes::node::Node,
        simulator::cli::{run_sim, FlightConfigs},
    },
    std::{env::args, fs::File, io::BufReader, net::IpAddr, path::Path},
};

#[cfg(feature = "gui")]
use aerolineas_rusticas::interface::run::run_app;

fn main() {
    let argv = args().collect::<Vec<String>>();
    let how_to_use =
        "Uso:\n\ncargo run [cli | --features \"gui\" gui | sim | nd [echo] | demo]\n";
    if argv.len() < 2 {
        println!("{}", how_to_use);
        return;
    }

    match argv[1].to_ascii_lowercase().as_str() {
        "nd" => {
            run_nd(argv);
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
            print_err(run_sim(&[]));
        }
        "demo" => {
            run_demo();
        }
        _ => {
            println!("{}", how_to_use);
        }
    }
}

fn run_nd(argv: Vec<String>) {
    if argv.len() >= 3 {
        if argv[2] == "new" && argv.len() >= 4 {
            // cargo run nd new <id> <ip> [echo]
            match argv[3].parse::<Byte>() {
                Ok(id) => {
                    if argv[4].parse::<IpAddr>().is_ok() {
                        println!("Nodo nuevo con id {} y dirección IP {}.", id, argv[4]);
                        if argv.len() == 5 && argv[4].to_ascii_lowercase() == "echo" {
                            print_err(Node::init_new_in_echo_mode(id, &argv[4]))
                        } else {
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
        } else if argv[2] == "delete" && argv.len() >= 3 {
            // cargo run nd delete <id>
            match argv[3].parse::<Byte>() {
                Ok(id) => {
                    println!("Nodo a eliminar: {}", argv[3]);
                    print_err(Node::delete_node(id));
                }
                Err(_) => {
                    println!("El id debe ser un número entero entre 0 y 255.");
                }
            }
        } else {
            // cargo run nd <id> [echo]
            match argv[2].parse::<Byte>() {
                Ok(id) => {
                    if argv.len() == 4 && argv[3].to_ascii_lowercase() == "echo" {
                        print_err(Node::init_in_echo_mode(id))
                    } else {
                        print_err(Node::init_in_parsing_mode(id))
                    }
                }
                Err(_) => {
                    println!("El id debe ser un número entero entre 0 y 255.");
                }
            }
        }
    } else {
        println!("Uso:\n\ncargo run nd [new] <id> [<ip>] [echo]\n");
    };
}

fn run_demo() {
    let file = match File::open(Path::new("media/flights/flights_configs.json")) {
        Ok(file) => file,
        Err(err) => {
            println!(
                "Error al abrir el archivo de configuración de vuelos: {}",
                err
            );
            return;
        }
    };
    let flight_configs: FlightConfigs = match serde_json::from_reader(BufReader::new(file)) {
        Ok(configs) => configs,
        Err(err) => {
            println!(
                "Error al leer el archivo de configuración de vuelos: {}",
                err
            );
            return;
        }
    };
    print_err(run_sim(&flight_configs.flight_configs));
}

/// Imprime por pantalla el error
fn print_err(res: Result<()>) {
    if let Err(err) = res {
        println!("{}", err);
    }
}
