//! Módulo de nodos.
use chrono::Utc;
use std::{
    cmp::PartialEq,
    collections::{HashMap, HashSet},
    fmt,
    io::{BufRead, BufReader, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    path::Path,
    sync::{Arc, Mutex},
    vec::IntoIter,
};

use crate::parser::{
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
                delete::Delete, insert::Insert, select::select_operation::Select, update::Update,
            },
        },
        statement::Statement,
    },
};
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
    traits::Serializable,
};
use crate::tokenizer::tokenizer::tokenize_query;
use crate::{client::cql_frame::query_body::QueryBody, server::pool::threadpool::ThreadPool};

use super::{
    addr::loader::AddrLoader,
    disk_operations::disk_handler::DiskHandler,
    graph::{LAST_ID, N_NODES, START_ID},
    keyspace_metadata::keyspace::Keyspace,
    port_type::PortType,
    states::{
        appstatus::AppStatus,
        endpoints::EndpointState,
        heartbeat::{
            HeartbeatState, {GenType, VerType},
        },
    },
    table_metadata::table::Table,
    utils::{
        divide_range, hash_value, hashmap_to_string, hashmap_vec_to_string, next_node_in_the_round,
        send_to_node, send_to_node_and_wait_response, send_to_node_and_wait_response_with_timeout,
        string_to_hashmap, string_to_hashmap_vec,
    },
};

/// El ID de un nodo. No se tienen en cuenta casos de cientos de nodos simultáneos,
/// así que un byte debería bastar para representarlo.
pub type NodeId = Byte;
/// Mapea todos los estados de los vecinos y de sí mismo.
pub type NodesMap = HashMap<NodeId, EndpointState>;
/// Mapea todas las conexiones actualmente abiertas.
pub type OpenConnectionsMap = HashMap<Stream, TcpStream>;

/// El límite posible para los rangos de los nodos.
const NODES_RANGE_END: u64 = 18446744073709551615;
/// El número de hilos para el [ThreadPool].
const N_THREADS: usize = 6;
/// El tiempo de espera _(en segundos)_ por una respuesta.
const TIMEOUT_SECS: u64 = 2;

/// Un nodo es una instancia de parser que se conecta con otros nodos para procesar _queries_.
pub struct Node {
    /// El ID del nodo mismo.
    id: NodeId,

    /// Los estados de los nodos vecinos, incluyendo este mismo.
    ///
    /// No necesariamente serán todos los otros nodos del grafo, sólo los que este nodo conoce.
    neighbours_states: NodesMap,

    /// Estado actual del nodo.
    endpoint_state: EndpointState,

    /// Dirección de almacenamiento en disco.
    storage_addr: String,

    /// Nombre del keyspace por defecto.
    default_keyspace_name: String,

    /// Los keyspaces que tiene el nodo.
    /// (nombre, keyspace)
    keyspaces: HashMap<String, Keyspace>,

    /// Las tablas que tiene el nodo.
    /// (nombre, tabla)
    tables: HashMap<String, Table>,

    /// Rangos asignados a cada nodo para determinar la partición de los datos.
    nodes_ranges: Vec<(u64, u64)>,

    /// Nombre de la tabla y los valores de las _partitions keys_ que contiene
    tables_and_partitions_keys_values: HashMap<String, Vec<String>>,

    /// El [ThreadPool] de tareas disponibles.
    pub pool: ThreadPool,

    /// Mapa de conexiones abiertas entre el nodo y otros clientes.
    open_connections: OpenConnectionsMap,
}

