//! Módulo para grafo de nodos.

use rand::{
    distributions::{Distribution, WeightedIndex},
    thread_rng,
};
use std::{
    collections::HashSet,
    net::SocketAddr,
    path::Path,
    sync::{
        mpsc::{channel, Sender},
        Arc, Mutex,
    },
    thread::{sleep, Builder, JoinHandle},
    time::Duration,
};

use crate::client::cql_frame::frame::Frame;
use crate::parser::{main_parser::make_parse, statements::statement::Statement};
use crate::protocol::{
    aliases::{results::Result, types::Byte},
    errors::error::Error,
    notations::consistency::Consistency,
    traits::Byteable,
};
use crate::server::{
    actions::opcode::SvAction,
    modes::ConnectionMode,
    nodes::{
        node::{Node, NodeId},
        port_type::PortType,
        utils::{load_init_queries, send_to_node},
    },
    utils::load_json,
};
use crate::tokenizer::tokenizer::tokenize_query;

use super::{
    disk_operations::disk_handler::{DiskHandler, NODES_METADATA_PATH},
    internal_threads::{cli_listen, priv_listen},
};

/// El handle donde vive una operación de nodo.
pub type NodeHandle = JoinHandle<Result<()>>;

/// Cantidad de nodos fija en cualquier momento.
pub const N_NODES: Byte = 5;
/// El ID con el que comenzar a contar los nodos.
pub const START_ID: NodeId = 10;
/// El último ID de los nodos, basado en la cantidad de nodos del clúster.
pub const LAST_ID: NodeId = START_ID + N_NODES;
/// Cantidad de vecinos a los cuales un nodo tratará de acercarse en un ronda de _gossip_.
const HANDSHAKE_NEIGHBOURS: Byte = 3;
/// La cantidad de nodos que comenzarán su intercambio de _gossip_ con otros [n](crate::server::nodes::graph::HANDSHAKE_NEIGHBOURS) nodos.
const SIMULTANEOUS_GOSSIPERS: Byte = 3;

/// Un grafo es una colección de nodos.
///
/// El mismo se encarga principalmente de gestionar los hilos en donde corren los nodos,
/// y mantener sus _handlers_ para luego finalizarlos, así como contar cuántos son para crear
/// nuevo, etc.
///
/// Sin embargo, no tiene ningún endpoint propio:
/// el cliente se comunica directo con los nodos.
pub struct NodesGraph {
    /// Todos los IDs de nodos bajo este grafo.
    node_ids: Vec<NodeId>,

    /// Los pesos de los nodos.
    node_weights: Vec<usize>,

    /// El próximo id disponible para un nodo.
    prox_id: NodeId,

    /// El modo con el que generar los siguientes nodos.
    preferred_mode: ConnectionMode,

    /// Todos los hilos bajo este grafo.
    ///
    /// NO incluye hilos especiales como el beater.
    handlers: Vec<Option<NodeHandle>>,
}

impl NodesGraph {
    /// Crea un nuevo grafo.
    pub fn new(node_ids: Vec<NodeId>, prox_id: NodeId, preferred_mode: ConnectionMode) -> Self {
        Self {
            node_ids,
            prox_id,
            preferred_mode,
            node_weights: Vec::new(),
            handlers: Vec::new(),
        }
    }

    /// Crea un nuevo grafo con el modo de conexión preferido.
    pub fn with_mode(preferred_mode: ConnectionMode) -> Self {
        Self::new(Vec::new(), START_ID, preferred_mode)
    }

    /// Crea una instancia del grafo en modo de DEBUG.
    pub fn echo_mode() -> Self {
        Self::with_mode(ConnectionMode::Echo)
    }

    /// Crea una instancia del grafo en modo para parsear _queries_.
    pub fn parsing_mode() -> Self {
        Self::with_mode(ConnectionMode::Parsing)
    }

    /// Inicializa el grafo y levanta todos los handlers necesarios.
    pub fn init(&mut self) -> Result<()> {
        let nodes = self.bootup_nodes(N_NODES)?;
        // REVISAR
        let (_beater, _beat_stopper) = self.beater()?;
        let (_gossiper, _gossip_stopper) = self.gossiper()?;

        self.handlers.extend(nodes);

        /*Paramos los handlers especiales primero
        let _ = beat_stopper.send(true);
        let _ = beater.join();

        let _ = gossip_stopper.send(true);
        let _ = gossiper.join();*/

        // Corremos los scripts iniciales
        if let Err(err) = self.send_init_queries() {
            println!("Error en las queries iniciales:\n{}", err);
        }

        self.wait();
        Ok(())
    }

