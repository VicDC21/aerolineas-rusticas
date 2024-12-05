//! Módulo que contiene las funciones que implementan los hilos internos de un nodo.

use rand::{distributions::WeightedIndex, prelude::Distribution, thread_rng};
use rustls::{
    pki_types::{pem::PemObject, CertificateDer, PrivateKeyDer},
    ServerConfig, ServerConnection, Stream,
};
use std::{
    collections::HashSet,
    io::{BufRead, BufReader, Read},
    net::{SocketAddr, TcpListener, TcpStream},
    sync::{
        mpsc::{channel, Sender},
        Arc, Mutex, MutexGuard, RwLock,
    },
    thread::{self, sleep, Builder},
    time::Duration,
};

use crate::{
    client::cli::handle_pem_file_iter,
    server::{
        actions::opcode::SvAction,
        nodes::{
            addr::loader::AddrLoader,
            node::{Node, NodeHandle, NodeId},
            port_type::PortType,
            utils::send_to_node,
        },
    },
};
use crate::protocol::{
        aliases::{results::Result, types::Byte},
        errors::error::Error,
        headers::opcode::Opcode,
        traits::Byteable,
    }
;

/// Un stream TLS.
type TlsStream<'a> = Stream<'a, ServerConnection, TcpStream>;

/// Cantidad de vecinos a los cuales un nodo tratará de acercarse en un ronda de _gossip_.
const HANDSHAKE_NEIGHBOURS: Byte = 3;

/// El número de hilos para el [ThreadPool].

/// Crea los _handlers_ que escuchan por conexiones entrantes.
///
/// <div class="warning">
///
/// Esta función toma _ownership_ del [nodo](Node) que se le pasa.
///
/// </div>
pub fn create_client_and_private_conexion(
    node: Node,
    id: u8,
    cli_socket: SocketAddr,
    priv_socket: SocketAddr,
    node_listeners: &mut Vec<Option<NodeHandle>>,
) -> Result<()> {
    let sendable_node = Arc::new(Mutex::new(node));
    // let sendable_node = RwLock::new(Arc::new(node));
    // creamos de esta manera el RwLock
    let cli_node = Arc::clone(&sendable_node);
    let priv_node = Arc::clone(&sendable_node);


    let cli_builder = Builder::new().name(format!("{}_cli", id));
    let cli_res = cli_builder.spawn(move || cli_listen(cli_socket, cli_node));
    match cli_res {
        Ok(cli_handler) => node_listeners.push(Some(cli_handler)),
        Err(err) => {
            return Err(Error::ServerError(format!(
                "Ocurrió un error tratando de crear el hilo listener de conexiones de cliente del nodo [{}]:\n\n{}",
                id, err
            )));
        }
    }
    let priv_builder = Builder::new().name(format!("{}_priv", id));
    let priv_res = priv_builder.spawn(move || priv_listen(priv_socket, priv_node));
    match priv_res {
        Ok(priv_handler) => node_listeners.push(Some(priv_handler)),
        Err(err) => {
            return Err(Error::ServerError(format!(
                "Ocurrió un error tratando de crear el hilo listener de conexiones privadas del nodo [{}]:\n\n{}",
                id, err
            )));
        }
    }
    Ok(())
}

/// Escucha por los eventos que recibe del cliente.
pub fn cli_listen(socket: SocketAddr, node: Arc<Mutex<Node>>) -> Result<()> {
    listen(socket, PortType::Cli, node)
}

/// Escucha por los eventos que recibe de otros nodos o estructuras internas.
pub fn priv_listen(socket: SocketAddr, node: Arc<Mutex<Node>>) -> Result<()> {
    listen(socket, PortType::Priv, node)
}

/// El escuchador de verdad.
///
/// Las otras funciones son wrappers para no repetir código.
fn listen(socket: SocketAddr, port_type: PortType, node: Arc<Mutex<Node>>) -> Result<()> {
    match port_type {
        PortType::Cli => listen_cli_port(socket, node),
        PortType::Priv => listen_priv_port(socket, node),
    }
}

fn listen_cli_port(socket: SocketAddr, node: Arc<Mutex<Node>>) -> Result<()> {
    let server_config = configure_tls()?;
    let listener = bind_with_socket(socket)?;
    let addr_loader = AddrLoader::default_loaded();
    let exit = false;
    for tcp_stream_res in listener.incoming() {
        match tcp_stream_res {
            Err(_) => return tcp_stream_error(&PortType::Cli, &socket, &addr_loader),
            Ok(tcp_stream) => {
                let config = Arc::clone(&server_config);
                let node = Arc::clone(&node);
                let arc_exit = Arc::new(Mutex::new(exit));
                println!("Se conectan a este nodo");
                thread::spawn(move || listen_single_client(config, tcp_stream, arc_exit, node));
            }
        };
        if exit {
            break;
        }
    }
    Ok(())
}





