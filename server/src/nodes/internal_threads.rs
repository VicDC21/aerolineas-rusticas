//! Módulo que contiene las funciones que implementan los hilos internos de un nodo.

use {
    crate::{
        nodes::{
            actions::opcode::SvAction,
            addr::loader::AddrLoader,
            node::{Node, NodeHandle, NodeId},
            port_type::PortType,
            session_handler::{make_error_response, SessionHandler},
            utils::send_to_node,
        },
        utils::handle_pem_file_iter,
    },
    protocol::{
        aliases::{
            results::Result,
            types::{Byte, Ulong},
        },
        errors::error::Error,
        headers::opcode::Opcode,
        traits::Byteable,
    },
    rand::{distributions::WeightedIndex, prelude::Distribution, thread_rng},
    rustls::{
        pki_types::{pem::PemObject, CertificateDer, PrivateKeyDer},
        ServerConfig, ServerConnection, Stream,
    },
    std::{
        collections::HashSet,
        io::{BufRead, BufReader, Read, Write},
        net::{SocketAddr, TcpListener, TcpStream},
        sync::{
            mpsc::{channel, Receiver, Sender},
            Arc, Mutex,
        },
        thread::{sleep, spawn, Builder},
        time::Duration,
    },
    utils::get_root_path::get_root_path,
};

/// Un stream TLS.
type TlsStream<'a> = Stream<'a, ServerConnection, TcpStream>;

/// Cantidad de vecinos a los cuales un nodo tratará de acercarse en un ronda de _gossip_.
const HANDSHAKE_NEIGHBOURS: Byte = 3;
/// Cantidad de tiempo _(en milisegundos)_ que duerme el hilo de _heartbeat_.
const HEARTBEAT_SLEEP_MILLIS: Ulong = 1000;
/// Cantidad de tiempo _(en milisegundos)_ que duerme el hilo de _gossip_.
const GOSSIP_SLEEP_MILLIS: Ulong = 350;

/// El número de hilos para el [ThreadPool].
///
/// Crea los _handlers_ que escuchan por conexiones entrantes.
///
/// <div class="warning">
///
/// Esta función toma _ownership_ del [nodo](Node) que se le pasa.
///
/// </div>
pub fn create_client_and_private_conexion(
    node: Node,
    id: Byte,
    cli_socket: SocketAddr,
    priv_socket: SocketAddr,
    node_listeners: &mut Vec<Option<NodeHandle>>,
) -> Result<()> {
    let sendable_node = match SessionHandler::new(id, node) {
        Ok(handler) => handler,
        Err(err) => {
            return Err(Error::ServerError(format!(
                "Error creando un SessionHandler para el nodo: [{id}]:\n\n{err}"
            )));
        }
    };
    let cli_node = sendable_node.clone();
    let priv_node = sendable_node.clone();

    let cli_builder = Builder::new().name(format!("{id}_cli"));
    let cli_res = cli_builder.spawn(move || cli_listen(cli_socket, cli_node));
    match cli_res {
        Ok(cli_handler) => node_listeners.push(Some(cli_handler)),
        Err(err) => {
            return Err(Error::ServerError(format!(
                "Ocurrió un error tratando de crear el hilo listener de conexiones de cliente del nodo [{id}]:\n\n{err}"
            )));
        }
    }
    let priv_builder = Builder::new().name(format!("{id}_priv"));
    let priv_res = priv_builder.spawn(move || priv_listen(priv_socket, priv_node));
    match priv_res {
        Ok(priv_handler) => node_listeners.push(Some(priv_handler)),
        Err(err) => {
            return Err(Error::ServerError(format!(
                "Ocurrió un error tratando de crear el hilo listener de conexiones privadas del nodo [{id}]:\n\n{err}"
            )));
        }
    }
    Ok(())
}

/// Escucha por los eventos que recibe del cliente.
pub fn cli_listen(socket: SocketAddr, session_handler: SessionHandler) -> Result<()> {
    session_handler
        .logger
        .debug(&format!("Escuchando conexiones de cliente en {socket}"))
        .map_err(|e| Error::ServerError(e.to_string()))?;
    listen_cli_port(socket, session_handler)
}

