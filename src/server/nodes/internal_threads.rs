//! Módulo que contiene las funciones que implementan los hilos internos de un nodo.

use rand::{distributions::WeightedIndex, prelude::Distribution, thread_rng};
use std::{
    collections::HashSet,
    io::{BufRead, BufReader},
    net::{SocketAddr, TcpListener, TcpStream},
    sync::{
        mpsc::{channel, Sender},
        Arc, Mutex,
    },
    thread::{sleep, Builder},
    time::Duration,
};

use crate::protocol::{
    aliases::{results::Result, types::Byte},
    errors::error::Error,
    traits::Byteable,
};
use crate::server::{
    actions::opcode::SvAction,
    nodes::{
        addr::loader::AddrLoader,
        node::{Node, NodeHandle, NodeId},
        port_type::PortType,
        utils::send_to_node,
    },
};

/// Cantidad de vecinos a los cuales un nodo tratará de acercarse en un ronda de _gossip_.
const HANDSHAKE_NEIGHBOURS: Byte = 3;

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
    let listener = bind_with_socket(socket)?;
    let addr_loader = AddrLoader::default_loaded();
    for tcp_stream_res in listener.incoming() {
        match tcp_stream_res {
            Err(_) => return tcp_stream_error(&port_type, &socket, &addr_loader),
            Ok(tcp_stream) => {
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
                        locked_in.process_tcp(tcp_stream, bytes_vec)?;
                    }
                    Err(poison_err) => {
                        println!("Error de lock envenenado:\n\n{}", poison_err);
                    }
                }
            }
        }
    }

    Ok(())
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
        sleep(Duration::from_millis(200));
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