    /// Manda todas las _queries_ iniciales.
    ///
    /// Dichas _queries_ normalmente vienen en forma de scripts, donde cada línea es una _query_.
    fn send_init_queries(&self) -> Result<()> {
        let node_id = self.node_ids[0]; // idealmente sería el primero que no esté caído
        let queries = load_init_queries();

        for (i, query) in queries.iter().enumerate() {
            let stream_id = format!("{}{}", node_id, i)
                .parse::<i16>()
                .unwrap_or(node_id as i16 + i as i16);
            match make_parse(&mut tokenize_query(query)) {
                Ok(statement) => {
                    let frame = match statement {
                        Statement::DmlStatement(_) | Statement::DdlStatement(_) => {
                            Frame::new(stream_id, query, Consistency::One)
                        } // Valor arbitrario por ahora
                        Statement::UdtStatement(_) => {
                            return Err(Error::ServerError("UDT statements no soportados".into()))
                        }
                        Statement::LoginUser(_) => {
                            return Err(Error::Invalid(
                                "No se deberia haber mandado el login por este canal".to_string(),
                            ))
                        }
                        Statement::Startup => {
                            return Err(Error::Invalid(
                                "No se deberia haber mandado el startup por este canal".to_string(),
                            ))
                        }
                    };
                    send_to_node(node_id, frame.as_bytes(), PortType::Priv)?;
                }
                Err(err) => {
                    println!(
                        "Ocurrió un error al crear el frame para una request inicial:\n\n{}",
                        err
                    );
                }
            }
        }

        Ok(())
    }

    /// Genera un vector de los IDs de los nodos.
    pub fn get_ids(&self) -> Vec<NodeId> {
        self.node_ids.clone()
    }

    /// Genera un vector de los pesos de los nodos.
    pub fn get_weights(&self) -> Vec<usize> {
        self.node_weights.clone()
    }

    /// "Inicia" los nodos del grafo en sus propios hilos.
    ///
    /// * `n` es la cantidad de nodos a crear en el proceso.
    fn bootup_nodes(&mut self, n: Byte) -> Result<Vec<Option<NodeHandle>>> {
        let nodes_folder = Path::new(&NODES_METADATA_PATH);
        if nodes_folder.exists() && nodes_folder.is_dir() {
            self.bootup_existing_nodes(n)
        } else {
            self.bootup_new_nodes(n)
        }
    }

    /// Inicializa nodos nuevos.
    fn bootup_new_nodes(&mut self, n: Byte) -> Result<Vec<Option<NodeHandle>>> {
        self.node_weights = vec![1; n as usize];
        self.node_weights[0] *= 3; // El primer nodo tiene el triple de probabilidades de ser elegido.

        let mut handlers: Vec<Option<NodeHandle>> = Vec::new();
        for i in 0..n {
            let mut node_listeners: Vec<Option<NodeHandle>> = Vec::new();
            let current_id = self.add_node_id();
            let node = Node::new(current_id, self.preferred_mode.clone())?;

            let cli_socket = node.get_endpoint_state().socket(&PortType::Cli);
            let priv_socket = node.get_endpoint_state().socket(&PortType::Priv);

            create_client_and_private_conexion(
                current_id,
                cli_socket,
                &mut node_listeners,
                i,
                priv_socket,
                node,
            )?;

            handlers.append(&mut node_listeners);
        }
        // Llenamos de información al nodo "seed".
        self.send_states_to_node(self.max_weight());
        Ok(handlers)
    }