/// Escucha por los eventos que recibe de otros nodos o estructuras internas.
pub fn priv_listen(socket: SocketAddr, session_handler: SessionHandler) -> Result<()> {
    session_handler
        .logger
        .debug(&format!("Escuchando conexiones privadas en {socket}"))
        .map_err(|e| Error::ServerError(e.to_string()))?;
    listen_priv_port(socket, session_handler)
}

fn listen_cli_port(socket: SocketAddr, session_handler: SessionHandler) -> Result<()> {
    let server_config = configure_tls()?;
    let listener = bind_with_socket(socket)?;
    let addr_loader = AddrLoader::default_loaded();

    let (shutdown_tx, shutdown_rx): (Sender<()>, Receiver<()>) = channel();
    let shutdown_rx = Arc::new(Mutex::new(shutdown_rx));

    let mut thread_handles = Vec::new();

    for tcp_stream_res in listener.incoming() {
        if let Ok(shutdown_rx) = shutdown_rx.try_lock() {
            if shutdown_rx.try_recv().is_ok() {
                break;
            }
        }

        match tcp_stream_res {
            Err(_) => return tcp_stream_error(&PortType::Cli, &socket, &addr_loader),
            Ok(tcp_stream) => {
                let config = Arc::clone(&server_config);
                let session_handler = session_handler.clone();
                let thread_shutdown_rx = Arc::clone(&shutdown_rx);
                let handle = spawn(move || loop {
                    if let Ok(rx) = thread_shutdown_rx.lock() {
                        if rx.try_recv().is_ok() {
                            break;
                        }
                    }

                    let tcp_stream = match tcp_stream.try_clone() {
                        Ok(stream) => stream,
                        Err(_) => break,
                    };

                    let result = listen_single_client(
                        config.clone(),
                        tcp_stream,
                        Arc::new(Mutex::new(false)),
                        session_handler.clone(),
                    );

                    if result.is_err() {
                        break;
                    }
                });
                thread_handles.push(handle);
            }
        };
    }

    drop(shutdown_tx);
    for handle in thread_handles {
        if let Err(err) = handle.join() {
            eprintln!("Error en el join de los threads: {err:?}");
        }
    }

    Ok(())
}

fn listen_priv_port(socket: SocketAddr, session_handler: SessionHandler) -> Result<()> {
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
                session_handler.process_stream(&mut tcp_stream, bytes_vec, true)?;
            }
        }
    }
    Ok(())
}

fn listen_single_client(
    config: Arc<ServerConfig>,
    tcp_stream: TcpStream,
    arc_exit: Arc<Mutex<bool>>,
    session_handler: SessionHandler,
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

    loop {
        let mut buffer: Vec<Byte> = vec![0; 2048];
        let size = match tls.read(&mut buffer) {
            Ok(value) => value,
            Err(_err) => {
                session_handler
                    .logger
                    .error("Error leyendo el stream TLS")
                    .map_err(|e| Error::ServerError(e.to_string()))?;
                return Err(Error::ServerError("No se pudo leer el stream".to_string()));
            }
        };

        buffer.truncate(size);
        session_handler
            .logger
            .info(&format!("Mensaje recibido (CLI): {buffer:?}"))
            .map_err(|e| Error::ServerError(e.to_string()))?;
        if is_exit(&buffer[..]) {
            session_handler
                .logger
                .info("Recibido mensaje EXIT desde cliente")
                .map_err(|e| Error::ServerError(e.to_string()))?;
            match arc_exit.lock() {
                Ok(mut locked_in) => *locked_in = true,
                Err(poison_err) => {
                    println!("Error de lock envenenado:\n\n{}", &poison_err);
                    arc_exit.clear_poison();
                }
            }
            break;
        }

        if !session_handler.node_is_responsive()? {
            let error = make_error_response(Error::ServerError(
                "Se esta cambiando la estructura de los nodos, vuelva luego.".to_string(),
            ));
            let _ = tls.write_all(&error);
        } else {
            let res = session_handler.process_stream(tls, buffer.to_vec(), is_logged)?;
            session_handler
                .logger
                .info(&format!("Respuesta enviada: {res:?}"))
                .map_err(|e| Error::ServerError(e.to_string()))?;
            if res.len() >= 9 && res[4] == Opcode::AuthSuccess.as_bytes()[0] {
                is_logged = true;
            }
        }
    }
    Ok(())
}

