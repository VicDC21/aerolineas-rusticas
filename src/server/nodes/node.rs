//! Módulo de nodos.
use crate::client::cql_frame::query_body::QueryBody;
use crate::protocol::{
    aliases::{results::Result, types::Byte},
    errors::error::Error,
    headers::{
        flags::Flag, length::Length, msg_headers::Headers, opcode::Opcode, stream::Stream,
        version::Version,
    },
    messages::responses::result_kinds::ResultKind,
    notations::consistency::Consistency,
    traits::Byteable,
};
use crate::server::{
    actions::opcode::{GossipInfo, SvAction},
    modes::ConnectionMode,
    utils::load_json,
};
use crate::tokenizer::tokenizer::tokenize_query;
use crate::{
    parser::{
        data_types::keyspace_name::KeyspaceName,
        main_parser::make_parse,
        statements::{
            ddl_statement::{
                alter_keyspace::AlterKeyspace, create_keyspace::CreateKeyspace,
                create_table::CreateTable, ddl_statement_parser::DdlStatement,
                drop_keyspace::DropKeyspace,
            },
            dml_statement::{
                dml_statement_parser::DmlStatement,
                main_statements::{
                    delete::Delete, insert::Insert, select::select_operation::Select,
                    update::Update,
                },
            },
            statement::Statement,
        },
    },
    protocol::utils::{parse_bytes_to_string, parse_bytes_to_string_map},
};
use chrono::Utc;
use serde::{Deserialize, Serialize};

use std::{
    cmp::PartialEq,
    collections::{HashMap, HashSet},
    io::{Read, Write},
    net::TcpStream,
    path::Path,
    thread::JoinHandle,
};

use super::{
    addr::loader::AddrLoader,
    disk_operations::disk_handler::DiskHandler,
    internal_threads::{beater, create_client_and_private_conexion, gossiper},
    keyspace_metadata::keyspace::Keyspace,
    port_type::PortType,
    states::{
        appstatus::AppStatus,
        endpoints::EndpointState,
        heartbeat::{GenType, HeartbeatState, VerType},
    },
    table_metadata::table::Table,
    utils::{
        divide_range, hash_value, next_node_in_the_cluster, send_to_node,
        send_to_node_and_wait_response_with_timeout,
    },
};

/// El ID de un nodo. No se tienen en cuenta casos de cientos de nodos simultáneos,
/// así que un byte debería bastar para representarlo.
pub type NodeId = Byte;
/// Mapea todos los estados de los vecinos y de sí mismo.
pub type NodesMap = HashMap<NodeId, EndpointState>;
/// Mapea todas las conexiones actualmente abiertas.
pub type OpenConnectionsMap = HashMap<Stream, TcpStream>;
/// El handle donde vive una operación de nodo.
pub type NodeHandle = JoinHandle<Result<()>>;

/// Cantidad de nodos fija en cualquier momento.
///
/// DEBE coincidir con la cantidad de nodos en el archivo de IPs `node_ips.csv`.
const N_NODES: Byte = 5;
/// El límite posible para los rangos de los nodos.
const NODES_RANGE_END: u64 = 18446744073709551615;
/// El tiempo de espera _(en segundos)_ por una respuesta.
const TIMEOUT_SECS: u64 = 2;

/// Un nodo es una instancia de parser que se conecta con otros nodos para procesar _queries_.
#[derive(Serialize, Deserialize)]
pub struct Node {
    /// El ID del nodo mismo.
    id: NodeId,

    /// Los estados de los nodos vecinos, incluyendo este mismo.
    ///
    /// No necesariamente serán todos los otros nodos del grafo, sólo los que este nodo conoce.
    #[serde(skip)]
    neighbours_states: NodesMap,

    /// Estado actual del nodo.
    #[serde(skip)]
    endpoint_state: EndpointState,

    /// Dirección de almacenamiento en disco.
    #[serde(skip)]
    storage_addr: String,

    /// Nombre del keyspace por defecto.
    default_keyspace_name: String,

    /// Nombre del keyspace por defecto de cada usuario.
    users_default_keyspace_name: HashMap<String, String>,

    /// Los keyspaces que tiene el nodo.
    /// (nombre, keyspace)
    keyspaces: HashMap<String, Keyspace>,

    /// Las tablas que tiene el nodo.
    /// (nombre, tabla)
    tables: HashMap<String, Table>,

    /// Rangos asignados a cada nodo para determinar la partición de los datos.
    #[serde(skip)]
    nodes_ranges: Vec<(u64, u64)>,

    /// Nombre de la tabla y los valores de las _partitions keys_ que contiene
    tables_and_partitions_keys_values: HashMap<String, Vec<String>>,

    /// Mapa de conexiones abiertas entre el nodo y otros clientes.
    #[serde(skip)]
    open_connections: OpenConnectionsMap,

    /// Los pesos de los nodos.
    nodes_weights: Vec<usize>,
}

impl Node {
    /// Crea un nuevo nodo.
    pub fn new(id: NodeId, mode: ConnectionMode) -> Result<Self> {
        let mut neighbours_states = NodesMap::new();
        let endpoint_state = EndpointState::with_id_and_mode(id, mode);
        neighbours_states.insert(id, endpoint_state.clone());

        Ok(Self {
            id,
            neighbours_states,
            endpoint_state,
            storage_addr: DiskHandler::new_node_storage(id)?,
            default_keyspace_name: "".to_string(),
            users_default_keyspace_name: HashMap::new(),
            keyspaces: HashMap::new(),
            tables: HashMap::new(),
            nodes_ranges: divide_range(0, NODES_RANGE_END, N_NODES as usize),
            tables_and_partitions_keys_values: HashMap::new(),
            open_connections: OpenConnectionsMap::new(),
            nodes_weights: Vec::new(),
        })
    }

    /// Setea el valor por defecto de los campos que no son guardados en su archivo JSON.
    ///
    /// Se asume que esta función se llama sobre un nodo que fue cargado recientemente de su archivo JSON.
    pub fn set_default_fields(&mut self, id: NodeId, mode: ConnectionMode) -> Result<()> {
        let mut neighbours_states = NodesMap::new();
        let endpoint_state = EndpointState::with_id_and_mode(id, mode);
        neighbours_states.insert(id, endpoint_state.clone());

        self.neighbours_states = neighbours_states;
        self.endpoint_state = endpoint_state;
        self.storage_addr = DiskHandler::get_node_storage(id);
        self.nodes_ranges = divide_range(0, NODES_RANGE_END, N_NODES as usize);
        self.open_connections = OpenConnectionsMap::new();

        Ok(())
    }

    /// Inicia un nuevo nodo con un ID específico en modo de conexión _parsing_.
    pub fn init_in_parsing_mode(id: NodeId) -> Result<()> {
        Self::init(id, ConnectionMode::Parsing)
    }

    /// Inicia un nuevo nodo con un ID específico en modo de conexión _echo_.
    pub fn init_in_echo_mode(id: NodeId) -> Result<()> {
        Self::init(id, ConnectionMode::Echo)
    }

    /// Crea un nuevo nodo con un ID específico.
    fn init(id: NodeId, mode: ConnectionMode) -> Result<()> {
        let mut nodes_weights: Vec<usize> = Vec::new();
        let handlers = Self::bootstrap(id, mode, &mut nodes_weights)?;

        let (_beater, _beat_stopper) = beater(id)?;
        let (_gossiper, _gossip_stopper) = gossiper(id, &nodes_weights)?;

        /*Paramos los handlers especiales primero
        let _ = beat_stopper.send(true);
        let _ = beater.join();

        let _ = gossip_stopper.send(true);
        let _ = gossiper.join();*/

        Self::wait(handlers);
        Ok(())
    }

    /// Inicia la metadata y los hilos necesarios para que el nodo se conecte al cluster.
    fn bootstrap(
        id: NodeId,
        mode: ConnectionMode,
        nodes_weights: &mut Vec<usize>,
    ) -> Result<Vec<Option<NodeHandle>>> {
        let nodes_ids = Self::get_nodes_ids();
        if nodes_ids.len() != N_NODES as usize {
            return Err(Error::ServerError(format!(
                "El archivo de IPs de los nodos no tiene la cantidad correcta de nodos. Se esperaban {} nodos, se encontraron {}.",
                N_NODES, nodes_ids.len()
            )));
        }
        if !nodes_ids.contains(&id) {
            return Err(Error::ServerError(format!(
                "El ID {} no está en el archivo de IPs de los nodos.",
                id
            )));
        }

        let mut handlers: Vec<Option<NodeHandle>> = Vec::new();
        let mut node_listeners: Vec<Option<NodeHandle>> = Vec::new();
        let metadata_path = DiskHandler::get_node_metadata_path(id);
        let node_metadata_path = Path::new(&metadata_path);
        let mut node = if node_metadata_path.exists() {
            let mut node: Node = load_json(&metadata_path)?;
            node.set_default_fields(id, mode)?;
            node
        } else {
            Self::new(id, mode)?
        };
        node.inicialize_nodes_weights();
        *nodes_weights = node.nodes_weights.clone();
        let max_weight_id = node.max_weight();

        let cli_socket = node.get_endpoint_state().socket(&PortType::Cli);
        let priv_socket = node.get_endpoint_state().socket(&PortType::Priv);

        create_client_and_private_conexion(node, id, cli_socket, priv_socket, &mut node_listeners)?;

        handlers.append(&mut node_listeners);

        // Llenamos de información al nodo "seed". Arbitrariamente será el último.
        // Cuando fue iniciado el último nodo (el de mayor ID), hacemos el envío de la información,
        // pues asumimos que todos los demás ya fueron iniciados.
        if id == nodes_ids[(N_NODES - 1) as usize] {
            Self::send_states_to_node(max_weight_id);
        }

        Ok(handlers)
    }

    fn inicialize_nodes_weights(&mut self) {
        self.nodes_weights = vec![1; N_NODES as usize];
        self.nodes_weights[(N_NODES - 1) as usize] *= 3; // El último nodo tiene el triple de probabilidades de ser elegido.
    }

