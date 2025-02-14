use {
    aerolineas_rusticas::{
        client::cli::Client,
        protocol::aliases::{results::Result, types::Byte},
        server::nodes::{graph::NodesGraph, node::Node},
        simulator::cli::{run_sim, FlightConfigs},
    },
    std::{env::args, fs::File, io::BufReader, net::IpAddr, path::Path},
};

#[cfg(feature = "gui")]
use aerolineas_rusticas::interface::run::run_app;

fn main() {
    let argv = args().collect::<Vec<String>>();
    let how_to_use =
        "Uso:\n\ncargo run [cli | --features \"gui\" gui | sim | sv | nd [echo] | demo]\n";
    if argv.len() < 2 {
        println!("{}", how_to_use);
        return;
    }

    match argv[1].to_ascii_lowercase().as_str() {
        "sv" => {
            let mut graph = if argv.len() == 3 && argv[2].to_ascii_lowercase() == "echo" {
                NodesGraph::echo_mode()
            } else {
                NodesGraph::parsing_mode()
            };
            print_err(graph.init());
        }
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

/*
ENUNCIADO:

Reconfiguración dinámica del cluster

Se pide realizar todos los cambios necesarios para que el sistema distribuido de DB soporte la
incorporación y/o desvinculación de un nodo de la red. Es decir, dado un cluster de N nodos se
deberá poder iniciar e incorporar un nuevo nodo a la red, de manera que este mismo reciba un
rango de particionamiento para cada una de las tablas de la DB y los nodos existentes le envíen
la información correspondiente a los datos del segmento de partición asignado al nuevo nodo. El
nuevo nodo deberá entonces recibir la información de tablas, particiones y datos almacenados (de
su propio segmento de particiones unicamente)



NOTAS:

Se agrega un nodo que funciona correctamente con gossip.
Se agrega a traves de la consola mediante el comando "cargo run nd new <id> <ip> [echo]", habria que ver si hay que agregarle algo
de seguridad para poder usar este comando, ya que estas agregando un nodo nuevo al cluster (preguntar a martin).


Falta hacer que se recoloquen todas las tablas teniendo en cuenta el nuevo nodo agregado, habria que crear un estado que permita que los nodos
se reorganicen tranquilamente sin que los usuarios puedan hacer consultas.


PASO 3:
    Formato de mensaje entre nodos para reinsertar cada fila donde corresponda, que cada componente mande cuantos elementos tiene dentro.
    Ej: 2 Keyspace, adentro de la primer keyspace 2 tablas, adentro de la primer tabla, 2 filas

    Keyspace
        tabla
            fila
            fila
        tabla
            fila
            fila
    Keyspace
        tabla
            fila
            ...
        ...
    ...



Preguntas a martin:

Los nodos originales pueden ser dados de baja?
Habria que ver si hay que agregarle algo de seguridad para poder usar este comando,
ya que estas agregando un nodo nuevo al cluster (preguntar a martin).


La reasignacion de nodos deberia funcionar con algun nodo apagado?

*/
