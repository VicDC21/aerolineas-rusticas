//! Módulo para grafo de nodos.

use rand::{
    distributions::{Distribution, WeightedIndex},
    thread_rng,
};
use std::{
    collections::HashSet,
    hash::{DefaultHasher, Hash, Hasher},
    net::TcpStream,
    sync::mpsc::{channel, Sender},
    thread::{sleep, Builder, JoinHandle},
    time::Duration,
};

use crate::protocol::{
    aliases::{results::Result, types::Byte},
    errors::error::Error,
    traits::Byteable,
};
use crate::server::{
    actions::opcode::SvAction,
    modes::ConnectionMode,
    nodes::{
        node::{Node, NodeId},
        port_type::PortType,
        utils::send_to_node,
    },
};

/// El handle donde vive una operación de nodo.
pub type NodeHandle = JoinHandle<Result<()>>;

/// Cantidad de nodos fija en cualquier momento.
pub const N_NODES: u8 = 5;
/// El ID con el que comenzar a contar los nodos.
pub const START_ID: NodeId = 10;
/// Cantidad de vecinos a los cuales un nodo tratará de acercarse en un ronda de _gossip_.
const HANDSHAKE_NEIGHBOURS: u8 = 3;
/// La cantidad de nodos que comenzarán su intercambio de _gossip_ con otros [n](crate::server::nodes::graph::HANDSHAKE_NEIGHBOURS) nodos.
const SIMULTANEOUS_GOSSIPERS: u8 = 3;

/// Un grafo es una colección de nodos.
///
/// El mismo se encarga principalmente de gestionar los hilos en donde corren los nodos,
/// y manteners sus _handlers_ para luego finalizarlos, así como contar cuántos son para crear
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
        let (gossiper, gossip_stopper) = self.gossiper()?;
        let (beater, beat_stopper) = self.beater()?;
        let nodes = self.bootup_nodes(N_NODES)?;

        self.handlers.extend(nodes);

        // Paramos los handlers especiales primero
        let _ = gossip_stopper.send(true);
        let _ = gossiper.join();

        let _ = beat_stopper.send(true);
        let _ = beater.join();

        self.wait();
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
    fn bootup_nodes(&mut self, n: u8) -> Result<Vec<Option<NodeHandle>>> {
        self.node_weights = vec![1; n as usize];
        self.node_weights[0] *= 3; // El primer nodo tiene el triple de probabilidades de ser elegido.

        let mut handlers: Vec<Option<NodeHandle>> = Vec::new();
        for i in 0..n {
            let mut node_listeners: Vec<NodeHandle> = Vec::new();
            let current_id = self.add_node_id();
            let node = Node::new(current_id, self.preferred_mode.clone());

            let cli_socket = node.get_endpoint_state().socket(&PortType::Cli);
            let priv_socket = node.get_endpoint_state().socket(&PortType::Priv);

            let (proc_sender, proc_receiver) = channel::<TcpStream>();

            // Sino aparentemente el hilo toma ownership antes de poder clonarlo.
            let cli_sender = proc_sender.clone();
            let priv_sender = proc_sender.clone();

            let cli_builder = Builder::new().name(format!("{}_cli", current_id));
            let cli_res = cli_builder.spawn(move || Node::cli_listen(cli_socket, cli_sender));
            match cli_res {
                Ok(cli_handler) => node_listeners.push(cli_handler),
                Err(err) => {
                    return Err(Error::ServerError(format!(
                        "Ocurrió un error tratando de crear el hilo listener de conexiones de cliente del nodo [{}]:\n\n{}",
                        i, err
                    )));
                }
            }

            let priv_builder = Builder::new().name(format!("{}_priv", current_id));
            let priv_res = priv_builder.spawn(move || Node::priv_listen(priv_socket, priv_sender));
            match priv_res {
                Ok(priv_handler) => node_listeners.push(priv_handler),
                Err(err) => {
                    return Err(Error::ServerError(format!(
                        "Ocurrió un error tratando de crear el hilo listener de conexiones privadas del nodo [{}]:\n\n{}",
                        i, err
                    )));
                }
            }
            let processor = node.request_processor(proc_receiver, node_listeners)?;

            // Los join de los listeners están dentro del procesador
            handlers.push(Some(processor));
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
        match builder.spawn(move || {
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
                    if !selected_ids.contains(&selected_id) {
                        // No contener repetidos
                        selected_ids.insert(selected_id);
                    }
                }

                for selected_id in selected_ids {
                    let mut neighbours: HashSet<NodeId> = HashSet::new();
                    while neighbours.len() < HANDSHAKE_NEIGHBOURS as usize {
                        let selected_neighbour = dist.sample(&mut rng) as NodeId;
                        if (selected_neighbour != selected_id)
                            && (!neighbours.contains(&selected_neighbour))
                        {
                            neighbours.insert(selected_neighbour);
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
        }) {
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
    fn beater(&mut self) -> Result<(NodeHandle, Sender<bool>)> {
        let (sender, receiver) = channel::<bool>();
        let builder = Builder::new().name("beater".to_string());
        let ids = self.get_ids();
        match builder.spawn(move || {
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
                            "Error enviado mensaje de heartbeat a nodo {}",
                            node_id
                        )));
                    }
                }
            }
            Ok(())
        }) {
            Ok(handler) => Ok((handler, sender.clone())),
            Err(_) => Err(Error::ServerError(
                "Error procesando los beats de los nodos.".to_string(),
            )),
        }
    }

    /// Selecciona un ID de nodo conforme al _hashing_ de un conjunto de [Byte]s.
    pub fn select_node(&self, bytes: &Vec<Byte>) -> NodeId {
        let mut hasher = DefaultHasher::new();
        bytes.hash(&mut hasher);
        let hash_val = hasher.finish();

        let n = self.node_ids.len() as u64;
        let magic_ind = (hash_val % n) as usize;
        self.node_ids[magic_ind]
    }

    /// Manda un mensaje al nodo relevante mediante el _hashing_ del mensaje.
    pub fn send_message(&self, bytes: Vec<Byte>, port_type: PortType) -> Result<()> {
        send_to_node(self.select_node(&bytes), bytes, port_type)
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

impl Default for NodesGraph {
    fn default() -> Self {
        Self::new(Vec::new(), START_ID, ConnectionMode::Parsing)
    }
}