fn listen_priv_port(socket: SocketAddr, node: Arc<Mutex<Node>>) -> Result<()> {
    let listener = bind_with_socket(socket)?;
    let addr_loader = AddrLoader::default_loaded();
    for tcp_stream_res in listener.incoming() {
        match tcp_stream_res {
            Err(_) => return tcp_stream_error(&PortType::Priv, &socket, &addr_loader),
            Ok(mut tcp_stream) => {
                let buffered_stream = clone_tcp_stream(&tcp_stream)?;
                let mut bufreader = BufReader::new(buffered_stream);
                let bytes_vec = write_bytes_in_buffer(&mut bufreader)?;
                // consumimos los bytes del stream para no mandarlos de vuelta en la response
                bufreader.consume(bytes_vec.len());
                if is_exit(&bytes_vec[..]) {
                    break;
                }
                match node.lock() {
                    Ok(mut locked_in) => {
                        locked_in.process_stream(&mut tcp_stream, bytes_vec, true)?;
                    }
                    Err(poison_err) => {
                        println!("Error de lock envenenado:\n\n{}", poison_err);
                        node.clear_poison();
                    }
                }
            }
        }
    }
    Ok(())
}

fn listen_single_client(
    config: Arc<ServerConfig>,
    tcp_stream: TcpStream,
    arc_exit: Arc<Mutex<bool>>,
    node: Arc<Mutex<Node>>,
) -> Result<()> {
    let mut server_conn = match ServerConnection::new(config) {
        Ok(conn) => conn,
        Err(_) => {
            return Err(Error::ServerError(
                "Error al crear la conexión TLS".to_string(),
            ));
        }
    };
    let mut buffered_stream = clone_tcp_stream(&tcp_stream)?;
    let mut tls_stream: TlsStream = Stream::new(&mut server_conn, &mut buffered_stream);
    let tls = &mut tls_stream;
    let mut is_logged = false;

    // aca crear una nueva estructura que se encargue de handelear las queries y que tenga el Arc del nodo
    loop {
        let mut buffer: Vec<u8> = vec![0; 2048];
        let size = match tls.read(&mut buffer) {
            Ok(value) => value,
            Err(_err) => return Err(Error::ServerError("No se pudo leer el stream".to_string())),
        };
        buffer.truncate(size);
        if is_exit(&buffer[..]) {
            match arc_exit.lock() {
                Ok(mut locked_in) => *locked_in = true,
                Err(poison_err) => {
                    println!("Error de lock envenenado:\n\n{}", poison_err);
                    node.clear_poison();
                }
            }
            break;
        }
        // aca sacar el lock del node, no vamos a lockear tan temprano
        match node.lock() {
            Ok(mut locked_in) => {
                let res = locked_in.process_stream(tls, buffer.to_vec(), is_logged)?;
                if res.len() >= 9 && res[4] == Opcode::AuthSuccess.as_bytes()[0] {
                    is_logged = true;
                }
            }
            Err(poison_err) => {
                println!("Error de lock envenenado:\n\n{}", poison_err);
                node.clear_poison();
            }
        }
    }
    Ok(())
}

// fn get_node_mutex(node: &mut Arc<Mutex<Node>>) -> Result<MutexGuard<'static, Node>>{
//     match node.lock() {
//         Ok(mut locked_in) => {
//             Ok(locked_in)
//         }
//         Err(poison_err) => {
//             println!("Error de lock envenenado:\n\n{}", poison_err);
//             node.clear_poison();
//             Err(Error::ServerError(format!("Error de lock envenenado:\n\n{}", poison_err)))
//         }
//     }
// }



fn configure_tls() -> Result<Arc<ServerConfig>> {
    let private_key_file = "custom.key";
    let certs: Vec<CertificateDer<'_>> = handle_pem_file_iter()?;
    let private_key = PrivateKeyDer::from_pem_file(private_key_file).unwrap();
    let config = match ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, private_key)
    {
        Ok(value) => value,
        Err(_err) => {
            return Err(Error::ServerError(
                "No se pudo buildear la configuracion tls".to_string(),
            ))
        }
    };
    Ok(Arc::new(config))
}

fn bind_with_socket(socket: SocketAddr) -> Result<TcpListener> {
    match TcpListener::bind(socket) {
        Ok(tcp_listener) => Ok(tcp_listener),
        Err(_) => Err(Error::ServerError(format!(
            "No se pudo bindear a la dirección '{}'",
            socket
        ))),
    }
}

fn tcp_stream_error(
    port_type: &PortType,
    socket: &SocketAddr,
    addr_loader: &AddrLoader,
) -> Result<()> {
    let falla = match port_type {
        PortType::Cli => "cliente",
        PortType::Priv => "nodo o estructura interna",
    };
    Err(Error::ServerError(format!(
        "Un {} no pudo conectarse al nodo con ID {}",
        falla,
        addr_loader.get_id(&socket.ip())?,
    )))
}