    /// Inicializa nodos existentes.
    fn bootup_existing_nodes(&mut self, n: Byte) -> Result<Vec<Option<NodeHandle>>> {
        let mut handlers: Vec<Option<NodeHandle>> = Vec::new();

        for i in 0..n {
            let current_id = self.add_node_id();
            let node_path = DiskHandler::get_node_metadata_path(current_id);
            let path = Path::new(&node_path);
            if path.exists() {
                let mut node: Node = load_json(&node_path)?;
                node.set_default_fields(current_id, self.preferred_mode.clone())?;

                if current_id == START_ID {
                    // El primer nodo tiene el triple de probabilidades de ser elegido.
                    self.node_weights.push(3);
                } else {
                    self.node_weights.push(1);
                }

                let mut node_listeners: Vec<Option<NodeHandle>> = Vec::new();

                let cli_socket = node.get_endpoint_state().socket(&PortType::Cli);
                let priv_socket = node.get_endpoint_state().socket(&PortType::Priv);
                create_client_and_private_conexion(
                    current_id,
                    cli_socket,
                    &mut node_listeners,
                    i as Byte,
                    priv_socket,
                    node,
                )?;

                handlers.append(&mut node_listeners);
            } else {
                return Err(Error::ServerError(format!(
                    "No se encontró el archivo de metadata del nodo {}",
                    current_id
                )));
            }
        }
        // Llenamos de información al nodo "seed".
        self.send_states_to_node(self.max_weight());
        Ok(handlers)
    }

    /// Realiza una ronda de _gossip_.
    fn gossiper(&self) -> Result<(NodeHandle, Sender<bool>)> {
        let (sender, receiver) = channel::<bool>();
        let builder = Builder::new().name("gossip".to_string());
        let weights = self.get_weights();
        match builder.spawn(move || exec_gossip(receiver, weights)) {
            Ok(handler) => Ok((handler, sender.clone())),
            Err(_) => Err(Error::ServerError(
                "Error procesando la ronda de gossip de los nodos.".to_string(),
            )),
        }
    }

    /// Agrega un nodo al grafo.
    ///
    /// También devuelve el ID del nodo recién agregado.
    pub fn add_node_id(&mut self) -> NodeId {
        self.node_ids.push(self.prox_id);
        self.prox_id += 1;
        self.prox_id - 1
    }

    /// Decide cuál es el nodo con el mayor "peso". Es decir, el que tiene más probabilidades
    /// de ser elegido cuando se los elige "al azar".
    ///
    /// Si todos son iguales, agarra el primero.
    pub fn max_weight(&self) -> NodeId {
        let mut max_id: usize = 0;
        for i in 0..self.node_ids.len() {
            if self.node_weights[i] > self.node_weights[max_id] {
                max_id = i;
            }
        }
        self.node_ids[max_id]
    }

    /// Ordena a todos los nodos existentes que envien su endpoint state al nodo con el ID correspondiente.
    fn send_states_to_node(&self, id: NodeId) {
        for node_id in self.get_ids() {
            if let Err(err) = send_to_node(
                node_id,
                SvAction::SendEndpointState(id).as_bytes(),
                PortType::Priv,
            ) {
                println!(
                    "Ocurrió un error presentando vecinos de un nodo:\n\n{}",
                    err
                );
            }
        }
    }

    /// Avanza a cada segundo el estado de _heartbeat_ de los nodos.
    fn beater(&self) -> Result<(NodeHandle, Sender<bool>)> {
        let (sender, receiver) = channel::<bool>();
        let builder = Builder::new().name("beater".to_string());
        let ids = self.get_ids();
        match builder.spawn(move || increase_heartbeat_and_store_nodes(receiver, ids)) {
            Ok(handler) => Ok((handler, sender.clone())),
            Err(_) => Err(Error::ServerError(
                "Error procesando los beats de los nodos.".to_string(),
            )),
        }
    }

    /// Espera a que terminen todos los handlers.
    ///
    /// Esto idealmente sólo debería llamarse una vez, ya que consume los handlers y además
    /// bloquea el hilo actual.
    pub fn wait(&mut self) {
        // long live the option dance
        for handler_opt in &mut self.handlers {
            if let Some(handler) = handler_opt.take() {
                if handler.join().is_err() {
                    // Un hilo caído NO debería interrumpir el dropping de los demás
                    println!("Ocurrió un error mientras se esperaba a que termine un hilo hijo.");
                }
            }
        }
    }
}