fn configure_tls() -> Result<Arc<ServerConfig>> {
    let private_key_file = get_root_path("custom.key");
    let certs: Vec<CertificateDer<'_>> = handle_pem_file_iter()?;
    let private_key = match PrivateKeyDer::from_pem_file(private_key_file) {
        Ok(value) => value,
        Err(_) => {
            return Err(Error::ServerError(
                "No se pudo leer la clave privada".to_string(),
            ))
        }
    };
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
            "No se pudo bindear a la dirección '{socket}'"
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
            "No se pudo clonar el stream:\n\n{err}"
        ))),
    }
}

fn write_bytes_in_buffer(bufreader: &mut BufReader<TcpStream>) -> Result<Vec<Byte>> {
    match bufreader.fill_buf() {
        Ok(recv) => Ok(recv.to_vec()),
        Err(err) => Err(Error::ServerError(format!(
            "No se pudo escribir los bytes:\n\n{err}"
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
pub fn beater(id: NodeId, receiver: Receiver<bool>) -> Result<NodeHandle> {
    let builder = Builder::new().name(format!("beater_node_{id}"));
    match builder.spawn(move || increase_heartbeat_and_store_metadata(receiver, id)) {
        Ok(handler) => Ok(handler),
        Err(_) => Err(Error::ServerError(format!(
            "Error procesando los beats del nodo {id}."
        ))),
    }
}

fn increase_heartbeat_and_store_metadata(
    receiver: Receiver<bool>,
    id: NodeId,
) -> std::result::Result<(), Error> {
    loop {
        sleep(Duration::from_millis(HEARTBEAT_SLEEP_MILLIS));
        if let Ok(stop) = receiver.try_recv() {
            if stop {
                println!("Frenando hilo beater");
                break;
            }
        }
        if send_to_node(id, SvAction::Beat.as_bytes(), PortType::Priv).is_err() {
            return Err(Error::ServerError(format!(
                "Error enviando mensaje de heartbeat a nodo {id}"
            )));
        }
        if send_to_node(id, SvAction::StoreMetadata.as_bytes(), PortType::Priv).is_err() {
            return Err(Error::ServerError(format!(
                "Error enviando mensaje de almacenamiento de metadata a nodo {id}"
            )));
        }
    }
    Ok(())
}

/// Periódicamente da inicio al proceso de _gossip_ de un nodo aleatorio.
pub fn gossiper(
    id: NodeId,
    nodes_weights: &[usize],
    receiver: Receiver<bool>,
) -> Result<NodeHandle> {
    let builder = Builder::new().name(format!("gossiper_node_{id}"));
    let weights = nodes_weights.to_vec();
    match builder.spawn(move || exec_gossip(receiver, id, weights)) {
        Ok(handler) => Ok(handler),
        Err(_) => Err(Error::ServerError(format!(
            "Error procesando la ronda de gossip del nodo {id}."
        ))),
    }
}

fn exec_gossip(receiver: Receiver<bool>, id: NodeId, weights: Vec<usize>) -> Result<()> {
    loop {
        sleep(Duration::from_millis(GOSSIP_SLEEP_MILLIS));
        if let Ok(stop) = receiver.try_recv() {
            if stop {
                println!("Frenando hilo gossiper");
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
                "Ocurrió un error enviando mensaje de gossip desde el nodo {id}:\n\n{err}"
            )));
        }
    }
    Ok(())
}