fn clone_tcp_stream(tcp_stream: &TcpStream) -> Result<TcpStream> {
    match tcp_stream.try_clone() {
        Ok(cloned) => Ok(cloned),
        Err(err) => Err(Error::ServerError(format!(
            "No se pudo clonar el stream:\n\n{}",
            err
        ))),
    }
}

fn write_bytes_in_buffer(bufreader: &mut BufReader<TcpStream>) -> Result<Vec<Byte>> {
    match bufreader.fill_buf() {
        Ok(recv) => Ok(recv.to_vec()),
        Err(err) => Err(Error::ServerError(format!(
            "No se pudo escribir los bytes:\n\n{}",
            err
        ))),
    }
}

/// Verifica rápidamente si un mensaje es de tipo [EXIT](SvAction::Exit).
fn is_exit(bytes: &[Byte]) -> bool {
    if let Some(action) = SvAction::get_action(bytes) {
        if matches!(action, SvAction::Exit) {
            return true;
        }
    }
    false
}

/// Avanza a cada segundo el estado de _heartbeat_ de un nodo.
///
/// Además, almacena la metadata del nodo en la carpeta `nodes_metadata`.
pub fn beater(id: NodeId) -> Result<(NodeHandle, Sender<bool>)> {
    let (sender, receiver) = channel::<bool>();
    let builder = Builder::new().name(format!("beater_node_{}", id));
    match builder.spawn(move || increase_heartbeat_and_store_metadata(receiver, id)) {
        Ok(handler) => Ok((handler, sender.clone())),
        Err(_) => Err(Error::ServerError(format!(
            "Error procesando los beats del nodo {}.",
            id
        ))),
    }
}

fn increase_heartbeat_and_store_metadata(
    receiver: std::sync::mpsc::Receiver<bool>,
    id: NodeId,
) -> std::result::Result<(), Error> {
    loop {
        sleep(Duration::from_secs(1));
        if let Ok(stop) = receiver.try_recv() {
            if stop {
                break;
            }
        }
        if send_to_node(id, SvAction::Beat.as_bytes(), PortType::Priv).is_err() {
            return Err(Error::ServerError(format!(
                "Error enviando mensaje de heartbeat a nodo {}",
                id
            )));
        }
        if send_to_node(id, SvAction::StoreMetadata.as_bytes(), PortType::Priv).is_err() {
            return Err(Error::ServerError(format!(
                "Error enviando mensaje de almacenamiento de metadata a nodo {}",
                id
            )));
        }
    }
    Ok(())
}

/// Periódicamente da inicio al proceso de _gossip_ de un nodo aleatorio.
pub fn gossiper(id: NodeId, nodes_weights: &[usize]) -> Result<(NodeHandle, Sender<bool>)> {
    let (sender, receiver) = channel::<bool>();
    let builder = Builder::new().name(format!("gossiper_node_{}", id));
    let weights = nodes_weights.to_vec();
    match builder.spawn(move || exec_gossip(receiver, id, weights)) {
        Ok(handler) => Ok((handler, sender.clone())),
        Err(_) => Err(Error::ServerError(format!(
            "Error procesando la ronda de gossip del nodo {}.",
            id
        ))),
    }
}

fn exec_gossip(
    receiver: std::sync::mpsc::Receiver<bool>,
    id: NodeId,
    weights: Vec<usize>,
) -> Result<()> {
    loop {
        sleep(Duration::from_millis(1500));
        if let Ok(stop) = receiver.try_recv() {
            if stop {
                break;
            }
        }

        let dist = match WeightedIndex::new(&weights) {
            Ok(dist) => dist,
            Err(_) => {
                return Err(Error::ServerError(format!(
                    "No se pudo crear una distribución de pesos con {:?}.",
                    &weights
                )));
            }
        };

        let nodes_ids = AddrLoader::default_loaded().get_ids();
        let mut rng = thread_rng();
        let selected_id = nodes_ids[dist.sample(&mut rng)];
        if selected_id != id {
            continue;
        }

        let mut neighbours: HashSet<NodeId> = HashSet::new();
        while neighbours.len() < HANDSHAKE_NEIGHBOURS as usize {
            let selected_neighbour = nodes_ids[dist.sample(&mut rng)];
            if (selected_neighbour != selected_id) && !neighbours.contains(&selected_neighbour) {
                neighbours.insert(selected_neighbour);
            }
        }

        if let Err(err) = send_to_node(
            selected_id,
            SvAction::Gossip(neighbours).as_bytes(),
            PortType::Priv,
        ) {
            return Err(Error::ServerError(format!(
                "Ocurrió un error enviando mensaje de gossip desde el nodo {}:\n\n{}",
                id, err
            )));
        }
    }
    Ok(())
}