/// Crea los _handlers_ que escuchan por conexiones entrantes.
///
/// <div class="warning">
///
/// Esta función toma _ownership_ del [nodo](Node) que se le pasa.
///
/// </div>
fn create_client_and_private_conexion(
    current_id: u8,
    cli_socket: SocketAddr,
    node_listeners: &mut Vec<Option<NodeHandle>>,
    i: u8,
    priv_socket: SocketAddr,
    node: Node,
) -> Result<()> {
    let sendable_node = Arc::new(Mutex::new(node));
    let cli_node = Arc::clone(&sendable_node);
    let priv_node = Arc::clone(&sendable_node);

    let cli_builder = Builder::new().name(format!("{}_cli", current_id));
    let cli_res = cli_builder.spawn(move || cli_listen(cli_socket, cli_node));
    match cli_res {
        Ok(cli_handler) => node_listeners.push(Some(cli_handler)),
        Err(err) => {
            return Err(Error::ServerError(format!(
                "Ocurrió un error tratando de crear el hilo listener de conexiones de cliente del nodo [{}]:\n\n{}",
                i, err
            )));
        }
    }
    let priv_builder = Builder::new().name(format!("{}_priv", current_id));
    let priv_res = priv_builder.spawn(move || priv_listen(priv_socket, priv_node));
    match priv_res {
        Ok(priv_handler) => node_listeners.push(Some(priv_handler)),
        Err(err) => {
            return Err(Error::ServerError(format!(
                "Ocurrió un error tratando de crear el hilo listener de conexiones privadas del nodo [{}]:\n\n{}",
                i, err
            )));
        }
    }
    Ok(())
}

fn increase_heartbeat_and_store_nodes(
    receiver: std::sync::mpsc::Receiver<bool>,
    ids: Vec<Byte>,
) -> std::result::Result<(), Error> {
    loop {
        sleep(Duration::from_secs(1));
        if let Ok(stop) = receiver.try_recv() {
            if stop {
                break;
            }
        }
        for node_id in &ids {
            if send_to_node(*node_id, SvAction::Beat.as_bytes(), PortType::Priv).is_err() {
                return Err(Error::ServerError(format!(
                    "Error enviando mensaje de heartbeat a nodo {}",
                    node_id
                )));
            }
            if send_to_node(*node_id, SvAction::StoreMetadata.as_bytes(), PortType::Priv).is_err() {
                return Err(Error::ServerError(format!(
                    "Error enviando mensaje de almacenamiento de metadata a nodo {}",
                    node_id
                )));
            }
        }
    }
    Ok(())
}

fn exec_gossip(
    receiver: std::sync::mpsc::Receiver<bool>,
    weights: Vec<usize>,
) -> std::result::Result<(), Error> {
    loop {
        sleep(Duration::from_millis(200));
        if let Ok(stop) = receiver.try_recv() {
            if stop {
                break;
            }
        }

        let dist = if let Ok(dist) = WeightedIndex::new(&weights) {
            dist
        } else {
            return Err(Error::ServerError(format!(
                "No se pudo crear una distribución de pesos con {:?}.",
                &weights
            )));
        };

        let mut rng = thread_rng();
        let mut selected_ids: HashSet<NodeId> = HashSet::new();
        while selected_ids.len() < SIMULTANEOUS_GOSSIPERS as usize {
            let selected_id = dist.sample(&mut rng) as NodeId;
            if !selected_ids.contains(&(selected_id + START_ID)) {
                // No contener repetidos
                selected_ids.insert(selected_id + START_ID);
            }
        }

        for selected_id in selected_ids {
            let mut neighbours: HashSet<NodeId> = HashSet::new();
            while neighbours.len() < HANDSHAKE_NEIGHBOURS as usize {
                let selected_neighbour = dist.sample(&mut rng) as NodeId;
                if ((selected_neighbour + START_ID) != selected_id)
                    && (!neighbours.contains(&(selected_neighbour + START_ID)))
                {
                    neighbours.insert(selected_neighbour + START_ID);
                }
            }

            if let Err(err) = send_to_node(
                selected_id as NodeId,
                SvAction::Gossip(neighbours).as_bytes(),
                PortType::Priv,
            ) {
                println!("Ocurrió un error enviando mensaje de gossip:\n\n{}", err);
            }
        }
    }
    Ok(())
}

impl Default for NodesGraph {
    fn default() -> Self {
        Self::new(Vec::new(), START_ID, ConnectionMode::Parsing)
    }
}