    /// Se le ordena a todos los nodos existentes que envien su _endpoint state_ al nodo con el ID dado.
    fn send_states_to_node(id: NodeId) {
        for node_id in Self::get_nodes_ids() {
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

    /// Decide cuál es el nodo con el mayor "peso". Es decir, el que tiene más probabilidades
    /// de ser elegido cuando se los elige "al azar".
    ///
    /// Si todos son iguales, agarra el primero.
    pub fn max_weight(&self) -> NodeId {
        let nodes_ids = Self::get_nodes_ids();
        let mut max_id: usize = 0;
        for i in 0..nodes_ids.len() {
            if self.nodes_weights[i] > self.nodes_weights[max_id] {
                max_id = i;
            }
        }
        nodes_ids[max_id]
    }

    fn add_table(&mut self, table: Table) {
        let table_name = table.get_name().to_string();
        let partition_key: Vec<String> = Vec::new();
        self.tables.insert(table_name.clone(), table);
        self.tables_and_partitions_keys_values
            .insert(table_name, partition_key);
    }

    fn get_table(&self, table_name: &str) -> Result<&Table> {
        match self.tables.get(table_name) {
            Some(table) => Ok(table),
            None => Err(Error::ServerError(format!(
                "La tabla llamada {} no existe",
                table_name
            ))),
        }
    }

    fn table_exists(&self, table_name: &str) -> bool {
        self.tables.contains_key(table_name)
    }

    fn add_keyspace(&mut self, keyspace: Keyspace) {
        self.keyspaces
            .insert(keyspace.get_name().to_string(), keyspace);
    }

    /// Obtiene un keyspace dado el nombre de una tabla.
    fn get_keyspace(&self, table_name: &str) -> Result<&Keyspace> {
        let table = self.get_table(table_name)?;

        match self.keyspaces.get(table.keyspace.as_str()) {
            Some(keyspace) => Ok(keyspace),
            None => Err(Error::ServerError(format!(
                "El keyspace `{}` no existe",
                table.keyspace.as_str()
            ))),
        }
    }

    /// Obtiene un keyspace dado su nombre.
    fn get_keyspace_from_name(&self, keyspace_name: &str) -> Result<&Keyspace> {
        match self.keyspaces.get(keyspace_name) {
            Some(keyspace) => Ok(keyspace),
            None => Err(Error::ServerError(format!(
                "El keyspace `{}` no existe",
                keyspace_name
            ))),
        }
    }

    fn keyspace_exists(&self, keyspace_name: &str) -> bool {
        self.keyspaces.contains_key(keyspace_name)
    }

    fn set_default_keyspace_name(&mut self, keyspace_name: String) -> Result<()> {
        if self.keyspace_exists(&keyspace_name) {
            self.default_keyspace_name = keyspace_name;
            Ok(())
        } else {
            Err(Error::ServerError(format!(
                "El keyspace `{}` no existe",
                keyspace_name
            )))
        }
    }

    fn get_default_keyspace_name(&self) -> Result<String> {
        if !self.default_keyspace_name.is_empty() {
            Ok(self.default_keyspace_name.clone())
        } else {
            Err(Error::ServerError(
                "No se ha seleccionado un keyspace por defecto".to_string(),
            ))
        }
    }

    /// Si se elige un keyspace preferido, se verifica que éste exista y devuelve su nombre.
    /// En caso contrario, devuelve el nombre del keyspace por defecto.
    ///
    /// Devuelve error si alguno de los dos no existe.
    fn choose_available_keyspace_name(
        &self,
        preferred_keyspace_name: Option<String>,
    ) -> Result<String> {
        let default_keyspace_name = self.get_default_keyspace_name()?;

        match preferred_keyspace_name {
            Some(preferred_keyspace_name) => {
                if self.keyspace_exists(&preferred_keyspace_name) {
                    Ok(preferred_keyspace_name.to_string())
                } else {
                    Err(Error::ServerError(format!(
                        "El keyspace llamado {} no existe",
                        preferred_keyspace_name
                    )))
                }
            }
            None => Ok(default_keyspace_name),
        }
    }

    /// Obtiene una copia del ID del nodo.
    pub fn get_id(&self) -> NodeId {
        self.id
    }

    /// Consulta el estado del nodo.
    pub fn get_endpoint_state(&self) -> &EndpointState {
        &self.endpoint_state
    }

    /// Devuelve los IDs de los nodos del cluster. Ordenados de menor a mayor.
    fn get_nodes_ids() -> Vec<NodeId> {
        let mut nodes_ids: Vec<NodeId> = AddrLoader::default_loaded().get_ids();
        nodes_ids.sort();
        nodes_ids
    }

    /// Selecciona un ID de nodo conforme al _hashing_ del valor del _partition key_ y los rangos de los nodos.
    fn select_node(&self, value: &str) -> NodeId {
        let nodes_ids = Self::get_nodes_ids();
        let hash_val = hash_value(value);

        let mut i = 0;
        for (a, b) in &self.nodes_ranges {
            if *a <= hash_val && hash_val < *b {
                return nodes_ids[i];
            }
            i += 1;
        }
        nodes_ids[i]
    }

    /// Manda un mensaje en bytes al nodo correspondiente mediante el _hashing_ del valor del _partition key_.
    fn _send_message(
        &mut self,
        bytes: Vec<Byte>,
        value: String,
        port_type: PortType,
    ) -> Result<()> {
        send_to_node(self.select_node(&value), bytes, port_type)
    }

    /// Manda un mensaje a un nodo específico y espera por la respuesta de este, con un timeout.
    /// Si el timeout se alcanza, se devuelve un buffer vacío.
    ///
    /// `timeout` es medido en segundos.
    fn send_message_and_wait_response_with_timeout(
        &self,
        bytes: Vec<Byte>,
        node_id: Byte,
        port_type: PortType,
        wait_response: bool,
        timeout: u64,
    ) -> Result<Vec<Byte>> {
        send_to_node_and_wait_response_with_timeout(
            node_id,
            bytes,
            port_type,
            wait_response,
            Some(timeout),
        )
    }

    /// Manda un mensaje en bytes a todos los vecinos del nodo.
    fn _notice_all_neighbours(&self, bytes: Vec<Byte>, port_type: PortType) -> Result<()> {
        for neighbour_id in Self::get_nodes_ids() {
            if neighbour_id == self.id {
                continue;
            }
            send_to_node(neighbour_id, bytes.clone(), port_type.clone())?;
        }
        Ok(())
    }

    /// Compara si el _heartbeat_ de un nodo es más nuevo que otro.
    pub fn is_newer(&self, other: &Self) -> bool {
        self.endpoint_state.is_newer(&other.endpoint_state)
    }

    /// Envia su endpoint state al nodo del ID correspondiente.
    fn send_endpoint_state(&mut self, id: NodeId) {
        if let Err(err) = send_to_node(
            id,
            SvAction::NewNeighbour(self.get_endpoint_state().clone()).as_bytes(),
            PortType::Priv,
        ) {
            println!(
                "Ocurrió un error presentando vecinos de un nodo:\n\n{}",
                err
            );
        }
    }

    /// Consulta si ya se tiene un [EndpointState].
    ///
    /// No compara los estados en profundidad, sólo verifica si se tiene un estado
    /// con la misma IP.
    fn _has_endpoint_state(&self, state: &EndpointState) -> bool {
        let guessed_id = match AddrLoader::default_loaded().get_id(state.get_addr()) {
            Ok(guessed_right) => guessed_right,
            Err(_) => return false,
        };
        self.has_endpoint_state_by_id(&guessed_id)
    }

    /// Consulta si ya se tiene un [EndpointState] por ID de nodo.
    ///
    /// No compara los estados en profundidad, sólo verifica si se tiene un estado
    /// con la misma IP.
    fn has_endpoint_state_by_id(&self, node_id: &NodeId) -> bool {
        self.neighbours_states.contains_key(node_id)
    }

    /// Consulta si el nodo todavía esta booteando.
    pub fn is_bootstraping(&self) -> bool {
        matches!(
            self.endpoint_state.get_appstate().get_status(),
            AppStatus::Bootstrap
        )
    }

    /// Consulta el modo de conexión del nodo.
    fn mode(&self) -> &ConnectionMode {
        self.endpoint_state.get_appstate().get_mode()
    }

    fn add_neighbour_state(&mut self, state: EndpointState) -> Result<()> {
        let guessed_id = AddrLoader::default_loaded().get_id(state.get_addr())?;
        if !self.has_endpoint_state_by_id(&guessed_id) {
            self.neighbours_states.insert(guessed_id, state);
        }
        Ok(())
    }

    /// Actualiza la información de vecinos con otro mapa dado.
    ///
    /// No se comprueba si las entradas nuevas son más recientes o no: reemplaza todo sin preguntar.
    fn update_neighbours(&mut self, new_neighbours: NodesMap) -> Result<()> {
        for (node_id, endpoint_state) in new_neighbours {
            self.neighbours_states.insert(node_id, endpoint_state);
        }

        Ok(())
    }

    /// Consulta si un nodo vecino está listo para recibir _queries_.
    fn neighbour_is_responsive(&self, node_id: NodeId) -> bool {
        if let Some(endpoint_state) = self.neighbours_states.get(&node_id) {
            return *endpoint_state.get_appstate_status() == AppStatus::Normal;
        }
        false
    }

    /// Actualiza el estado del nodo recibido a _Offline_.
    fn acknowledge_offline_neighbour(&mut self, node_id: NodeId) {
        if let Some(endpoint_state) = self.neighbours_states.get_mut(&node_id) {
            endpoint_state.set_appstate_status(AppStatus::Offline);
        }
    }

    /// Consulta si el nodo ya está listo para recibir _queries_. Si lo está, actualiza su estado.
    fn is_bootstrap_done(&mut self) {
        if self.neighbours_states.len() == N_NODES as usize
            && *self.endpoint_state.get_appstate_status() != AppStatus::Normal
        {
            self.endpoint_state.set_appstate_status(AppStatus::Normal);
            println!("El nodo {} fue iniciado correctamente.", self.id);
        }
    }

    /// Consulta el estado de _heartbeat_.
    pub fn get_beat(&mut self) -> (GenType, VerType) {
        self.endpoint_state.get_heartbeat().as_tuple()
    }

    /// Avanza el tiempo para el nodo.
    fn beat(&mut self) -> VerType {
        self.endpoint_state.beat();
        self.neighbours_states
            .insert(self.id, self.endpoint_state.clone());
        self.get_beat().1
    }

    /// Consigue la información de _gossip_ que contiene este nodo.
    fn get_gossip_info(&self) -> GossipInfo {
        let mut gossip_info = GossipInfo::new();
        for (node_id, endpoint_state) in &self.neighbours_states {
            gossip_info.insert(node_id.to_owned(), endpoint_state.clone_heartbeat());
        }

        gossip_info
    }

    /// Inicia un intercambio de _gossip_ con los vecinos dados.
    fn gossip(&mut self, neighbours: HashSet<NodeId>) -> Result<()> {
        self.is_bootstrap_done();

        for neighbour_id in neighbours {
            if send_to_node(
                neighbour_id,
                SvAction::Syn(self.get_id().to_owned(), self.get_gossip_info()).as_bytes(),
                PortType::Priv,
            )
            .is_err()
            {
                // No devolvemos error porque no se considera un error que un vecino no responda en esta instancia.
                self.acknowledge_offline_neighbour(neighbour_id);
            }
        }
        Ok(())
    }

    /// Se recibe un mensaje [SYN](crate::server::actions::opcode::SvAction::Syn).
    fn syn(&mut self, emissor_id: NodeId, emissor_gossip_info: GossipInfo) -> Result<()> {
        let mut own_gossip_info = GossipInfo::new(); // quiero info de estos nodos
        let mut response_nodes = NodesMap::new(); // doy info de estos nodos

        self.classify_nodes_in_gossip(
            &emissor_gossip_info,
            &mut own_gossip_info,
            &mut response_nodes,
        );

        // Ahora rondamos nuestros vecinos para ver si tenemos uno que el nodo emisor no
        for (own_node_id, endpoint_state) in &self.neighbours_states {
            if !emissor_gossip_info.contains_key(own_node_id) {
                response_nodes.insert(*own_node_id, endpoint_state.clone());
            }
        }

        if let Err(err) = send_to_node(
            emissor_id,
            SvAction::Ack(self.get_id().to_owned(), own_gossip_info, response_nodes).as_bytes(),
            PortType::Priv,
        ) {
            println!(
                "Ocurrió un error al mandar un mensaje ACK al nodo [{}]:\n\n{}",
                emissor_id, err
            );
        }
        Ok(())
    }

    /// Clasifica los nodos en el _SYN_ recibido. Determina cuales deben ser pedidos (_own_gossip_info_) y cuales
    /// deben ser compartidos _(response_nodes)_.
    fn classify_nodes_in_gossip(
        &mut self,
        emissor_gossip_info: &HashMap<Byte, HeartbeatState>,
        own_gossip_info: &mut HashMap<Byte, HeartbeatState>,
        response_nodes: &mut HashMap<Byte, EndpointState>,
    ) {
        for (node_id, emissor_heartbeat) in emissor_gossip_info {
            match &self.neighbours_states.get(node_id) {
                Some(own_endpoint_state) => {
                    let own_heartbeat = own_endpoint_state.get_heartbeat();
                    if own_heartbeat > emissor_heartbeat
                        || *own_endpoint_state.get_appstate_status() == AppStatus::Offline
                    {
                        // El nodo propio tiene un heartbeat más nuevo que el que se recibió
                        // o
                        // El nodo propio no está listo para recibir queries
                        response_nodes.insert(*node_id, (*own_endpoint_state).clone());
                    } else if own_heartbeat < emissor_heartbeat {
                        // El nodo propio tiene un heartbeat más viejo que el que se recibió
                        own_gossip_info.insert(*node_id, own_heartbeat.clone());
                    }
                }
                None => {
                    // Se trata de un vecino que no conocemos aún
                    own_gossip_info.insert(*node_id, HeartbeatState::minimal());
                }
            }
        }
    }

    /// Se recibe un mensaje [ACK](crate::server::actions::opcode::SvAction::Ack).
    fn ack(
        &mut self,
        receptor_id: NodeId,
        receptor_gossip_info: GossipInfo,
        response_nodes: NodesMap,
    ) -> Result<()> {
        // Poblamos un mapa con los estados que pide el receptor
        let mut nodes_for_receptor = NodesMap::new();
        for (node_id, receptor_heartbeat) in &receptor_gossip_info {
            let own_endpoint_state = &self.neighbours_states[node_id];
            if own_endpoint_state.get_heartbeat() > receptor_heartbeat {
                // Hacemos doble chequeo que efectivamente tenemos información más nueva
                nodes_for_receptor.insert(*node_id, own_endpoint_state.clone());
            }
        }

        // Reemplazamos la información de nuestros vecinos por la más nueva que viene del nodo receptor
        // Asumimos que es más nueva ya que fue previamente verificada
        self.update_neighbours(response_nodes)?;

        if let Err(err) = send_to_node(
            receptor_id,
            SvAction::Ack2(nodes_for_receptor).as_bytes(),
            PortType::Priv,
        ) {
            println!(
                "Ocurrió un error al mandar un mensaje ACK2 al nodo [{}]:\n\n{}",
                receptor_id, err
            );
        }
        Ok(())
    }

    /// Se recibe un mensaje [ACK2](crate::server::actions::opcode::SvAction::Ack2).
    fn ack2(&mut self, nodes_map: NodesMap) -> Result<()> {
        self.update_neighbours(nodes_map)
    }

    /// Limpia las conexiones cerradas.
    ///
    /// Devuelve las conexiones que logró cerrar.
    pub fn clean_closed_connections(&mut self) -> usize {
        let mut closed_count = 0;

        self.open_connections.retain(|_, tcp_stream| {
            if tcp_stream.peer_addr().is_ok() {
                return true;
            }
            closed_count += 1;
            false
        });

        closed_count
    }

    /// Procesa una _request_ en forma de [Byte]s.
    /// También devuelve un [bool] indicando si se debe parar el hilo.
    pub fn process_stream<S>(
        &mut self,
        stream: &mut S,
        bytes: Vec<Byte>,
        is_logged: bool,
    ) -> Result<Vec<Byte>>
    where
        S: Read + Write,
    {
        if bytes.is_empty() {
            return Ok(vec![]);
        }
        // println!("Esta en process_tcp");
        match SvAction::get_action(&bytes[..]) {
            Some(action) => {
                if let Err(err) = self.handle_sv_action(action, stream) {
                    println!(
                        "[{} - ACTION] Error en la acción del servidor: {}",
                        self.id, err
                    );
                }
                Ok(vec![])
            }
            None => self.match_kind_of_conection_mode(bytes, stream, is_logged),
        }
    }

    /// Maneja una acción de servidor.
    fn handle_sv_action<S>(&mut self, action: SvAction, mut tcp_stream: S) -> Result<bool>
    where
        S: Read + Write,
    {
        let mut stop = false;
        match action {
            SvAction::Exit => stop = true, // La comparación para salir ocurre en otro lado
            SvAction::Beat => {
                self.beat();
            }
            SvAction::Gossip(neighbours) => {
                self.gossip(neighbours)?;
            }
            SvAction::Syn(emissor_id, gossip_info) => {
                self.syn(emissor_id, gossip_info)?;
            }
            SvAction::Ack(receptor_id, gossip_info, nodes_map) => {
                self.ack(receptor_id, gossip_info, nodes_map)?;
            }
            SvAction::Ack2(nodes_map) => {
                self.ack2(nodes_map)?;
            }
            SvAction::NewNeighbour(state) => {
                self.add_neighbour_state(state)?;
            }
            SvAction::SendEndpointState(id) => {
                self.send_endpoint_state(id);
            }
            SvAction::InternalQuery(bytes) => {
                let response = self.handle_request(&bytes, true, true);
                let _ = tcp_stream.write_all(&response[..]);
                if let Err(err) = tcp_stream.flush() {
                    return Err(Error::ServerError(err.to_string()));
                };
            }
            SvAction::StoreMetadata => {
                if let Err(err) = DiskHandler::store_node_metadata(self) {
                    return Err(Error::ServerError(format!(
                        "Error guardando metadata del nodo {}: {}",
                        &self.id, err
                    )));
                }
            }
            SvAction::DirectReadRequest(bytes) => {
                let res = self.exec_direct_read_request(bytes)?;
                let _ = tcp_stream.write_all(res.as_bytes());
                if let Err(err) = tcp_stream.flush() {
                    return Err(Error::ServerError(err.to_string()));
                };
            }
            SvAction::DigestReadRequest(bytes) => {
                let res = self.exec_digest_read_request(bytes);
                let _ = tcp_stream.write_all(&res);
                if let Err(err) = tcp_stream.flush() {
                    return Err(Error::ServerError(err.to_string()));
                };
            }
            SvAction::RepairRows(table_name, node_id, rows_bytes) => {
                self.repair_rows(table_name, node_id, rows_bytes)?;
            }
            SvAction::AddPartitionValueToMetadata(table_name, partition_value) => {
                let table = self.get_table(&table_name)?;
                match self.check_if_has_new_partition_value(
                    partition_value,
                    &table.get_name().to_string(),
                )? {
                    Some(new_partition_values) => self
                        .tables_and_partitions_keys_values
                        .insert(table_name, new_partition_values),
                    None => None,
                };
            }
        };
        Ok(stop)
    }

    fn exec_direct_read_request(&self, mut bytes: Vec<Byte>) -> Result<String> {
        let node_number = match bytes.pop() {
            Some(node_number) => node_number,
            None => {
                return Err(Error::ServerError(
                    "No se especificó el ID del nodo al hacer read-repair".to_string(),
                ))
            }
        };
        let bytes_borrowed: &[u8] = &bytes;
        let statement = match QueryBody::try_from(bytes_borrowed) {
            Ok(query_body) => match make_parse(&mut tokenize_query(query_body.get_query())) {
                Ok(statement) => statement,
                Err(_err) => {
                    return Err(Error::ServerError(
                        "No se pudo parsear el statement al hacer read-repair".to_string(),
                    ))
                }
            },
            Err(_err) => {
                return Err(Error::ServerError(
                    "No se pudo parsear el body de la query al hacer read-repair".to_string(),
                ))
            }
        };
        let select = match statement {
            Statement::DmlStatement(DmlStatement::SelectStatement(select)) => select,
            _ => {
                return Err(Error::ServerError(
                    "La declaración no es un SELECT".to_string(),
                ))
            }
        };

        let res = DiskHandler::get_rows_with_timestamp_as_string(
            &self.storage_addr,
            &self.get_default_keyspace_name()?,
            &select,
            node_number,
        )?;
        Ok(res)
    }

    fn exec_digest_read_request(&mut self, bytes: Vec<Byte>) -> Vec<Byte> {
        let response = self.handle_request(&bytes, true, true);
        // Devolvemos además un opcode para poder saber si el resultado fue un error o no.
        if verify_succesful_response(&response) {
            let mut res = Opcode::Result.as_bytes();
            res.extend_from_slice(&hash_value(&response).to_be_bytes());
            res
        } else {
            let mut res = Opcode::RequestError.as_bytes();
            res.extend_from_slice(&response);
            res
        }
    }

    fn repair_rows(&self, table_name: String, node_id: Byte, rows_bytes: Vec<Byte>) -> Result<()> {
        if !self.table_exists(&table_name) {
            return Err(Error::ServerError(format!(
                "La tabla `{}` no existe",
                table_name
            )));
        }

        let table = self.get_table(&table_name)?;
        let keyspace_name = table.get_keyspace();
        if !self.keyspace_exists(keyspace_name) {
            return Err(Error::ServerError(format!(
                "El keyspace `{}` asociado a la tabla `{}` no existe",
                keyspace_name, table_name
            )));
        }
        let rows = String::from_utf8(rows_bytes)
            .map_err(|_| Error::ServerError("Error al castear de bytes a string".to_string()))?;
        DiskHandler::repair_rows(
            &self.storage_addr,
            &table_name,
            keyspace_name,
            &self.get_default_keyspace_name()?,
            node_id,
            &rows,
        )
    }

    /// Maneja una request.
    fn handle_request(
        &mut self,
        request: &[Byte],
        is_internal_request: bool,
        is_logged: bool,
    ) -> Vec<Byte> {
        if request.len() < 9 {
            return Vec::<Byte>::new();
        }
        let header = match Headers::try_from(&request[..9]) {
            Ok(header) => header,
            Err(err) => return make_error_response(err),
        };
        let left_response = match header.opcode {
            Opcode::Startup => self.handle_startup(&request[9..]),
            Opcode::Options => self.handle_options(),
            Opcode::Query => {
                self.handle_query(request, &header.length, is_internal_request, is_logged)
            }
            Opcode::Prepare => self.handle_prepare(),
            Opcode::Execute => self.handle_execute(),
            Opcode::Register => self.handle_register(),
            Opcode::Batch => self.handle_batch(),
            Opcode::AuthResponse => self.handle_auth_response(request, &header.length),
            _ => Err(Error::ProtocolError(
                "El opcode recibido no es una request".to_string(),
            )),
        };
        match left_response {
            Ok(value) => wrap_header(value, is_internal_request, header),
            Err(err) => wrap_header(make_error_response(err), is_internal_request, header),
        }
    }

    fn handle_startup(&self, request_body: &[Byte]) -> Result<Vec<Byte>> {
        // Si tuviesemos la opcion del READY pondriamos un if
        let string_map = parse_bytes_to_string_map(request_body)?;
        if string_map.is_empty() {
            return Ok(make_error_response(Error::ConfigError(
                "En el startup se debia mandar al menos la version CQL".to_string(),
            )));
        }
        if string_map[0].1 != "5.0.0" {
            return Ok(make_error_response(Error::ConfigError(format!(
                "{} es una version CQL no soportada",
                string_map[0].1
            ))));
        }
        let mut response: Vec<Byte> = Vec::new();
        response.append(&mut Version::ResponseV5.as_bytes());
        response.append(&mut Flag::Default.as_bytes());
        response.append(&mut Stream::new(0).as_bytes());
        response.append(&mut Opcode::AuthChallenge.as_bytes());
        response.append(&mut Length::new(0).as_bytes()); // REVISAR ESTO
        Ok(response)
    }

    fn handle_options(&self) -> Result<Vec<Byte>> {
        // No tiene body
        // Responder con supported
        Opcode::Supported.as_bytes();
        Ok(vec![0])
    }

    fn handle_query(
        &mut self,
        request: &[Byte],
        lenght: &Length,
        internal_request: bool,
        is_logged: bool,
    ) -> Result<Vec<Byte>> {
        if !is_logged {
            return Err(Error::AuthenticationError(
                "No se pueden mandar queries antes de autenticar el usuario".to_string(),
            ));
        }
        if let Ok(query_body) = QueryBody::try_from(&request[9..(lenght.len as usize) + 9]) {
            let res = match make_parse(&mut tokenize_query(query_body.get_query())) {
                Ok(statement) => {
                    if internal_request {
                        let mut internal_metadata: Vec<Byte> = Vec::new();
                        if request.len() > (lenght.len as usize) + 9 {
                            internal_metadata
                                .append(&mut request[(lenght.len as usize) + 9..].to_vec());
                        }
                        let internal_metadata =
                            self.read_metadata_from_internal_request(internal_metadata);
                        self.handle_internal_statement(statement, internal_metadata)
                    } else {
                        self.handle_statement(
                            statement,
                            request,
                            query_body.get_consistency_level(),
                        )
                    }
                }
                Err(err) => {
                    return Err(err);
                }
            };
            return res;
            // aca usariamos la query como corresponda
        }
        Err(Error::ServerError(
            "No se pudieron transformar los bytes al body de la query".to_string(),
        ))
    }

    fn handle_prepare(&self) -> Result<Vec<Byte>> {
        // El body es <query><flags>[<keyspace>]
        Ok(vec![0])
    }

    fn handle_execute(&self) -> Result<Vec<Byte>> {
        // El body es <id><result_metadata_id><query_parameters>
        Ok(vec![0])
    }

    fn handle_register(&self) -> Result<Vec<Byte>> {
        Ok(vec![0])
    }

    fn handle_batch(&self) -> Result<Vec<Byte>> {
        Ok(vec![0])
    }

    fn handle_auth_response(&mut self, request: &[Byte], lenght: &Length) -> Result<Vec<Byte>> {
        let req = &request[9..(lenght.len as usize) + 9];
        let users = DiskHandler::read_admitted_users(&self.storage_addr)?;
        let mut response: Vec<Byte> = Vec::new();
        let mut i = 0;
        let user_from_req = parse_bytes_to_string(req, &mut i)?;
        let password_from_req = parse_bytes_to_string(&req[i..], &mut i)?;
        for user in users {
            if user.0 == user_from_req && user.1 == password_from_req {
                response.append(&mut Version::ResponseV5.as_bytes());
                response.append(&mut Flag::Default.as_bytes());
                response.append(&mut Stream::new(0).as_bytes());
                response.append(&mut Opcode::AuthSuccess.as_bytes());
                response.append(&mut Length::new(0).as_bytes());

                if !self.users_default_keyspace_name.contains_key(&user.0) {
                    self.users_default_keyspace_name
                        .insert(user.0.to_string(), "".to_string());
                }
                return Ok(response);
            }
        }
        response = make_error_response(Error::AuthenticationError(
            "Las credenciales pasadas no son validas".to_string(),
        ));
        Ok(response)
    }

    /// Maneja una declaración interna.
    fn handle_internal_statement(
        &mut self,
        statement: Statement,
        internal_metadata: (Option<i64>, Option<Byte>),
    ) -> Result<Vec<Byte>> {
        match statement {
            Statement::DdlStatement(ddl_statement) => {
                self.handle_internal_ddl_statement(ddl_statement, internal_metadata)
            }
            Statement::DmlStatement(dml_statement) => {
                self.handle_internal_dml_statement(dml_statement, internal_metadata)
            }
            Statement::UdtStatement(_udt_statement) => todo!(),
            Statement::Startup => Err(Error::Invalid(
                "No se deberia haber mandado el startup por este canal".to_string(),
            )),
            Statement::LoginUser(_) => Err(Error::Invalid(
                "No se deberia haber mandado el login por este canal".to_string(),
            )),
        }
    }

    /// Maneja una declaración DDL.
    fn handle_internal_ddl_statement(
        &mut self,
        ddl_statement: DdlStatement,
        internal_metadata: (Option<i64>, Option<Byte>),
    ) -> Result<Vec<Byte>> {
        match ddl_statement {
            DdlStatement::UseStatement(keyspace_name) => {
                self.process_internal_use_statement(&keyspace_name)
            }
            DdlStatement::CreateKeyspaceStatement(create_keyspace) => {
                self.process_internal_create_keyspace_statement(&create_keyspace)
            }
            DdlStatement::AlterKeyspaceStatement(alter_keyspace) => {
                self.process_internal_alter_keyspace_statement(&alter_keyspace)
            }
            DdlStatement::DropKeyspaceStatement(drop_keyspace) => {
                self.process_internal_drop_keyspace_statement(&drop_keyspace)
            }
            DdlStatement::CreateTableStatement(create_table) => match internal_metadata.1 {
                Some(node_number) => {
                    self.process_internal_create_table_statement(&create_table, node_number)
                }
                None => Err(Error::ServerError(
                    "No se paso metadata necesaria".to_string(),
                )),
            },
            DdlStatement::AlterTableStatement(_alter_table) => todo!(),
            DdlStatement::DropTableStatement(_drop_table) => todo!(),
            DdlStatement::TruncateStatement(_truncate) => todo!(),
        }
    }

    fn check_if_keyspace_exists(&self, keyspace_name: &KeyspaceName) -> bool {
        let keyspace_addr = format!("{}/{}", self.storage_addr, keyspace_name.get_name());
        let path_folder = Path::new(&keyspace_addr);
        path_folder.exists() && path_folder.is_dir()
    }

    fn process_use_statement(
        &mut self,
        keyspace_name: KeyspaceName,
        request: &[Byte],
    ) -> Result<Vec<Byte>> {
        let mut response: Vec<Byte> = Vec::new();
        let mut actual_node_id = self.id;
        let nodes_ids = Self::get_nodes_ids();
        for _ in 0..N_NODES {
            response = if actual_node_id != self.id {
                self.send_message_and_wait_response_with_timeout(
                    SvAction::InternalQuery(request.to_vec()).as_bytes(),
                    actual_node_id,
                    PortType::Priv,
                    true,
                    TIMEOUT_SECS,
                )?
            } else {
                self.process_internal_use_statement(&keyspace_name)?
            };
            actual_node_id = next_node_in_the_cluster(actual_node_id, &nodes_ids);
        }
        Ok(response)
    }

    fn process_internal_use_statement(
        &mut self,
        keyspace_name: &KeyspaceName,
    ) -> Result<Vec<Byte>> {
        let name = keyspace_name.get_name().to_string();
        if self.keyspaces.contains_key(&name) {
            self.set_default_keyspace_name(name.clone())?;
            Ok(Self::create_result_void())
        } else {
            if self.check_if_keyspace_exists(keyspace_name) {
                self.set_default_keyspace_name(name.clone())?;
                return Ok(Self::create_result_void());
            }
            Err(Error::ServerError(
                "El keyspace solicitado no existe".to_string(),
            ))
        }
    }

    fn process_create_keyspace_statement(
        &mut self,
        create_keyspace: CreateKeyspace,
        request: &[Byte],
    ) -> Result<Vec<Byte>> {
        let mut response: Vec<Byte> = Vec::new();
        let mut actual_node_id = self.id;
        let nodes_ids = Self::get_nodes_ids();
        for _ in 0..N_NODES {
            response = if actual_node_id != self.id {
                self.send_message_and_wait_response_with_timeout(
                    SvAction::InternalQuery(request.to_vec()).as_bytes(),
                    actual_node_id,
                    PortType::Priv,
                    true,
                    TIMEOUT_SECS,
                )?
            } else {
                self.process_internal_create_keyspace_statement(&create_keyspace)?
            };
            actual_node_id = next_node_in_the_cluster(actual_node_id, &nodes_ids);
        }
        Ok(response)
    }

    fn process_internal_create_keyspace_statement(
        &mut self,
        create_keyspace: &CreateKeyspace,
    ) -> Result<Vec<Byte>> {
        match DiskHandler::create_keyspace(create_keyspace, &self.storage_addr) {
            Ok(Some(keyspace)) => self.add_keyspace(keyspace),
            Ok(None) => return Ok(Self::create_result_void()),
            Err(err) => return Err(err),
        };
        Ok(Self::create_result_void())
    }

    fn process_alter_statement(
        &mut self,
        alter_keyspace: AlterKeyspace,
        request: &[Byte],
    ) -> Result<Vec<Byte>> {
        let keyspace_name = alter_keyspace.name.get_name();
        if !self.keyspaces.contains_key(keyspace_name) && !alter_keyspace.if_exists {
            return Err(Error::ServerError(format!(
                "El keyspace {} no existe",
                keyspace_name
            )));
        }

        let mut responses = Vec::new();
        let mut actual_node_id = self.id;
        let nodes_ids = Self::get_nodes_ids();
        for _ in 0..N_NODES {
            let response = if actual_node_id != self.id {
                self.send_message_and_wait_response_with_timeout(
                    SvAction::InternalQuery(request.to_vec()).as_bytes(),
                    actual_node_id,
                    PortType::Priv,
                    true,
                    TIMEOUT_SECS,
                )?
            } else {
                self.process_internal_alter_keyspace_statement(&alter_keyspace)?
            };
            responses.push(response);
            actual_node_id = next_node_in_the_cluster(actual_node_id, &nodes_ids);
        }
        Ok(Self::create_result_void())
    }

    fn process_internal_alter_keyspace_statement(
        &mut self,
        alter_keyspace: &AlterKeyspace,
    ) -> Result<Vec<Byte>> {
        let keyspace_name = alter_keyspace.name.get_name();
        match self.keyspaces.get_mut(keyspace_name) {
            Some(keyspace) => {
                if let Ok(Some(new_replication)) =
                    DiskHandler::get_keyspace_replication(alter_keyspace.get_options())
                {
                    keyspace.set_replication(new_replication);
                }
                Ok(Self::create_result_void())
            }
            None => {
                if alter_keyspace.if_exists {
                    Ok(Self::create_result_void())
                } else {
                    Err(Error::ServerError(format!(
                        "El keyspace {} no existe",
                        keyspace_name
                    )))
                }
            }
        }
    }

    fn process_drop_keyspace_statement(
        &mut self,
        drop_keyspace: DropKeyspace,
        request: &[Byte],
    ) -> Result<Vec<Byte>> {
        let keyspace_name = drop_keyspace.name.get_name();
        if !self.keyspaces.contains_key(keyspace_name) && !drop_keyspace.if_exists {
            return Err(Error::ServerError(format!(
                "El keyspace {} no existe",
                keyspace_name
            )));
        }

        let mut responses = Vec::new();
        let mut actual_node_id = self.id;
        let nodes_ids = Self::get_nodes_ids();
        for _ in 0..N_NODES {
            let response = if actual_node_id != self.id {
                self.send_message_and_wait_response_with_timeout(
                    SvAction::InternalQuery(request.to_vec()).as_bytes(),
                    actual_node_id,
                    PortType::Priv,
                    true,
                    TIMEOUT_SECS,
                )?
            } else {
                self.process_internal_drop_keyspace_statement(&drop_keyspace)?
            };
            responses.push(response);
            actual_node_id = next_node_in_the_cluster(actual_node_id, &nodes_ids);
        }
        Ok(Self::create_result_void())
    }

    fn process_internal_drop_keyspace_statement(
        &mut self,
        drop_keyspace: &DropKeyspace,
    ) -> Result<Vec<Byte>> {
        let keyspace_name = drop_keyspace.name.get_name();
        if self.keyspaces.contains_key(keyspace_name) {
            self.keyspaces.remove(keyspace_name);
            match DiskHandler::drop_keyspace(keyspace_name, &self.storage_addr) {
                Ok(_) => Ok(Self::create_result_void()),
                Err(e) => Err(e),
            }
        } else if drop_keyspace.if_exists {
            Ok(Self::create_result_void())
        } else {
            Err(Error::ServerError(format!(
                "El keyspace {} no existe",
                keyspace_name
            )))
        }
    }

    fn create_result_void() -> Vec<Byte> {
        let mut response: Vec<Byte> = Vec::new();
        response.append(&mut Version::ResponseV5.as_bytes());
        response.append(&mut Flag::Default.as_bytes());
        response.append(&mut Stream::new(0).as_bytes());
        response.append(&mut Opcode::Result.as_bytes());
        response.append(&mut Length::new(4).as_bytes());
        response.append(&mut ResultKind::Void.as_bytes());
        response
    }

    fn process_internal_create_table_statement(
        &mut self,
        create_table: &CreateTable,
        node_number: u8,
    ) -> Result<Vec<Byte>> {
        let default_keyspace_name = self.get_default_keyspace_name()?;

        match DiskHandler::create_table(
            create_table,
            &self.storage_addr,
            &default_keyspace_name,
            node_number,
        ) {
            Ok(Some(table)) => {
                self.add_table(table);
            }
            Ok(None) => return Err(Error::ServerError("No se pudo crear la tabla".to_string())),
            Err(err) => return Err(err),
        };
        Ok(Self::create_result_void())
    }

    fn process_create_table_statement(
        &mut self,
        create_table: CreateTable,
        request: &[Byte],
    ) -> Result<Vec<Byte>> {
        let keyspace_name =
            self.choose_available_keyspace_name(create_table.name.get_keyspace())?;
        let keyspace = self.get_keyspace_from_name(&keyspace_name)?;
        let quantity_replicas = self.get_quantity_of_replicas_from_keyspace(keyspace)?;
        let mut response: Vec<Byte> = Vec::new();
        let nodes_ids = Self::get_nodes_ids();
        for actual_node_id in &nodes_ids {
            let mut next_node_id = *actual_node_id;
            for _ in 0..quantity_replicas {
                response = if next_node_id == self.id {
                    self.process_internal_create_table_statement(&create_table, *actual_node_id)?
                } else {
                    let request_with_metadata = add_metadata_to_internal_request_of_any_kind(
                        SvAction::InternalQuery(request.to_vec()).as_bytes(),
                        None,
                        Some(*actual_node_id),
                    );
                    self.send_message_and_wait_response_with_timeout(
                        request_with_metadata,
                        next_node_id,
                        PortType::Priv,
                        true,
                        TIMEOUT_SECS,
                    )?
                };
                next_node_id = next_node_in_the_cluster(next_node_id, &nodes_ids);
            }
        }
        Ok(response)
    }

    /// Maneja una declaración DML.
    fn handle_internal_dml_statement(
        &mut self,
        dml_statement: DmlStatement,
        internal_metadata: (Option<i64>, Option<Byte>),
    ) -> Result<Vec<Byte>> {
        let node_number = match internal_metadata.1 {
            Some(value) => value,
            None => {
                return Err(Error::ServerError(
                    "No se paso la informacion del nodo en la metadata interna".to_string(),
                ))
            }
        };
        match dml_statement {
            DmlStatement::SelectStatement(select) => self.process_select(&select, node_number),
            DmlStatement::InsertStatement(insert) => {
                let timestamp = match internal_metadata.0 {
                    Some(value) => value,
                    None => {
                        return Err(Error::ServerError(
                            "No se paso la informacion del timestamp en la metadata interna"
                                .to_string(),
                        ))
                    }
                };
                self.process_insert(&insert, timestamp, node_number)
            }
            DmlStatement::UpdateStatement(update) => {
                let timestamp = match internal_metadata.0 {
                    Some(value) => value,
                    None => {
                        return Err(Error::ServerError(
                            "No se paso la informacion del timestamp en la metadata interna"
                                .to_string(),
                        ))
                    }
                };
                self.process_update(&update, timestamp, node_number)
            }
            DmlStatement::DeleteStatement(delete) => self.process_delete(&delete, node_number),
            DmlStatement::BatchStatement(_batch) => todo!(),
        }
    }

    fn process_select(&self, select: &Select, node_id: Byte) -> Result<Vec<Byte>> {
        let table = match self.get_table(&select.from.get_name()) {
            Ok(table) => table,
            Err(err) => return Err(err),
        };
        let mut res = DiskHandler::do_select(
            select,
            &self.storage_addr,
            table,
            &self.get_default_keyspace_name()?,
            node_id,
        )?;

        Ok(Self::create_result_select(&mut res))
    }

    fn create_result_select(res: &mut Vec<Byte>) -> Vec<Byte> {
        let mut response: Vec<Byte> = Vec::new();
        response.append(&mut Version::ResponseV5.as_bytes());
        response.append(&mut Flag::Default.as_bytes());
        response.append(&mut Stream::new(0).as_bytes());
        response.append(&mut Opcode::Result.as_bytes());
        response.append(&mut Length::new(res.len() as u32).as_bytes());
        response.append(res);
        response
    }

    fn process_insert(
        &mut self,
        insert: &Insert,
        timestamp: i64,
        node_number: Byte,
    ) -> Result<Vec<Byte>> {
        let table = match self.get_table(&insert.table.get_name()) {
            Ok(table) => table,
            Err(err) => return Err(err),
        };
        DiskHandler::do_insert(
            insert,
            &self.storage_addr,
            table,
            &self.get_default_keyspace_name()?,
            timestamp,
            node_number,
        )?;
        let partition_value = self.get_partition_value_from_insert(insert, table)?;
        match self.check_if_has_new_partition_value(partition_value, &insert.get_table_name())? {
            Some(new_partition_values) => self
                .tables_and_partitions_keys_values
                .insert(insert.table.get_name().to_string(), new_partition_values),
            None => None,
        };
        Ok(Self::create_result_void())
    }

    fn check_if_has_new_partition_value(
        &self,
        partition_value: String,
        table_name: &String,
    ) -> Result<Option<Vec<String>>> {
        let mut partition_values: Vec<String> =
            match self.tables_and_partitions_keys_values.get(table_name) {
                Some(partition_values) => partition_values.clone(),
                None => {
                    return Err(Error::ServerError(format!(
                        "La tabla llamada {} no existe",
                        table_name
                    )))
                }
            };
        if !partition_values.contains(&partition_value) {
            partition_values.push(partition_value.clone());
            return Ok(Some(partition_values));
        };
        Ok(None)
    }

    fn get_partition_value_from_insert(&self, insert: &Insert, table: &Table) -> Result<String> {
        let insert_columns = insert.get_columns_names();
        let insert_column_values = insert.get_values();

        let position = match insert_columns
            .iter()
            .position(|x| x == &table.get_partition_key()[0])
        {
            Some(position) => position,
            None => {
                return Err(Error::SyntaxError(
                    "No se mando la partition key en la query del insert".to_string(),
                ))
            }
        };
        Ok(insert_column_values[position].to_string())
    }

    fn process_update(
        &mut self,
        update: &Update,
        timestamp: i64,
        node_number: Byte,
    ) -> Result<Vec<Byte>> {
        let table = match self.get_table(&update.table_name.get_name()) {
            Ok(table) => table,
            Err(err) => return Err(err),
        };
        DiskHandler::do_update(
            update,
            &self.storage_addr,
            table,
            &self.get_default_keyspace_name()?,
            timestamp,
            node_number,
        )?;
        Ok(Self::create_result_void())
    }

    fn handle_statement(
        &mut self,
        statement: Statement,
        request: &[Byte],
        consistency_level: &Consistency,
    ) -> Result<Vec<Byte>> {
        match statement {
            Statement::DdlStatement(ddl_statement) => {
                self.handle_ddl_statement(ddl_statement, request)
            }
            Statement::DmlStatement(dml_statement) => {
                self.handle_dml_statement(dml_statement, request, consistency_level)
            }
            Statement::UdtStatement(_udt_statement) => todo!(),
            Statement::Startup => Err(Error::Invalid(
                "No se deberia haber mandado el startup por este canal".to_string(),
            )),
            Statement::LoginUser(_) => Err(Error::Invalid(
                "No se deberia haber mandado el login por este canal".to_string(),
            )),
        }
    }

    fn handle_ddl_statement(
        &mut self,
        ddl_statement: DdlStatement,
        request: &[Byte],
    ) -> Result<Vec<Byte>> {
        match ddl_statement {
            DdlStatement::UseStatement(keyspace_name) => {
                self.process_use_statement(keyspace_name, request)
            }
            DdlStatement::CreateKeyspaceStatement(create_keyspace) => {
                self.process_create_keyspace_statement(create_keyspace, request)
            }
            DdlStatement::AlterKeyspaceStatement(alter_keyspace) => {
                self.process_alter_statement(alter_keyspace, request)
            }
            DdlStatement::DropKeyspaceStatement(drop_keyspace) => {
                self.process_drop_keyspace_statement(drop_keyspace, request)
            }
            DdlStatement::CreateTableStatement(create_table) => {
                self.process_create_table_statement(create_table, request)
            }
            DdlStatement::AlterTableStatement(_alter_table) => {
                todo!()
            }
            DdlStatement::DropTableStatement(_drop_table) => {
                todo!()
            }
            DdlStatement::TruncateStatement(_truncate) => {
                todo!()
            }
        }
    }

    fn handle_dml_statement(
        &mut self,
        dml_statement: DmlStatement,
        request: &[Byte],
        consistency_level: &Consistency,
    ) -> Result<Vec<Byte>> {
        match dml_statement {
            DmlStatement::SelectStatement(select) => {
                self.select_with_other_nodes(select, request, consistency_level)
            }
            DmlStatement::InsertStatement(insert) => {
                self.insert_with_other_nodes(insert, request, consistency_level)
            }
            DmlStatement::UpdateStatement(update) => {
                self.update_with_other_nodes(update, request, consistency_level)
            }
            DmlStatement::DeleteStatement(delete) => {
                self.delete_with_other_nodes(delete, request, consistency_level)
            }
            DmlStatement::BatchStatement(_batch) => todo!(),
        }
    }

    fn insert_with_other_nodes(
        &mut self,
        insert: Insert,
        request: &[Byte],
        consistency_level: &Consistency,
    ) -> Result<Vec<Byte>> {
        let timestamp = Utc::now().timestamp();
        let table_name: String = insert.table.get_name();
        // let partitions_keys_to_nodes = self.get_partition_keys_values(&table_name)?.clone();
        let mut response: Vec<Byte> = Vec::new();
        let partition_key_value = self
            .get_partition_key_value_from_insert_statement(&insert, self.get_table(&table_name)?)?;
        let node_id = self.select_node(&partition_key_value);
        let replication_factor_quantity = self.get_replicas_from_table_name(&table_name)?;
        let consistency_number = consistency_level.as_usize(replication_factor_quantity as usize);
        let mut consistency_counter = 0;
        let mut wait_response = true;

        let nodes_ids = Self::get_nodes_ids();
        let mut node_to_replicate = node_id;
        for i in 0..N_NODES {
            if (i as u32) < replication_factor_quantity {
                response = if node_to_replicate == self.id {
                    self.process_insert(&insert, timestamp, node_id)?
                } else {
                    let request_with_metadata = add_metadata_to_internal_request_of_any_kind(
                        SvAction::InternalQuery(request.to_vec()).as_bytes(),
                        Some(timestamp),
                        Some(node_id),
                    );
                    self.send_message_and_wait_response_with_timeout(
                        request_with_metadata,
                        node_to_replicate,
                        PortType::Priv,
                        wait_response,
                        TIMEOUT_SECS,
                    )?
                }
            } else if node_to_replicate == self.id {
                let table = self.get_table(&table_name)?;
                let partition_value =
                    self.get_partition_key_value_from_insert_statement(&insert, table)?;
                match self.check_if_has_new_partition_value(
                    partition_value,
                    &table.get_name().to_string(),
                )? {
                    Some(new_partition_values) => self
                        .tables_and_partitions_keys_values
                        .insert(insert.table.get_name().to_string(), new_partition_values),
                    None => None,
                };
            } else {
                let partition_value =
                    self.get_partition_value_from_insert(&insert, self.get_table(&table_name)?)?;
                let request_with_metadata = add_metadata_to_internal_request_of_any_kind(
                    SvAction::AddPartitionValueToMetadata(table_name.clone(), partition_value)
                        .as_bytes(),
                    None,
                    None,
                );
                self.send_message_and_wait_response_with_timeout(
                    request_with_metadata,
                    node_to_replicate,
                    PortType::Priv,
                    false,
                    TIMEOUT_SECS,
                )?;
            };
            node_to_replicate = next_node_in_the_cluster(node_to_replicate, &nodes_ids);

            if consistency_counter >= consistency_number {
                wait_response = false;
            } else if verify_succesful_response(&response) {
                consistency_counter += 1;
            }
        }

        if consistency_counter < consistency_number {
            Err(Error::ServerError(format!(
                "No se pudo cumplir con el nivel de consistencia {}, solo se logró con {} de {}",
                consistency_level, consistency_counter, consistency_number,
            )))
        } else {
            Ok(response)
        }
    }

    /// Revisa si hay metadata extra necesaria para la query pedida
    fn read_metadata_from_internal_request(
        &self,
        internal_metadata: Vec<Byte>,
    ) -> (Option<i64>, Option<Byte>) {
        if internal_metadata.len() == 9 {
            let bytes: [u8; 8] = match internal_metadata[0..8].try_into() {
                Ok(value) => value,
                Err(_err) => [5, 5, 5, 5, 5, 5, 5, 5], // nunca pasa
            };
            let timestamp = i64::from_be_bytes(bytes);
            let node_id = internal_metadata[8];
            return (Some(timestamp), Some(node_id));
        } else if internal_metadata.len() == 8 {
            let bytes: [u8; 8] = match internal_metadata[0..8].try_into() {
                Ok(value) => value,
                Err(_err) => [5, 5, 5, 5, 5, 5, 5, 5], // nunca pasa
            };
            let timestamp = i64::from_be_bytes(bytes);
            return (Some(timestamp), None);
        } else if internal_metadata.len() == 1 {
            let node_id = internal_metadata[0];
            return (None, Some(node_id));
        }
        (None, None)
    }

    fn get_partition_key_value_from_insert_statement(
        &self,
        insert: &Insert,
        table: &Table,
    ) -> Result<String> {
        let insert_columns = insert.get_columns_names();
        let position = match insert_columns
            .iter()
            .position(|col| col == &table.get_partition_key()[0])
        {
            Some(position) => position,
            None => {
                return Err(Error::SyntaxError(
                    "The partition key column must be in the request".to_string(),
                ))
            }
        };
        match insert.get_values().get(position) {
            Some(partition_value) => Ok(partition_value.to_string()),
            None => Err(Error::SyntaxError(
                "The partition key column must be in the request".to_string(),
            )),
        }
    }

    fn update_with_other_nodes(
        &mut self,
        update: Update,
        request: &[Byte],
        consistency_level: &Consistency,
    ) -> Result<Vec<Byte>> {
        let timestamp = Utc::now().timestamp();
        let table_name = update.table_name.get_name();
        let partitions_keys_to_nodes = self.get_partition_keys_values(&table_name)?.clone();
        let mut consulted_nodes: Vec<String> = Vec::new();
        let consistency_number = consistency_level.as_usize(N_NODES as usize);
        let mut consistency_counter = 0;
        let mut wait_response = true;
        for partition_key_value in partitions_keys_to_nodes {
            let node_id = self.select_node(&partition_key_value);
            if !consulted_nodes.contains(&partition_key_value) {
                let current_response = if node_id == self.id {
                    self.process_update(&update, timestamp, self.id)?
                } else {
                    let request_with_metadata = add_metadata_to_internal_request_of_any_kind(
                        SvAction::InternalQuery(request.to_vec()).as_bytes(),
                        Some(timestamp),
                        Some(node_id),
                    );
                    self.send_message_and_wait_response_with_timeout(
                        request_with_metadata,
                        node_id,
                        PortType::Priv,
                        wait_response,
                        TIMEOUT_SECS,
                    )?
                };

                consulted_nodes.push(partition_key_value.clone());
                let replication_factor = self.get_replicas_from_table_name(&table_name)?;
                self.replicate_update_in_other_nodes(
                    replication_factor,
                    node_id,
                    request,
                    &update,
                    timestamp,
                )?;

                if consistency_counter >= consistency_number {
                    wait_response = false;
                } else if verify_succesful_response(&current_response) {
                    consistency_counter += 1;
                }
            }
        }

        if consistency_counter < consistency_number {
            Err(Error::ServerError(format!(
                "No se pudo cumplir con el nivel de consistencia {}, solo se logró con {} de {}",
                consistency_level, consistency_counter, consistency_number,
            )))
        } else {
            Ok(Self::create_result_void())
        }
    }

    fn replicate_update_in_other_nodes(
        &mut self,
        replication_factor: u32,
        node_id: Byte,
        request: &[Byte],
        update: &Update,
        timestamp: i64,
    ) -> Result<()> {
        let nodes_ids = Self::get_nodes_ids();
        let mut node_to_replicate = node_id;
        for _ in 1..replication_factor {
            if node_to_replicate == self.id {
                self.process_update(update, timestamp, node_id)?;
            } else {
                let request_with_metadata = add_metadata_to_internal_request_of_any_kind(
                    SvAction::InternalQuery(request.to_vec()).as_bytes(),
                    Some(timestamp),
                    Some(node_id),
                );
                let replica_response = self.send_message_and_wait_response_with_timeout(
                    request_with_metadata,
                    node_to_replicate,
                    PortType::Priv,
                    true,
                    TIMEOUT_SECS,
                )?;
                match Opcode::try_from(replica_response[4])? {
                    Opcode::RequestError => {
                        return Err(Error::try_from(replica_response[9..].to_vec())?)
                    }
                    Opcode::Result => (),
                    _ => {
                        return Err(Error::ServerError(
                            "Nodo de réplica manda opcode inesperado".to_string(),
                        ))
                    }
                }
            }
            node_to_replicate = next_node_in_the_cluster(node_to_replicate, &nodes_ids);

            // if consistency_counter >= consistency_number {
            //     wait_response = false;
            // } else if verify_succesful_response(&current_response) {
            //     consistency_counter += 1;
            // }
        }
        Ok(())
    }

    fn select_with_other_nodes(
        &mut self,
        select: Select,
        request: &[Byte],
        consistency_level: &Consistency,
    ) -> Result<Vec<Byte>> {
        let table_name = select.from.get_name();
        let mut results_from_another_nodes: Vec<Byte> = Vec::new();
        let partitions_keys_to_nodes = self.get_partition_keys_values(&table_name)?.clone(); // Tuve que agregar un clone para que no me tire error de referencia mutable e inmutable al mismo tiempo
        let mut consulted_nodes: Vec<Byte> = Vec::new();
        let replication_factor_quantity = self.get_replicas_from_table_name(&table_name)?;
        let consistency_number = consistency_level.as_usize(replication_factor_quantity as usize);
        for partition_key_value in partitions_keys_to_nodes {
            let node_id = self.select_node(&partition_key_value);
            if !consulted_nodes.contains(&node_id) {
                let wait_response = true;
                let mut read_repair_executed = false;
                let mut consistency_counter = 0;
                let mut responsive_replica = node_id;
                let mut replicas_asked = 0;
                let mut actual_result = self.decide_how_to_request_internal_query_select(
                    node_id,
                    (&select, request),
                    wait_response,
                    &mut responsive_replica,
                    &mut replicas_asked,
                    replication_factor_quantity,
                )?;
                consistency_counter += 1;
                match self.consult_replica_nodes(
                    (node_id, replicas_asked),
                    (request, &table_name),
                    &mut consistency_counter,
                    consistency_number,
                    (responsive_replica, &actual_result),
                    replication_factor_quantity,
                ) {
                    Ok(rr_executed) => {
                        // Este chequeo es porque si ya es true, no queremos que vuelva a ser false
                        // Nos importa si se ejecutó al menos una vez
                        if !read_repair_executed {
                            read_repair_executed = rr_executed;
                        }
                    }
                    Err(err) => return Err(Error::ServerError(format!(
                        "No se pudo cumplir con el nivel de consistencia {}, solo se logró con {} de {}: {}",
                        consistency_level, consistency_counter, consistency_number, err,
                    ))),
                }
                // Una vez que todo fue reparado, queremos reenviar la query para obtener el resultado
                // pero ahora con las tablas reparadas.
                if read_repair_executed {
                    actual_result = self.decide_how_to_request_internal_query_select(
                        node_id,
                        (&select, request),
                        wait_response,
                        &mut responsive_replica,
                        &mut replicas_asked,
                        replication_factor_quantity,
                    )?;
                };
                self.handle_result_from_node(
                    &mut results_from_another_nodes,
                    &actual_result,
                    &select,
                )?;
                consulted_nodes.push(node_id);
            }
        }
        Ok(results_from_another_nodes)
    }

    fn decide_how_to_request_internal_query_select(
        &mut self,
        node_id: NodeId,
        select_and_request: (&Select, &[Byte]),
        wait_response: bool,
        responsive_replica: &mut NodeId,
        replicas_asked: &mut usize,
        replication_factor_quantity: u32,
    ) -> Result<Vec<Byte>> {
        let (select, request) = select_and_request;
        let actual_result = if node_id == self.id {
            self.process_select(select, node_id)?
        } else {
            let request_with_metadata = add_metadata_to_internal_request_of_any_kind(
                SvAction::InternalQuery(request.to_vec()).as_bytes(),
                None,
                Some(node_id),
            );
            let mut result: Vec<u8> = Vec::new();
            *responsive_replica = node_id;
            *replicas_asked = 0;
            if self.neighbour_is_responsive(node_id) {
                result = self
                    .send_message_and_wait_response_with_timeout(
                        request_with_metadata,
                        node_id,
                        PortType::Priv,
                        wait_response,
                        TIMEOUT_SECS,
                    )
                    .unwrap_or_default()
            }
            *replicas_asked += 1;

            // Si hubo error al enviar el mensaje, se asume que el vecino está apagado,
            // entonces se intenta con las replicas
            if result.is_empty() {
                self.acknowledge_offline_neighbour(node_id);
                result = self.forward_request_to_replicas(
                    node_id,
                    (select, request),
                    wait_response,
                    responsive_replica,
                    replicas_asked,
                    replication_factor_quantity,
                )?;
            }
            result
        };
        Ok(actual_result)
    }

    fn forward_request_to_replicas(
        &mut self,
        node_id: NodeId,
        select_and_request: (&Select, &[Byte]),
        wait_response: bool,
        responsive_replica: &mut NodeId,
        replicas_asked: &mut usize,
        replication_factor_quantity: u32,
    ) -> Result<Vec<Byte>> {
        let (select, request) = select_and_request;
        let mut result: Vec<u8> = Vec::new();
        let nodes_ids = Self::get_nodes_ids();
        let mut node_replica = next_node_in_the_cluster(node_id, &nodes_ids);

        for _ in 1..replication_factor_quantity {
            if self.neighbour_is_responsive(node_replica) {
                let request_with_metadata = add_metadata_to_internal_request_of_any_kind(
                    SvAction::InternalQuery(request.to_vec()).as_bytes(),
                    None,
                    Some(node_id),
                );
                let replica_response = if node_replica == self.id {
                    self.process_select(select, node_id)?
                } else {
                    self.send_message_and_wait_response_with_timeout(
                        request_with_metadata,
                        node_replica,
                        PortType::Priv,
                        wait_response,
                        TIMEOUT_SECS,
                    )?
                };
                *replicas_asked += 1;

                if replica_response.is_empty() {
                    self.acknowledge_offline_neighbour(node_replica);
                } else {
                    result = replica_response;
                    *responsive_replica = node_replica;
                    break;
                }
            } else {
                *replicas_asked += 1;
            }
            node_replica = next_node_in_the_cluster(node_replica, &nodes_ids);
        }

        Ok(result)
    }

    /// Revisa si se cumple el _Consistency Level_ y además si es necesario ejecutar _read-repair_, si es el caso, lo ejecuta.
    ///
    /// Devuelve un booleano indicando si _read-repair_ fue ejecutado o no.
    fn consult_replica_nodes(
        &mut self,
        id_and_replicas_asked: (NodeId, usize),
        request_and_table_name: (&[Byte], &str),
        consistency_counter: &mut usize,
        consistency_number: usize,
        first_responsive_id_and_response: (NodeId, &[Byte]),
        replication_factor_quantity: u32,
    ) -> Result<bool> {
        if consistency_number == 1 {
            return Ok(false);
        }
        let mut exec_read_repair = false;
        let (node_id, replicas_asked) = id_and_replicas_asked;
        let (request, table_name) = request_and_table_name;
        let (responsive_replica, response_from_first_responsive_replica) =
            first_responsive_id_and_response;

        let first_hashed_value = hash_value(response_from_first_responsive_replica);
        let mut responses: Vec<Vec<Byte>> = Vec::new();
        let nodes_ids = Self::get_nodes_ids();
        let mut node_to_consult = next_node_in_the_cluster(responsive_replica, &nodes_ids);
        for _ in (replicas_asked as u32)..replication_factor_quantity {
            let opcode_with_hashed_value = self
                .decide_how_to_request_the_digest_read_request(node_to_consult, request, node_id)
                .unwrap_or_default();
            if opcode_with_hashed_value.is_empty() {
                node_to_consult = next_node_in_the_cluster(node_to_consult, &nodes_ids);
                continue;
            }
            let res_hashed_value = self.get_digest_read_request_value(&opcode_with_hashed_value)?;
            check_consistency_of_the_responses(
                opcode_with_hashed_value,
                first_hashed_value,
                res_hashed_value,
                consistency_counter,
                &mut responses,
            )?;
            if *consistency_counter >= consistency_number {
                break;
            }
            node_to_consult = next_node_in_the_cluster(node_to_consult, &nodes_ids);
        }
        check_if_read_repair_is_neccesary(
            consistency_counter,
            consistency_number,
            &mut exec_read_repair,
            responses,
            first_hashed_value,
        );
        if exec_read_repair && self.neighbour_is_responsive(node_id) {
            return self.exec_read_repair(node_id, request, consistency_number, table_name);
        };
        Ok(false)
    }

    fn decide_how_to_request_the_digest_read_request(
        &mut self,
        node_to_consult: u8,
        request: &[u8],
        node_id: u8,
    ) -> Result<Vec<u8>> {
        let opcode_with_hashed_value = if node_to_consult == self.id {
            let internal_request =
                add_metadata_to_internal_request_of_any_kind(request.to_vec(), None, Some(node_id));
            let res = self.handle_request(&internal_request, true, true);
            let mut res_with_opcode;
            if verify_succesful_response(&res) {
                res_with_opcode = Opcode::Result.as_bytes();
                res_with_opcode.extend_from_slice(&hash_value(&res).to_be_bytes());
            } else {
                res_with_opcode = Opcode::RequestError.as_bytes();
                res_with_opcode.extend_from_slice(&res);
            }
            res_with_opcode
        } else {
            let request_with_metadata = add_metadata_to_internal_request_of_any_kind(
                SvAction::DigestReadRequest(request.to_vec()).as_bytes(),
                None,
                Some(node_id),
            );
            self.send_message_and_wait_response_with_timeout(
                request_with_metadata,
                node_to_consult,
                PortType::Priv,
                true,
                TIMEOUT_SECS,
            )?
        };
        Ok(opcode_with_hashed_value)
    }

    fn exec_read_repair(
        &self,
        node_id: u8,
        request: &[Byte],
        consistency_number: usize,
        table_name: &str,
    ) -> Result<bool> {
        let mut ids_and_rows: Vec<(NodeId, Vec<Vec<String>>)> = vec![];
        let mut req_with_node_replica = request[9..].to_vec();
        req_with_node_replica.push(node_id);
        let nodes_ids = Self::get_nodes_ids();
        let mut node_to_consult = node_id;
        for _ in 0..consistency_number {
            let res = if node_to_consult == self.id {
                self.exec_direct_read_request(req_with_node_replica.clone())?
            } else {
                let extern_response = self.send_message_and_wait_response_with_timeout(
                    SvAction::DirectReadRequest(req_with_node_replica.clone()).as_bytes(),
                    node_to_consult,
                    PortType::Priv,
                    true,
                    TIMEOUT_SECS,
                )?;
                create_utf8_string_from_bytes(extern_response)?
            };
            add_rows_with_his_node(res, &mut ids_and_rows, node_to_consult);
            node_to_consult = next_node_in_the_cluster(node_to_consult, &nodes_ids);
        }
        let rows_as_string = get_most_recent_rows_as_string(ids_and_rows);
        let mut node_to_repair = node_id;
        for _ in 0..consistency_number {
            if node_to_repair == self.id {
                let table = self.get_table(table_name)?;
                DiskHandler::repair_rows(
                    &self.storage_addr,
                    table_name,
                    table.get_keyspace(),
                    &self.default_keyspace_name,
                    node_to_repair,
                    &rows_as_string,
                )?;
            } else {
                let sv_action = SvAction::RepairRows(
                    table_name.to_string(),
                    node_id,
                    rows_as_string.as_bytes().to_vec(),
                )
                .as_bytes();
                self.send_message_and_wait_response_with_timeout(
                    sv_action,
                    node_to_repair,
                    PortType::Priv,
                    false,
                    TIMEOUT_SECS,
                )?;
            };
            node_to_repair = next_node_in_the_cluster(node_to_repair, &nodes_ids);
        }
        Ok(true)
    }

    fn process_delete(&mut self, delete: &Delete, node_number: Byte) -> Result<Vec<Byte>> {
        let table = match self.get_table(&delete.from.get_name()) {
            Ok(table) => table,
            Err(err) => return Err(err),
        };

        DiskHandler::do_delete(
            delete,
            &self.storage_addr,
            table,
            &self.get_default_keyspace_name()?,
            node_number,
        )?;

        Ok(Self::create_result_void())
    }

    fn delete_with_other_nodes(
        &mut self,
        delete: Delete,
        request: &[Byte],
        consistency_level: &Consistency,
    ) -> Result<Vec<Byte>> {
        let table_name = delete.from.get_name();
        let partitions_keys_to_nodes = self.get_partition_keys_values(&table_name)?.clone();
        let mut consulted_nodes: Vec<String> = Vec::new();
        let consistency_number = consistency_level.as_usize(N_NODES as usize);
        for partition_key_value in partitions_keys_to_nodes {
            let node_id = self.select_node(&partition_key_value);
            if !consulted_nodes.contains(&partition_key_value) {
                consulted_nodes.push(partition_key_value.clone());
                let replication_factor = self.get_replicas_from_table_name(&table_name)?;
                self.replicate_delete_in_other_nodes(
                    replication_factor,
                    node_id,
                    request,
                    &delete,
                    consistency_number,
                )?;
            }
        }
        Ok(Self::create_result_void())
    }

    // Función auxiliar para replicar el delete en otros nodos
    fn replicate_delete_in_other_nodes(
        &mut self,
        replication_factor: u32,
        node_id: Byte,
        request: &[Byte],
        delete: &Delete,
        consistency_number: usize,
    ) -> Result<()> {
        let mut consistency_counter = 0;
        let mut wait_response = true;
        let nodes_ids = Self::get_nodes_ids();
        let mut node_to_replicate = node_id;
        for _ in 0..replication_factor {
            let current_response = if node_to_replicate == self.id {
                self.process_delete(delete, node_id)?
            } else {
                let request_with_metadata = add_metadata_to_internal_request_of_any_kind(
                    SvAction::InternalQuery(request.to_vec()).as_bytes(),
                    None,
                    Some(node_id),
                );
                self.send_message_and_wait_response_with_timeout(
                    request_with_metadata,
                    node_to_replicate,
                    PortType::Priv,
                    wait_response,
                    TIMEOUT_SECS,
                )?
            };
            if consistency_counter >= consistency_number {
                wait_response = false;
            } else if verify_succesful_response(&current_response) {
                consistency_counter += 1;
            }
            node_to_replicate = next_node_in_the_cluster(node_to_replicate, &nodes_ids);
        }
        if consistency_counter < consistency_number {
            return Err(Error::ServerError(format!(
                "No se pudo cumplir con el nivel de consistencia, solo se logró con {} de {}",
                consistency_counter, consistency_number,
            )));
        };
        Ok(())
    }

    fn get_partition_keys_values(&self, table_name: &String) -> Result<&Vec<String>> {
        match self.tables_and_partitions_keys_values.get(table_name) {
            Some(partitions_keys_to_nodes) => Ok(partitions_keys_to_nodes),
            None => Err(Error::ServerError(
                "La tabla indicada no existe".to_string(),
            )),
        }
    }

    fn get_replicas_from_table_name(&self, table_name: &str) -> Result<u32> {
        let keyspace = self.get_keyspace(table_name)?;
        match keyspace.simple_replicas() {
            Some(replication_factor) => Ok(replication_factor),
            None => Err(Error::ServerError("No es una simple strategy".to_string())),
        }
    }

    fn handle_result_from_node(
        &self,
        results_from_another_nodes: &mut Vec<Byte>,
        result_from_actual_node: &[Byte],
        _select: &Select,
    ) -> Result<()> {
        let mut res = result_from_actual_node.to_vec();
        if res.is_empty() {
            return Ok(());
        }
        if results_from_another_nodes.is_empty() {
            results_from_another_nodes.append(&mut res);
            return Ok(());
        }
        let total_length_until_end_of_metadata = self.get_columns_metadata_length(&res);
        let total_lenght_until_rows_content = total_length_until_end_of_metadata + 4;
        let mut quantity_rows = self.get_quantity_of_rows(
            results_from_another_nodes,
            total_length_until_end_of_metadata,
        );
        let new_quantity_rows =
            self.get_quantity_of_rows(result_from_actual_node, total_length_until_end_of_metadata);
        quantity_rows += new_quantity_rows;
        results_from_another_nodes
            [total_length_until_end_of_metadata..total_lenght_until_rows_content]
            .copy_from_slice(&quantity_rows.to_be_bytes());

        let mut new_res = result_from_actual_node[total_lenght_until_rows_content..].to_vec();
        results_from_another_nodes.append(&mut new_res);

        let final_length = (results_from_another_nodes.len() as u32) - 9;
        results_from_another_nodes[5..9].copy_from_slice(&final_length.to_be_bytes());
        Ok(())
    }

    fn get_quantity_of_rows(
        &self,
        results_from_another_nodes: &[Byte],
        rows_quantity_position: usize,
    ) -> i32 {
        let new_quantity_rows =
            &results_from_another_nodes[rows_quantity_position..(rows_quantity_position + 4)];
        i32::from_be_bytes([
            new_quantity_rows[0],
            new_quantity_rows[1],
            new_quantity_rows[2],
            new_quantity_rows[3],
        ])
    }

    fn get_columns_metadata_length(&self, results_from_another_nodes: &[Byte]) -> usize {
        let mut total_length_from_metadata: usize = 21;
        if results_from_another_nodes.len() < total_length_from_metadata {
            return 0;
        }
        // el 13 al 17 son flags
        let column_quantity = &results_from_another_nodes[17..21];
        let column_quantity = i32::from_be_bytes([
            column_quantity[0],
            column_quantity[1],
            column_quantity[2],
            column_quantity[3],
        ]);
        for _ in 0..column_quantity {
            let name_length = &results_from_another_nodes
                [total_length_from_metadata..(total_length_from_metadata + 2)]; // Consigo el largo del [String]
            let name_length = u16::from_be_bytes([name_length[0], name_length[1]]); // Lo casteo para sumarlo al total
            total_length_from_metadata += (name_length as usize) + 2 + 2; // Esto es [String] + [Option]
        }
        total_length_from_metadata
    }

    fn _get_ordered_new_res_bytes(
        &self,
        results_from_another_nodes: &[Byte],
        total_length_from_metadata: usize,
        select: &Select,
    ) -> Result<Vec<Byte>> {
        let table_name = select.from.get_name();
        let table = self.get_table(&table_name)?;

        let result_string =
            String::from_utf8(results_from_another_nodes[total_length_from_metadata..].to_vec())
                .map_err(|e| Error::ServerError(e.to_string()))?;

        let rows: Vec<&str> = result_string.split("\n").collect();
        let mut splitted_rows: Vec<Vec<String>> = rows
            .iter()
            .map(|r| r.split(",").map(|s| s.to_string()).collect())
            .collect();
        if let Some(order_by) = &select.options.order_by {
            order_by.order(&mut splitted_rows, &table.get_columns_names());
        }

        let new_ordered_res = splitted_rows
            .iter()
            .map(|r| r.join(","))
            .collect::<Vec<String>>()
            .join("\n");

        Ok(new_ordered_res.as_bytes().to_vec())
    }

    fn get_quantity_of_replicas_from_keyspace(&self, keyspace: &Keyspace) -> Result<u32> {
        let replicas = match keyspace.simple_replicas() {
            Some(value) => value,
            None => {
                return Err(Error::ServerError(
                    "No se usa una estrategia de replicacion simple".to_string(),
                ))
            }
        };
        Ok(replicas)
    }

    fn get_digest_read_request_value(&self, opcode_with_hashed_value: &[Byte]) -> Result<u64> {
        if opcode_with_hashed_value.len() != 9 {
            // OpCode + i64
            return Err(Error::ServerError(
                "Se esperaba un vec de largo 9".to_string(),
            ));
        }
        let array = match opcode_with_hashed_value[1..9].try_into().ok() {
            Some(value) => value,
            None => {
                return Err(Error::ServerError(
                    "No se pudo transformar el vector a i64".to_string(),
                ))
            }
        };
        let res_hashed_value = u64::from_be_bytes(array);
        Ok(res_hashed_value)
    }

    fn match_kind_of_conection_mode<S>(
        &mut self,
        bytes: Vec<Byte>,
        mut stream: S,
        is_logged: bool,
    ) -> Result<Vec<Byte>>
    where
        S: Read + Write,
    {
        match self.mode() {
            ConnectionMode::Echo => {
                let printable_bytes = bytes
                    .iter()
                    .map(|b| format!("{:#X}", b))
                    .collect::<Vec<String>>();
                println!("[{} - ECHO] {}", self.id, printable_bytes.join(" "));
                if let Err(err) = stream.write_all(&bytes) {
                    println!("Error al escribir en el TCPStream:\n\n{}", err);
                }
                if let Err(err) = stream.flush() {
                    println!("Error haciendo flush desde el nodo:\n\n{}", err);
                }
            }
            ConnectionMode::Parsing => {
                let res = self.handle_request(&bytes[..], false, is_logged);
                let _ = stream.write_all(&res[..]);
                if let Err(err) = stream.flush() {
                    println!("Error haciendo flush desde el nodo:\n\n{}", err);
                }
                return Ok(res);
            }
        }
        Ok(vec![])
    }

    /// Espera a que terminen todos los handlers.
    ///
    /// Esto idealmente sólo debería llamarse una vez, ya que consume los handlers y además
    /// bloquea el hilo actual.
    fn wait(mut handlers: Vec<Option<NodeHandle>>) {
        // long live the option dance
        for handler_opt in &mut handlers {
            if let Some(handler) = handler_opt.take() {
                if handler.join().is_err() {
                    // Un hilo caído NO debería interrumpir el dropping de los demás
                    println!("Ocurrió un error mientras se esperaba a que termine un hilo hijo.");
                }
            }
        }
    }

    //     fn check_if_response_is_error(&self, res: &[Byte]) -> Result<Vec<Byte>>{
    //         match Opcode::try_from(res[4])? {
    //             Opcode::RequestError => return Err(Error::try_from(res[9..].to_vec())?),
    //             Opcode::Result => self.handle_result_from_node(
    //                 &mut results_from_another_nodes,
    //                 res,
    //                 &select,
    //             )?,
    //             _ => {
    //                 return Err(Error::ServerError(
    //                     "Nodo manda opcode inesperado".to_string(),
    //                 ))
    //             }
    //         };
    //     }
}

fn wrap_header(mut response: Vec<Byte>, is_internal_request: bool, header: Headers) -> Vec<Byte> {
    if response.is_empty() {
        response.append(&mut Node::create_result_void())
    }
    if !is_internal_request {
        let ver = Version::ResponseV5.as_bytes();
        let stream = header.stream.as_bytes();
        response.splice(0..1, ver);
        response.splice(2..4, stream);
    }
    response
}

fn make_error_response(err: Error) -> Vec<Byte> {
    let mut response: Vec<Byte> = Vec::new();
    let mut bytes_err = err.as_bytes();
    response.append(&mut Version::ResponseV5.as_bytes());
    response.append(&mut Flag::Default.as_bytes());
    response.append(&mut Stream::new(0).as_bytes());
    response.append(&mut Opcode::RequestError.as_bytes());
    response.append(&mut Length::new(bytes_err.len() as u32).as_bytes());
    response.append(&mut bytes_err);
    response
}

fn check_consistency_of_the_responses(
    opcode_with_hashed_value: Vec<u8>,
    first_hashed_value: u64,
    res_hashed_value: u64,
    consistency_counter: &mut usize,
    responses: &mut Vec<Vec<u8>>,
) -> Result<()> {
    if Opcode::try_from(opcode_with_hashed_value[0])? == Opcode::Result
        && first_hashed_value == res_hashed_value
    {
        *consistency_counter += 1;
        responses.push(opcode_with_hashed_value[1..].to_vec());
    };
    Ok(())
}

fn check_if_read_repair_is_neccesary(
    consistency_counter: &mut usize,
    consistency_number: usize,
    exec_read_repair: &mut bool,
    responses: Vec<Vec<u8>>,
    first_hashed_value: u64,
) {
    if *consistency_counter < consistency_number {
        *exec_read_repair = true
    }

    for hashed_value_vec in responses {
        if hashed_value_vec.len() < 8 {
            *exec_read_repair = true;
        }
        let mut array = [0u8; 8]; // 8 es el len del hashed_value
        array.copy_from_slice(&hashed_value_vec[0..8]);
        let hashed_value_of_response = u64::from_be_bytes(array);
        if first_hashed_value != hashed_value_of_response {
            *exec_read_repair = true;
        }
    }
}

fn add_rows_with_his_node(
    res: String,
    ids_and_rows: &mut Vec<(u8, Vec<Vec<String>>)>,
    node_to_consult: u8,
) {
    let rows: Vec<Vec<String>> = res
        .split("\n")
        .map(|row| row.split(",").map(|col| col.to_string()).collect())
        .collect();
    ids_and_rows.push((node_to_consult, rows));
}

fn create_utf8_string_from_bytes(extern_response: Vec<u8>) -> Result<String> {
    Ok(match String::from_utf8(extern_response) {
        Ok(value) => value,
        Err(_err) => {
            return Err(Error::ServerError(
                "Error al castear de vector a string".to_string(),
            ))
        }
    })
}

fn get_most_recent_rows_as_string(ids_and_rows: Vec<(u8, Vec<Vec<String>>)>) -> String {
    let mut most_recent_timestamps: Vec<(usize, String)> = Vec::new();
    let mut newer_rows: Vec<Vec<String>> = Vec::new();

    for (i, (_node, rows)) in ids_and_rows.iter().enumerate() {
        for (j, row) in rows.iter().enumerate() {
            if most_recent_timestamps.len() <= j {
                most_recent_timestamps.push((i, row[row.len() - 1].clone()));
            } else {
                let actual_timestamp = row[row.len() - 1].clone();
                if actual_timestamp > most_recent_timestamps[j].1 {
                    most_recent_timestamps[j] = (i, actual_timestamp);
                }
            }
        }
    }
    for (i, actual_timestamp) in most_recent_timestamps.iter().enumerate() {
        let new_row = &ids_and_rows[actual_timestamp.0].1[i];
        newer_rows.push(new_row.clone());
    }
    let rows_as_string = newer_rows
        .iter()
        .map(|row| row.join(","))
        .collect::<Vec<String>>()
        .join("\n");
    rows_as_string
}

/// Agrega metadata, como el timestamp o el node_id si es necesario, sino no agrega estos campos.
fn add_metadata_to_internal_request_of_any_kind(
    mut sv_action_with_request: Vec<Byte>,
    timestamp: Option<i64>,
    node_id: Option<Byte>,
) -> Vec<Byte> {
    let mut metadata: Vec<Byte> = Vec::new();
    if let Some(value) = timestamp {
        metadata.append(&mut value.to_be_bytes().to_vec())
    };
    if let Some(value) = node_id {
        metadata.push(value)
    };
    sv_action_with_request.append(&mut metadata);
    sv_action_with_request
}

fn verify_succesful_response(response: &[Byte]) -> bool {
    if response.len() < 9 {
        return false;
    };
    let opcode = match Opcode::try_from(response[4]) {
        Ok(opcode) => opcode,
        Err(_err) => Opcode::RequestError,
    };
    match opcode {
        Opcode::Result => true, // Si la response tiene el opcode Result entonces es valida
        _ => false,
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.endpoint_state.eq(&other.endpoint_state)
    }
}