impl Node {
    /// Crea un nuevo nodo.
    pub fn new(id: NodeId, mode: ConnectionMode) -> Result<Self> {
        let storage_addr = DiskHandler::new_node_storage(id)?;
        let nodes_ranges = divide_range(0, NODES_RANGE_END, N_NODES as usize);
        let mut neighbours_states = NodesMap::new();
        let endpoint_state = EndpointState::with_id_and_mode(id, mode);
        neighbours_states.insert(id, endpoint_state.clone());

        Ok(Self {
            id,
            neighbours_states,
            endpoint_state,
            storage_addr,
            default_keyspace_name: "".to_string(),
            keyspaces: HashMap::new(),
            tables: HashMap::new(),
            nodes_ranges,
            tables_and_partitions_keys_values: HashMap::new(),
            pool: ThreadPool::build(N_THREADS).unwrap(),
            open_connections: OpenConnectionsMap::new(),
        })
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

    /// Consulta los IDs de los vecinos, incluyendo el propio.
    fn get_neighbours_ids(&self) -> Vec<NodeId> {
        self.neighbours_states.keys().copied().collect()
    }

    /// Selecciona un ID de nodo conforme al _hashing_ del valor del _partition key_ y los rangos de los nodos.
    fn select_node(&self, value: &str) -> NodeId {
        let hash_val = hash_value(value);

        let mut i = 0;
        for (a, b) in &self.nodes_ranges {
            if *a <= hash_val && hash_val < *b {
                return START_ID + i as NodeId;
            }
            i += 1;
        }
        START_ID + (i) as NodeId
    }

    /// Manda un mensaje a un nodo específico y espera por la respuesta de este.
    fn _send_message_and_wait_response(
        &self,
        bytes: Vec<Byte>,
        node_id: Byte,
        port_type: PortType,
        wait_response: bool,
    ) -> Result<Vec<Byte>> {
        send_to_node_and_wait_response(node_id, bytes, port_type, wait_response)
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

    /// Manda un mensaje en bytes al nodo correspondiente mediante el _hashing_ del valor del _partition key_.
    fn _send_message(
        &mut self,
        bytes: Vec<Byte>,
        value: String,
        port_type: PortType,
    ) -> Result<()> {
        send_to_node(self.select_node(&value), bytes, port_type)
    }

    /// Manda un mensaje en bytes a todos los vecinos del nodo.
    pub fn notice_all_neighbours(&self, bytes: Vec<Byte>, port_type: PortType) -> Result<()> {
        for neighbour_id in self.get_neighbours_ids() {
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

    /// Verifica rápidamente si un mensaje es de tipo [EXIT](SvAction::Exit).
    fn is_exit(bytes: &[Byte]) -> bool {
        if let Some(action) = SvAction::get_action(bytes) {
            if matches!(action, SvAction::Exit) {
                return true;
            }
        }
        false
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

    /// Consigue la información de _gossip_ que contiene este nodo.
    fn get_gossip_info(&self) -> GossipInfo {
        let mut gossip_info = GossipInfo::new();
        for (node_id, endpoint_state) in &self.neighbours_states {
            gossip_info.insert(node_id.to_owned(), endpoint_state.clone_heartbeat());
        }

        gossip_info
    }

    /// Consulta el modo de conexión del nodo.
    fn mode(&self) -> &ConnectionMode {
        self.endpoint_state.get_appstate().get_mode()
    }

    /// Consulta si el nodo todavía esta booteando.
    pub fn is_bootstraping(&self) -> bool {
        matches!(
            self.endpoint_state.get_appstate().get_status(),
            AppStatus::Bootstrap
        )
    }

    /// Consulta si el nodo ya está listo para recibir _queries_. Si lo está, actualiza su estado.
    fn is_bootstrap_done(&mut self) {
        if self.neighbours_states.len() == N_NODES as usize {
            self.endpoint_state.set_appstate_status(AppStatus::Normal);
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

    /// Inicia un intercambio de _gossip_ con los vecinos dados.
    fn gossip(&mut self, neighbours: HashSet<NodeId>) -> Result<()> {
        self.is_bootstrap_done();

        for neighbour_id in neighbours {
            if let Err(err) = send_to_node(
                neighbour_id,
                SvAction::Syn(self.get_id().to_owned(), self.get_gossip_info()).as_bytes(),
                PortType::Priv,
            ) {
                println!(
                    "Ocurrió un error al mandar un mensaje SYN al nodo [{}]:\n\n{}",
                    neighbour_id, err
                );
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

    /// Escucha por los eventos que recibe del cliente.
    pub fn cli_listen(socket: SocketAddr, node: Arc<Mutex<Node>>) -> Result<()> {
        Self::listen(socket, PortType::Cli, node)
    }

    /// Escucha por los eventos que recibe de otros nodos o estructuras internas.
    pub fn priv_listen(socket: SocketAddr, node: Arc<Mutex<Node>>) -> Result<()> {
        Self::listen(socket, PortType::Priv, node)
    }

    /// El escuchador de verdad.
    ///
    /// Las otras funciones son wrappers para no repetir código.
    fn listen(socket: SocketAddr, port_type: PortType, node: Arc<Mutex<Node>>) -> Result<()> {
        let listener = Node::bind_with_socket(socket)?;
        let addr_loader = AddrLoader::default_loaded();
        for tcp_stream_res in listener.incoming() {
            match tcp_stream_res {
                Err(_) => return Node::tcp_stream_error(&port_type, &socket, &addr_loader),
                Ok(tcp_stream) => {
                    let buffered_stream = Node::clone_tcp_stream(&tcp_stream)?;
                    let mut bufreader = BufReader::new(buffered_stream);
                    let bytes_vec = Node::write_bytes_in_buffer(&mut bufreader)?;
                    // consumimos los bytes del stream para no mandarlos de vuelta en la response
                    bufreader.consume(bytes_vec.len());
                    if Self::is_exit(&bytes_vec[..]) {
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

    /// Procesa una _request_ en forma de [Byte]s.
    /// También devuelve un [bool] indicando si se debe parar el hilo.
    pub fn process_tcp(&mut self, tcp_stream: TcpStream, bytes: Vec<Byte>) -> Result<()> {
        if bytes.is_empty() {
            return Ok(());
        }
        match SvAction::get_action(&bytes[..]) {
            Some(action) => {
                if let Err(err) = self.handle_sv_action(action, tcp_stream) {
                    println!(
                        "[{} - ACTION] Error en la acción del servidor: {}",
                        self.id, err
                    );
                }
                Ok(())
            }
            None => self.match_kind_of_conection_mode(bytes, tcp_stream),
        }
    }

    /// Maneja una acción de servidor.
    fn handle_sv_action(&mut self, action: SvAction, mut tcp_stream: TcpStream) -> Result<bool> {
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
                let response = self.handle_request(&bytes, true);
                let _ = tcp_stream.write_all(&response[..]);
                if let Err(err) = tcp_stream.flush() {
                    return Err(Error::ServerError(err.to_string()));
                };
            }
            SvAction::StoreMetadata => {
                if let Err(e) = DiskHandler::store_node_metadata(self.id, &self.serialize()) {
                    return Err(Error::ServerError(format!(
                        "Error guardando metadata del nodo {}: {}",
                        &self.id, e
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
                let response = self.handle_request(&bytes, true);
                // Devolvemos además un opcode para poder saber si el resultado fue un error o no.
                if verify_succesful_response(&response) {
                    let mut res = Opcode::Result.as_bytes();
                    res.extend_from_slice(&hash_value(&response).to_be_bytes());
                    let _ = tcp_stream.write_all(&res);
                } else {
                    let mut res = Opcode::RequestError.as_bytes();
                    res.extend_from_slice(&response);
                    let _ = tcp_stream.write_all(&res);
                }
                if let Err(err) = tcp_stream.flush() {
                    return Err(Error::ServerError(err.to_string()));
                };
            }
            SvAction::RepairRows(table_name, node_id, rows_bytes) => {
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
                let rows = String::from_utf8(rows_bytes).map_err(|_| {
                    Error::ServerError("Error al castear de bytes a string".to_string())
                })?;
                DiskHandler::repair_rows(
                    &self.storage_addr,
                    &table_name,
                    keyspace_name,
                    &self.get_default_keyspace_name()?,
                    node_id,
                    &rows,
                )?;
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
        let node_number = bytes.pop().unwrap_or(0); // despues cambiar
        let copia: &[u8] = &bytes;
        let statement = match QueryBody::try_from(copia) {
            Ok(query_body) => match make_parse(&mut tokenize_query(query_body.get_query())) {
                Ok(statement) => statement,
                Err(_err) => {
                    return Err(Error::ServerError(
                        "No se cumplio el protocolo al hacer read-repair".to_string(),
                    ))
                }
            },
            Err(_err) => {
                return Err(Error::ServerError(
                    "No se cumplio el protocolo al hacer read-repair".to_string(),
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

    /// Maneja una request.
    fn handle_request(&mut self, request: &[Byte], internal_request: bool) -> Vec<Byte> {
        let header = match Headers::try_from(&request[..9]) {
            Ok(header) => header,
            Err(err) => return self.make_error_response(err),
        };

        let left_response = match header.opcode {
            Opcode::Startup => self.handle_startup(request, header.length),
            Opcode::Options => self.handle_options(),
            Opcode::Query => self.handle_query(request, header.length, internal_request),
            Opcode::Prepare => self.handle_prepare(),
            Opcode::Execute => self.handle_execute(),
            Opcode::Register => self.handle_register(),
            Opcode::Batch => self.handle_batch(),
            Opcode::AuthResponse => self.handle_auth_response(),
            _ => Err(Error::ProtocolError(
                "El opcode recibido no es una request".to_string(),
            )),
        };
        match left_response {
            Ok(value) => value,
            Err(err) => self.make_error_response(err),
        }
    }

    fn make_error_response(&mut self, err: Error) -> Vec<Byte> {
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

    fn handle_startup(&self, _request: &[Byte], _length: Length) -> Result<Vec<Byte>> {
        // El body es un [string map] con posibles opciones
        Ok(vec![0])
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
        lenght: Length,
        internal_request: bool,
    ) -> Result<Vec<Byte>> {
        // if let Ok(query) = String::from_utf8(request[9..(lenght.len as usize) + 9].to_vec())
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

    fn handle_auth_response(&self) -> Result<Vec<Byte>> {
        Ok(vec![0])
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
        for actual_node in 0..N_NODES {
            let node_id = next_node_in_the_round(self.id, actual_node as Byte, START_ID, LAST_ID);
            response = if node_id != self.id {
                self.send_message_and_wait_response_with_timeout(
                    SvAction::InternalQuery(request.to_vec()).as_bytes(),
                    node_id,
                    PortType::Priv,
                    true,
                    TIMEOUT_SECS,
                )?
            } else {
                self.process_internal_use_statement(&keyspace_name)?
            }
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
            Ok(self.create_result_void())
        } else {
            if self.check_if_keyspace_exists(keyspace_name) {
                self.set_default_keyspace_name(name.clone())?;
                return Ok(self.create_result_void());
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
        for actual_node in 0..N_NODES {
            let node_id = next_node_in_the_round(self.id, actual_node as Byte, START_ID, LAST_ID);
            response = if node_id != self.id {
                self.send_message_and_wait_response_with_timeout(
                    SvAction::InternalQuery(request.to_vec()).as_bytes(),
                    node_id,
                    PortType::Priv,
                    true,
                    TIMEOUT_SECS,
                )?
            } else {
                self.process_internal_create_keyspace_statement(&create_keyspace)?
            }
        }
        Ok(response)
    }

    fn process_internal_create_keyspace_statement(
        &mut self,
        create_keyspace: &CreateKeyspace,
    ) -> Result<Vec<Byte>> {
        match DiskHandler::create_keyspace(create_keyspace, &self.storage_addr) {
            Ok(Some(keyspace)) => self.add_keyspace(keyspace),
            Ok(None) => return Ok(self.create_result_void()),
            Err(err) => return Err(err),
        };
        Ok(self.create_result_void())
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
        for actual_node in 0..N_NODES {
            let node_id = next_node_in_the_round(self.id, actual_node as Byte, START_ID, LAST_ID);

            let response = if node_id != self.id {
                self.send_message_and_wait_response_with_timeout(
                    SvAction::InternalQuery(request.to_vec()).as_bytes(),
                    node_id,
                    PortType::Priv,
                    true,
                    TIMEOUT_SECS,
                )?
            } else {
                self.process_internal_alter_keyspace_statement(&alter_keyspace)?
            };
            responses.push(response);
        }
        Ok(self.create_result_void())
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
                Ok(self.create_result_void())
            }
            None => {
                if alter_keyspace.if_exists {
                    Ok(self.create_result_void())
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
        for actual_node in 0..N_NODES {
            let node_id = next_node_in_the_round(self.id, actual_node as Byte, START_ID, LAST_ID);

            let response = if node_id != self.id {
                self.send_message_and_wait_response_with_timeout(
                    SvAction::InternalQuery(request.to_vec()).as_bytes(),
                    node_id,
                    PortType::Priv,
                    true,
                    TIMEOUT_SECS,
                )?
            } else {
                self.process_internal_drop_keyspace_statement(&drop_keyspace)?
            };
            responses.push(response);
        }
        Ok(self.create_result_void())
    }

    fn process_internal_drop_keyspace_statement(
        &mut self,
        drop_keyspace: &DropKeyspace,
    ) -> Result<Vec<Byte>> {
        let keyspace_name = drop_keyspace.name.get_name();
        if self.keyspaces.contains_key(keyspace_name) {
            self.keyspaces.remove(keyspace_name);
            match DiskHandler::drop_keyspace(keyspace_name, &self.storage_addr) {
                Ok(_) => Ok(self.create_result_void()),
                Err(e) => Err(e),
            }
        } else if drop_keyspace.if_exists {
            Ok(self.create_result_void())
        } else {
            Err(Error::ServerError(format!(
                "El keyspace {} no existe",
                keyspace_name
            )))
        }
    }

    fn create_result_void(&mut self) -> Vec<Byte> {
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
        Ok(self.create_result_void())
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
        for actual_node_id in START_ID..LAST_ID {
            for i in 0..quantity_replicas {
                let next_node_id =
                    next_node_in_the_round(actual_node_id, i as u8, START_ID, LAST_ID);
                response = if next_node_id == self.id {
                    self.process_internal_create_table_statement(&create_table, actual_node_id)?
                } else {
                    let request_with_metadata = add_metadata_to_internal_request_of_any_kind(
                        SvAction::InternalQuery(request.to_vec()).as_bytes(),
                        None,
                        Some(actual_node_id),
                    );
                    self.send_message_and_wait_response_with_timeout(
                        request_with_metadata,
                        next_node_id,
                        PortType::Priv,
                        true,
                        TIMEOUT_SECS,
                    )?
                }
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
        let mut response: Vec<Byte> = Vec::new();
        response.append(&mut Version::ResponseV5.as_bytes());
        response.append(&mut Flag::Default.as_bytes());
        response.append(&mut Stream::new(0).as_bytes());
        response.append(&mut Opcode::Result.as_bytes());
        response.append(&mut Length::new(res.len() as u32).as_bytes());
        response.append(&mut res);
        Ok(response)
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
        Ok(self.create_result_void())
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
        Ok(self.create_result_void())
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

        for i in 0..N_NODES {
            let node_to_replicate = next_node_in_the_round(node_id, i as Byte, START_ID, LAST_ID);
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
            Ok(self.create_result_void())
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
        for i in 1..replication_factor {
            let node_to_replicate = next_node_in_the_round(node_id, i as Byte, START_ID, LAST_ID);
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
                let mut replicas_asked = 0;

                let mut actual_result = self.decide_how_to_request_internal_query_select(
                    node_id,
                    &select,
                    request,
                    wait_response,
                    &mut replicas_asked,
                    replication_factor_quantity,
                )?;
                consistency_counter += 1;
                match self.consult_replica_nodes(
                    (node_id, replicas_asked),
                    request,
                    &mut consistency_counter,
                    consistency_number,
                    &actual_result,
                    &table_name,
                ) {
                    Ok(rr_executed) => {
                        // Este chequeo es porque si ya es true, no queremos que vuelva a ser false
                        // Nos importa si se ejecutó al menos una vez
                        if !read_repair_executed {
                            read_repair_executed = rr_executed;
                        }
                    }
                    Err(_) => return Err(Error::ServerError(format!(
                        "No se pudo cumplir con el nivel de consistencia {}, solo se logró con {} de {}",
                        consistency_level, consistency_counter, consistency_number,
                    ))),
                }
                // Una vez que todo fue reparado, queremos reenviar la query para obtener el resultado
                // pero ahora con las tablas reparadas.
                if read_repair_executed {
                    actual_result = self.decide_how_to_request_internal_query_select(
                        node_id,
                        &select,
                        request,
                        wait_response,
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
        node_id: u8,
        select: &Select,
        request: &[u8],
        wait_response: bool,
        replicas_asked: &mut usize,
        replication_factor_quantity: u32,
    ) -> Result<Vec<u8>> {
        let actual_result = if node_id == self.id {
            self.process_select(select, node_id)?
        } else {
            let request_with_metadata = add_metadata_to_internal_request_of_any_kind(
                SvAction::InternalQuery(request.to_vec()).as_bytes(),
                None,
                Some(node_id),
            );
            let mut result: Vec<u8> = Vec::new();
            if self.neighbour_is_responsive(node_id) {
                result = self.send_message_and_wait_response_with_timeout(
                    request_with_metadata,
                    node_id,
                    PortType::Priv,
                    wait_response,
                    TIMEOUT_SECS,
                )?;
            }
            *replicas_asked += 1;

            // Si hubo timeout al esperar la respuesta, se intenta con las replicas
            if result.is_empty() {
                self.acknowledge_offline_neighbour(node_id);
                for i in 1..replication_factor_quantity {
                    let node_to_replicate =
                        next_node_in_the_round(node_id, i as Byte, START_ID, LAST_ID);
                    if node_to_replicate != node_id {
                        if self.neighbour_is_responsive(node_to_replicate) {
                            let request_with_metadata =
                                add_metadata_to_internal_request_of_any_kind(
                                    SvAction::InternalQuery(request.to_vec()).as_bytes(),
                                    None,
                                    Some(node_id),
                                );
                            let replica_response = self
                                .send_message_and_wait_response_with_timeout(
                                    SvAction::InternalQuery(request_with_metadata).as_bytes(),
                                    node_to_replicate,
                                    PortType::Priv,
                                    wait_response,
                                    TIMEOUT_SECS,
                                )?;
                            *replicas_asked += 1;
                            if replica_response.is_empty() {
                                self.acknowledge_offline_neighbour(node_to_replicate);
                            } else {
                                result = replica_response;
                                break;
                            }
                        } else {
                            *replicas_asked += 1;
                        }
                    }
                }
            }
            result
        };
        Ok(actual_result)
    }

    /// Revisa si se cumple el _Consistency Level_ y además si es necesario ejecutar _read-repair_, si es el caso, lo ejecuta.
    ///
    /// Devuelve un booleano indicando si _read-repair_ fue ejecutado o no.
    fn consult_replica_nodes(
        &mut self,
        id_and_replicas_asked: (u8, usize),
        request: &[Byte],
        consistency_counter: &mut usize,
        consistency_number: usize,
        response_from_first_responsive_replica: &[Byte],
        table_name: &str,
    ) -> Result<bool> {
        let mut exec_read_repair = false;
        if consistency_number == 1 {
            return Ok(false);
        }
        let (node_id, replicas_asked) = id_and_replicas_asked;
        let first_hashed_value = hash_value(response_from_first_responsive_replica);
        let mut responses: Vec<Vec<Byte>> = Vec::new();
        for i in replicas_asked..consistency_number {
            let node_to_consult = next_node_in_the_round(node_id, i as u8, START_ID, LAST_ID);
            let opcode_with_hashed_value = self.decide_how_to_request_the_digest_read_request(
                node_to_consult,
                request,
                node_id,
            )?;
            let res_hashed_value = self.get_digest_read_request_value(&opcode_with_hashed_value)?;
            check_consistency_of_the_responses(
                opcode_with_hashed_value,
                first_hashed_value,
                res_hashed_value,
                consistency_counter,
                &mut responses,
            )?;
        }
        check_if_read_repair_is_neccesary(
            consistency_counter,
            consistency_number,
            &mut exec_read_repair,
            responses,
            first_hashed_value,
        );
        if exec_read_repair {
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
            let res = self.handle_request(&internal_request, true);
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
        for i in 0..consistency_number {
            let node_to_consult = next_node_in_the_round(node_id, i as u8, START_ID, LAST_ID);
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
        }
        let rows_as_string = get_most_recent_rows_as_string(ids_and_rows);
        for i in 0..consistency_number {
            let node_to_repair = next_node_in_the_round(node_id, i as u8, START_ID, LAST_ID);
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

        Ok(self.create_result_void())
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
        Ok(self.create_result_void())
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
        for i in 0..replication_factor {
            let node_to_replicate = next_node_in_the_round(node_id, i as Byte, START_ID, LAST_ID);
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

        // No funciona, las filas no son un string largo, el formato es [largo del string][string], entonces si intentas parsear todo como si fuese un string te va a devolver cualquier cosa
        // let mut new_ordered_res_bytes = self.get_ordered_new_res_bytes(
        //     results_from_another_nodes,
        //     total_length_until_end_of_metadata,
        //     select,
        // )?;

        // le agrego el body de las filas a las que ya tenia
        // results_from_another_nodes.truncate(total_length_until_end_of_metadata);
        // results_from_another_nodes.append(&mut new_ordered_res_bytes);

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

    fn match_kind_of_conection_mode(
        &mut self,
        bytes: Vec<Byte>,
        mut tcp_stream: TcpStream,
    ) -> Result<()> {
        match self.mode() {
            ConnectionMode::Echo => {
                let printable_bytes = bytes
                    .iter()
                    .map(|b| format!("{:#X}", b))
                    .collect::<Vec<String>>();
                println!("[{} - ECHO] {}", self.id, printable_bytes.join(" "));
                if let Err(err) = tcp_stream.write_all(&bytes) {
                    println!("Error al escribir en el TCPStream:\n\n{}", err);
                }
                if let Err(err) = tcp_stream.flush() {
                    println!("Error haciendo flush desde el nodo:\n\n{}", err);
                }
            }
            ConnectionMode::Parsing => {
                let res = self.handle_request(&bytes[..], false);
                let _ = tcp_stream.write_all(&res[..]);
                if let Err(err) = tcp_stream.flush() {
                    println!("Error haciendo flush desde el nodo:\n\n{}", err);
                }
            }
        }
        Ok(())
    }
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
    let opcode = match Opcode::try_from(response[4]) {
        Ok(opcode) => opcode,
        Err(_err) => Opcode::RequestError,
    };
    if response.len() < 9 {
        return false;
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

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Se muestra en formato .csv
        // id,default_keyspace,keyspaces,tables,tables_and_partitions_keys_values
        writeln!(
            f,
            "{},{},{},{},{}",
            self.id,
            self.default_keyspace_name,
            hashmap_to_string(&self.keyspaces),
            hashmap_to_string(&self.tables),
            hashmap_vec_to_string(&self.tables_and_partitions_keys_values),
        )
    }
}

impl Serializable for Node {
    fn serialize(&self) -> Vec<Byte> {
        self.to_string().into_bytes()
    }

    fn deserialize(data: &[Byte]) -> Result<Self> {
        let line: String = String::from_utf8(data.to_vec())
            .map_err(|_| Error::ServerError("No se pudieron deserializar los datos".to_string()))?;
        let mut parameters: IntoIter<String> = line
            .split(",")
            .map(|s| s.to_string())
            .collect::<Vec<String>>()
            .into_iter();

        let id: Byte = parameters
            .next()
            .ok_or(Error::ServerError(
                "No se pudo obtener el ID del nodo".to_string(),
            ))?
            .parse()
            .map_err(|_| Error::ServerError("No se pudo parsear el ID del nodo".to_string()))?;

        let default_keyspace_name: String = parameters
            .next()
            .ok_or(Error::ServerError(
                "No se pudo obtener el keyspace por defecto del nodo".to_string(),
            ))?
            .parse()
            .map_err(|_| {
                Error::ServerError(
                    "No se pudo parsear el keyspace por defecto del nodo".to_string(),
                )
            })?;

        let keyspaces: String = parameters.next().ok_or(Error::ServerError(
            "No se pudo obtener los keyspaces del nodo".to_string(),
        ))?;
        let keyspaces: HashMap<String, Keyspace> = string_to_hashmap(&keyspaces).map_err(|_| {
            Error::ServerError("No se pudieron parsear los keyspaces del nodo".to_string())
        })?;

        let tables: String = parameters.next().ok_or(Error::ServerError(
            "No se pudo obtener las tablas del nodo".to_string(),
        ))?;
        let tables: HashMap<String, Table> = string_to_hashmap(&tables).map_err(|_| {
            Error::ServerError("No se pudieron parsear las tablas del nodo".to_string())
        })?;

        let tables_and_partitions_keys_values: String =
            parameters.next().ok_or(Error::ServerError(
                "No se pudo obtener las tablas y sus particiones del nodo".to_string(),
            ))?;
        let tables_and_partitions_keys_values: HashMap<String, Vec<String>> =
            string_to_hashmap_vec(&tables_and_partitions_keys_values).map_err(|_| {
                Error::ServerError(
                    "No se pudieron parsear las tablas y sus particiones del nodo".to_string(),
                )
            })?;

        Ok(Node {
            id,
            neighbours_states: HashMap::new(),
            endpoint_state: EndpointState::with_id(id),
            storage_addr: DiskHandler::get_node_storage(id),
            default_keyspace_name,
            keyspaces,
            tables,
            nodes_ranges: divide_range(0, 18446744073709551615, N_NODES as usize),
            tables_and_partitions_keys_values,
            pool: ThreadPool::build(N_THREADS)?,
            open_connections: OpenConnectionsMap::new(),
        })
    }
}

// esta comprobacion podriamos usarla en handle_result_from_node
// match Opcode::try_from(res[4])? {
//     Opcode::RequestError => return Err(Error::try_from(res[9..].to_vec())?),
//     Opcode::Result => self.handle_result_from_node(
//         &mut results_from_another_nodes,
//         res,
//         &select,
//     )?,
//     _ => {
//         return Err(Error::ServerError(
//             "Nodo manda opcode inesperado".to_string(),
//         ))
//     }
// };
