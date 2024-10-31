//! Módulo de nodos.

use std::{
    cmp::PartialEq,
    collections::{HashMap, HashSet},
    hash::{DefaultHasher, Hash, Hasher},
    io::{BufRead, BufReader, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    path::Path,
    sync::mpsc::{Receiver, Sender},
    thread::Builder,
};

use crate::protocol::headers::{
    flags::Flag, length::Length, opcode::Opcode, stream::Stream, version::Version,
};
use crate::protocol::{
    aliases::{results::Result, types::Byte},
    errors::error::Error,
    traits::Byteable,
};
use crate::server::{
    actions::opcode::{GossipInfo, SvAction},
    modes::ConnectionMode,
    nodes::{
        disk_handler::DiskHandler,
        graph::NodeHandle,
        port_type::PortType,
        states::{
            appstatus::AppStatus,
            endpoints::EndpointState,
            heartbeat::{
                HeartbeatState, {GenType, VerType},
            },
        },
        utils::{guess_id, send_to_node},
    },
    utils::get_available_sockets,
};
use crate::tokenizer::tokenizer::tokenize_query;
use crate::{
    parser::{
        data_types::keyspace_name::KeyspaceName,
        main_parser::make_parse,
        statements::{
            ddl_statement::{
                create_keyspace::CreateKeyspace, create_table::CreateTable,
                ddl_statement_parser::DdlStatement,
            },
            dml_statement::{
                dml_statement_parser::DmlStatement,
                main_statements::{insert::Insert, select::select_operation::Select},
            },
            statement::Statement,
        },
    },
    protocol::{headers::msg_headers::Headers, messages::responses::result_kinds::ResultKind},
};

use super::{
    graph::{N_NODES, START_ID},
    keyspace::Keyspace,
    table::Table,
    utils::{divide_range, send_to_node_and_wait_response},
};

/// El ID de un nodo. No se tienen en cuenta casos de cientos de nodos simultáneos,
/// así que un byte debería bastar para representarlo.
pub type NodeId = u8;

/// Mapea todos los estados de los vecinos y de sí mismo.
pub type NodesMap = HashMap<NodeId, EndpointState>;

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
}

impl Node {
    /// Crea un nuevo nodo.
    pub fn new(id: NodeId, mode: ConnectionMode) -> Self {
        let storage_addr = DiskHandler::new_node_storage(id);
        let nodes_ranges = divide_range(0, 18446744073709551615, N_NODES as usize);
        Self {
            id,
            neighbours_states: NodesMap::new(),
            endpoint_state: EndpointState::with_id_and_mode(id, mode),
            storage_addr,
            default_keyspace_name: "".to_string(),
            keyspaces: HashMap::new(),
            tables: HashMap::new(),
            nodes_ranges,
            tables_and_partitions_keys_values: HashMap::new(),
        }
    }

    fn add_table(&mut self, table: Table) {
        let table_name = table.get_name().to_string();
        let partition_key = table.get_partition_key();
        self.tables.insert(table_name.clone(), table);
        self.tables_and_partitions_keys_values
            .insert(table_name, partition_key);
    }

    fn get_table(&self, table_name: &str) -> Result<&Table> {
        match self.tables.get(table_name) {
            Some(table) => Ok(table),
            None => Err(Error::ServerError(
                "La tabla solicitada no existe".to_string(),
            )),
        }
    }

    fn add_keyspace(&mut self, keyspace: Keyspace) {
        self.keyspaces
            .insert(keyspace.get_name().to_string(), keyspace);
    }

    fn get_keyspace(&self, table_name: &str) -> Result<&Keyspace> {
        let table = match self.tables.get(table_name) {
            Some(table) => table,
            None => {
                return Err(Error::ServerError(
                    "La tabla solicitada no existe".to_string(),
                ))
            }
        };
        match self.keyspaces.get(table.keyspace.as_str()) {
            Some(keyspace) => Ok(keyspace),
            None => Err(Error::ServerError(
                "El keyspace solicitado no existe".to_string(),
            )),
        }
    }

    fn set_default_keyspace(&mut self, keyspace_name: String) {
        self.default_keyspace_name = keyspace_name;
    }

    fn get_default_keyspace(&self) -> Result<&Keyspace> {
        match self.keyspaces.get(&self.default_keyspace_name) {
            Some(keyspace) => Ok(keyspace),
            None => Err(Error::ServerError(
                "El keyspace por defecto no existe".to_string(),
            )),
        }
    }

    /// Consulta el ID del nodo.
    fn get_id(&self) -> &NodeId {
        &self.id
    }

    /// Consulta el estado del nodo.
    pub fn get_endpoint_state(&self) -> &EndpointState {
        &self.endpoint_state
    }

    /// Consulta los IDs de los vecinos, incluyendo el propio.
    fn get_neighbours_ids(&self) -> Vec<NodeId> {
        self.neighbours_states.keys().copied().collect()
    }

    /// Hashea el valor recibido.
    fn hash_value(value: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        value.hash(&mut hasher);
        hasher.finish()
    }

    /// Selecciona un ID de nodo conforme al _hashing_ del valor del _partition key_ y los rangos de los nodos.
    fn select_node(&self, value: &str) -> NodeId {
        let hash_val = Self::hash_value(value);

        let mut i = 0;
        for (a, b) in &self.nodes_ranges {
            if *a <= hash_val && hash_val < *b {
                return START_ID + (i - 1) as NodeId;
            }
            i += 1;
        }

        START_ID + (i - 1) as NodeId
    }

    fn send_message_and_wait_response(
        &self,
        bytes: Vec<Byte>,
        node_id: u8,
        port_type: PortType,
    ) -> Result<Vec<u8>> {
        send_to_node_and_wait_response(node_id, bytes, port_type)
    }

    /// Manda un mensaje en bytes al nodo correspondiente mediante el _hashing_ del valor del _partition key_.
    fn send_message(&mut self, bytes: Vec<Byte>, value: String, port_type: PortType) -> Result<()> {
        send_to_node(self.select_node(&value), bytes, port_type)
    }

    /// Manda un mensaje en bytes a todos los vecinos del nodo.
    fn notice_all_neighbours(&self, bytes: Vec<Byte>, port_type: PortType) -> Result<()> {
        for neighbour_id in self.get_neighbours_ids() {
            if neighbour_id == self.id {
                continue;
            }
            send_to_node(neighbour_id, bytes.clone(), port_type.clone())?;
        }
        Ok(())
    }

    /// Compara si el _heartbeat_ de un nodo es más nuevo que otro.
    fn is_newer(&self, other: &Self) -> bool {
        self.endpoint_state.is_newer(&other.endpoint_state)
    }

    /// Verifica rápidamente si un mensaje es de tipo [EXIT](SvAction::Exit).
    fn is_exit(bytes: &[Byte]) -> bool {
        if let Some(action) = SvAction::get_action(bytes) {
            if matches!(action, SvAction::Exit(_)) {
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
    fn has_endpoint_state(&self, state: &EndpointState) -> bool {
        self.neighbours_states
            .contains_key(&guess_id(state.get_addr()))
    }

    fn add_neighbour_state(&mut self, state: EndpointState) {
        if !self.has_endpoint_state(&state) {
            self.neighbours_states
                .insert(guess_id(state.get_addr()), state);
        }
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
    fn is_bootstraping(&self) -> bool {
        matches!(
            self.endpoint_state.get_appstate().get_status(),
            AppStatus::Bootstrap
        )
    }

    /// Consulta el estado de _heartbeat_.
    fn get_beat(&mut self) -> (GenType, VerType) {
        self.endpoint_state.get_heartbeat().as_tuple()
    }

    /// Avanza el tiempo para el nodo.
    fn beat(&mut self) -> VerType {
        self.endpoint_state.beat()
    }

    /// Inicia un intercambio de _gossip_ con los vecinos dados.
    fn gossip(&mut self, neighbours: HashSet<NodeId>) -> Result<()> {
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

    /// Crea el hilo que procesa _requests_.
    ///
    /// Dicho hilo toma _ownership_ del nodo, por lo que ya no es accesible después
    /// sino es con mensajes al puerto de escucha.
    pub fn request_processor(
        mut self,
        receiver: Receiver<(TcpStream, Vec<Byte>)>,
        listeners: Vec<NodeHandle>,
    ) -> Result<NodeHandle> {
        let builder = Builder::new().name("processor".to_string());
        match builder.spawn(move || {
            loop {
                match receiver.recv() {
                    Err(err) => {
                        println!("Error recibiendo request en hilo procesador:\n\n{}", err);
                        break;
                    }

                    Ok((tcp_stream, bytes)) => match self.process_tcp(tcp_stream, bytes) {
                        Ok(stop) => {
                            if stop {
                                break;
                            }
                        }
                        Err(err) => {
                            println!("Error procesando request en hilo procesador:\n\n{}", err);
                        }
                    },
                }
            }

            // Esperamos primero a que todos los hilos relacionados mueran primero.
            for listener in listeners {
                if listener.join().is_err() {
                    println!("Ocurrió un error mientras se esperaba a que termine un escuchador.");
                }
            }

            Ok(())
        }) {
            Ok(proc_handler) => Ok(proc_handler),
            Err(err) => Err(Error::ServerError(format!(
                "Error em el hilo procesador:\n\n{}",
                err
            ))),
        }
    }

    /// Se recibe un mensaje [SYN](crate::server::actions::opcode::SvAction::Syn).
    fn syn(&mut self, emissor_id: NodeId, gossip_info: GossipInfo) -> Result<()> {
        let mut own_gossip = GossipInfo::new(); // quiero info de estos nodos
        let mut response_nodes = NodesMap::new(); // doy info de estos nodos

        for (node_id, heartbeat) in &gossip_info {
            let endpoint_state_opt = &self.neighbours_states.get(node_id);
            match endpoint_state_opt {
                Some(endpoint_state) => {
                    let cur_heartbeat = endpoint_state.get_heartbeat();
                    if cur_heartbeat > heartbeat {
                        response_nodes.insert(*node_id, (*endpoint_state).clone());
                    } else if cur_heartbeat < heartbeat {
                        own_gossip.insert(*node_id, endpoint_state.clone_heartbeat());
                    }
                }
                None => {
                    // Se trata de un vecino que no conocemos aún
                    own_gossip.insert(*node_id, HeartbeatState::minimal());
                }
            }
        }

        // Ahora rondamos nuestros vecinos para ver si tenemos uno que el nodo emisor no
        for (own_node_id, endpoint_state) in &self.neighbours_states {
            if !gossip_info.contains_key(own_node_id) {
                response_nodes.insert(*own_node_id, endpoint_state.clone());
            }
        }

        if let Err(err) = send_to_node(
            emissor_id,
            SvAction::Ack(self.get_id().to_owned(), own_gossip, response_nodes).as_bytes(),
            PortType::Priv,
        ) {
            println!(
                "Ocurrió un error al mandar un mensaje ACK al nodo [{}]:\n\n{}",
                emissor_id, err
            );
        }
        Ok(())
    }

    /// Se recibe un mensaje [ACK](crate::server::actions::opcode::SvAction::Ack).
    fn ack(
        &mut self,
        receptor_id: NodeId,
        gossip_info: GossipInfo,
        nodes_map: NodesMap,
    ) -> Result<()> {
        // Poblamos un mapa con los estados que pide el receptor
        let mut nodes_for_receptor = NodesMap::new();
        for (node_id, heartbeat) in &gossip_info {
            let cur_endpoint_state = &self.neighbours_states[node_id];
            if cur_endpoint_state.get_heartbeat() > heartbeat {
                // hacemos doble chequeo que efectivamente tenemos información más nueva
                nodes_for_receptor.insert(*node_id, cur_endpoint_state.clone());
            }
        }

        // Reemplazamos la información de nuestros vecinos por la más nueva que viene del nodo receptor
        self.update_neighbours(nodes_map)?;

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

    /// Escucha por los eventos que recibe del cliente.

    pub fn cli_listen(
        socket: SocketAddr,
        proc_sender: Sender<(TcpStream, Vec<Byte>)>,
    ) -> Result<()> {
        Self::listen(socket, proc_sender, PortType::Cli)
    }

    /// Escucha por los eventos que recibe de otros nodos o estructuras internas.

    pub fn priv_listen(
        socket: SocketAddr,
        proc_sender: Sender<(TcpStream, Vec<Byte>)>,
    ) -> Result<()> {
        Self::listen(socket, proc_sender, PortType::Priv)
    }

    /// El escuchador de verdad.
    ///
    /// Las otras funciones son wrappers para no repetir código.
    fn listen(
        socket: SocketAddr,
        proc_sender: Sender<(TcpStream, Vec<Byte>)>,
        port_type: PortType,
    ) -> Result<()> {
        let listener = match TcpListener::bind(socket) {
            Ok(tcp_listener) => tcp_listener,
            Err(_) => {
                return Err(Error::ServerError(format!(
                    "No se pudo bindear a la dirección '{}'",
                    socket
                )))
            }
        };

        for tcp_stream_res in listener.incoming() {
            match tcp_stream_res {
                Err(_) => {
                    let falla = match &port_type {
                        PortType::Cli => "cliente",
                        PortType::Priv => "nodo o estructura interna",
                    };
                    return Err(Error::ServerError(format!(
                        "Un {} no pudo conectarse al nodo con ID {}",
                        falla,
                        guess_id(&socket.ip())
                    )));
                }
                Ok(tcp_stream) => {
                    let buffered_stream = match tcp_stream.try_clone() {
                        Ok(cloned) => cloned,
                        Err(err) => {
                            return Err(Error::ServerError(format!(
                                "No se pudo clonar el stream:\n\n{}",
                                err
                            )))
                        }
                    };
                    let mut bufreader = BufReader::new(buffered_stream);
                    let bytes_vec = match bufreader.fill_buf() {
                        Ok(recv) => recv.to_vec(),
                        Err(err) => {
                            return Err(Error::ServerError(format!(
                                "No se pudo escribir los bytes:\n\n{}",
                                err
                            )))
                        }
                    };
                    // consumimos los bytes del stream para no mandarlos de vuelta en la response
                    bufreader.consume(bytes_vec.len());
                    let can_exit = Self::is_exit(&bytes_vec[..]);
                    if let Err(err) = proc_sender.send((tcp_stream, bytes_vec)) {
                        println!("Error mandando bytes al procesador:\n\n{}", err);
                    }
                    // El procesamiento del stream ocurre en otro hilo, así que necesitamos
                    // verificar si salimos aparte.
                    if can_exit {
                        break;
                    }
                }
            }
        }

        Ok(())
    }

    /// Procesa una _request_ en forma de [Byte]s.
    /// También devuelve un [bool] indicando si se debe parar el hilo.
    ///
    /// Esta función no debería ser llamada en los listeners, y está más pensada para el hilo
    /// procesador del nodo.

    pub fn process_tcp(&mut self, mut tcp_stream: TcpStream, bytes: Vec<Byte>) -> Result<bool> {
        match SvAction::get_action(&bytes[..]) {
            Some(action) => match self.handle_sv_action(action, tcp_stream) {
                Ok(stop_loop) => Ok(stop_loop),
                Err(err) => {
                    println!(
                        "[{} - ACTION] Error en la acción del servidor: {}",
                        self.id, err
                    );
                    Ok(false)
                }
            },
            None => match self.mode() {
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
                    Ok(false)
                }
                ConnectionMode::Parsing => {
                    let res = self.handle_request(&bytes[..], false);
                    let _ = tcp_stream.write_all(&res[..]);
                    if let Err(err) = tcp_stream.flush() {
                        println!("Error haciendo flush desde el nodo:\n\n{}", err);
                    }
                    Ok(false)
                }
            },
        }
    }

    /// Maneja una acción de servidor.
    fn handle_sv_action(&mut self, action: SvAction, mut tcp_stream: TcpStream) -> Result<bool> {
        match action {
            SvAction::Exit(proc_stop) => Ok(proc_stop),
            SvAction::Beat => {
                self.beat();
                Ok(false)
            }
            SvAction::Gossip(neighbours) => {
                self.gossip(neighbours)?;
                Ok(false)
            }
            SvAction::Syn(emissor_id, gossip_info) => {
                self.syn(emissor_id, gossip_info)?;
                Ok(false)
            }
            SvAction::Ack(receptor_id, gossip_info, nodes_map) => {
                self.ack(receptor_id, gossip_info, nodes_map)?;
                Ok(false)
            }
            SvAction::Ack2(nodes_map) => {
                self.ack2(nodes_map)?;
                Ok(false)
            }
            SvAction::NewNeighbour(state) => {
                self.add_neighbour_state(state);
                Ok(false)
            }
            SvAction::SendEndpointState(id) => {
                self.send_endpoint_state(id);
                Ok(false)
            }
            SvAction::Shutdown => {
                for socket in get_available_sockets() {
                    let node_id = guess_id(&socket.ip());
                    send_to_node(node_id, SvAction::Exit(false).as_bytes(), PortType::Cli)?;
                    send_to_node(node_id, SvAction::Exit(true).as_bytes(), PortType::Priv)?;
                }
                // no interrumpe el nodo porque es el trabajo de EXIT
                Ok(false)
            }
            SvAction::InternalQuery(bytes) => {
                let response = self.handle_request(&bytes, true);
                let _ = tcp_stream.write_all(&response[..]);
                if let Err(err) = tcp_stream.flush() {
                    Err(Error::ServerError(err.to_string()))
                } else {
                    Ok(false)
                }
            }
        }
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
        // response.append(&mut left_response);
        // aca deberiamos mandar la response de alguna manera
        // response.append(&mut left_response);
        // Ok(left_response)
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
        if let Ok(query) = String::from_utf8(request[9..(lenght.len as usize) + 9].to_vec()) {
            let res = match make_parse(&mut tokenize_query(&query)) {
                Ok(statement) => {
                    if internal_request {
                        self.handle_internal_statement(statement)
                    } else {
                        self.handle_statement(statement, request)
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
            "No se pudieron transformar los bytes a string".to_string(),
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
    fn handle_internal_statement(&mut self, statement: Statement) -> Result<Vec<Byte>> {
        match statement {
            Statement::DdlStatement(ddl_statement) => {
                self.handle_internal_ddl_statement(ddl_statement)
            }
            Statement::DmlStatement(dml_statement) => {
                self.handle_internal_dml_statement(dml_statement)
            }
            Statement::UdtStatement(_udt_statement) => todo!(),
        }
    }

    /// Maneja una declaración DDL.
    fn handle_internal_ddl_statement(&mut self, ddl_statement: DdlStatement) -> Result<Vec<Byte>> {
        match ddl_statement {
            DdlStatement::UseStatement(keyspace_name) => self.process_use_statement(keyspace_name),
            DdlStatement::CreateKeyspaceStatement(create_keyspace) => {
                self.process_create_keyspace_statement(create_keyspace)
            }
            DdlStatement::AlterKeyspaceStatement(_alter_keyspace) => todo!(),
            DdlStatement::DropKeyspaceStatement(_drop_keyspace) => todo!(),
            DdlStatement::CreateTableStatement(create_table) => {
                self.process_create_table_statement(create_table)
            }
            DdlStatement::AlterTableStatement(_alter_table) => todo!(),
            DdlStatement::DropTableStatement(_drop_table) => todo!(),
            DdlStatement::TruncateStatement(_truncate) => todo!(),
        }
    }

    fn process_use_statement(&mut self, keyspace_name: KeyspaceName) -> Result<Vec<u8>> {
        let name = keyspace_name.get_name().to_string();
        if self.keyspaces.contains_key(&name) {
            self.set_default_keyspace(name.clone());
            Ok(self.create_result_void())
        } else {
            if self.check_if_keyspace_exists(keyspace_name) {
                self.set_default_keyspace(name.clone());
                return Ok(self.create_result_void());
            }
            Err(Error::ServerError(
                "El keyspace solicitado no existe".to_string(),
            ))
        }
    }

    fn check_if_keyspace_exists(&self, keyspace_name: KeyspaceName) -> bool {
        let keyspace_addr = format!("{}/{}", self.storage_addr, keyspace_name.get_name());
        let path_folder = Path::new(&keyspace_addr);
        path_folder.exists() && path_folder.is_dir()
    }

    fn process_create_keyspace_statement(
        &mut self,
        create_keyspace: CreateKeyspace,
    ) -> Result<Vec<u8>> {
        match DiskHandler::create_keyspace(create_keyspace, &self.storage_addr) {
            Ok(Some(keyspace)) => self.add_keyspace(keyspace),
            Ok(None) => return Ok(self.create_result_void()),
            Err(err) => return Err(err),
        };
        Ok(self.create_result_void())
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

    fn process_create_table_statement(&mut self, create_table: CreateTable) -> Result<Vec<u8>> {
        let default_keyspace_name = match self.get_default_keyspace() {
            Ok(keyspace) => keyspace.get_name().to_string(),
            Err(err) => return Err(err),
        };
        match DiskHandler::create_table(create_table, &self.storage_addr, &default_keyspace_name) {
            Ok(Some(table)) => {
                self.add_table(table);
            }
            Ok(None) => return Err(Error::ServerError("No se pudo crear la tabla".to_string())),
            Err(err) => return Err(err),
        };
        Ok(self.create_result_void())
    }

    /// Maneja una declaración DML.
    fn handle_internal_dml_statement(&mut self, dml_statement: DmlStatement) -> Result<Vec<Byte>> {
        match dml_statement {
            DmlStatement::SelectStatement(select) => self.process_select(&select),
            DmlStatement::InsertStatement(insert) => self.process_insert(&insert),
            DmlStatement::UpdateStatement(_update) => todo!(),
            DmlStatement::DeleteStatement(_delete) => todo!(),
            DmlStatement::BatchStatement(_batch) => todo!(),
        }
    }

    fn process_select(&self, select: &Select) -> Result<Vec<Byte>> {
        let table = match self.get_table(&select.from.get_name()) {
            Ok(table) => table,
            Err(err) => return Err(err),
        };
        DiskHandler::do_select(
            select,
            &self.storage_addr,
            table,
            &self.default_keyspace_name,
        )
    }

    fn process_insert(&mut self, insert: &Insert) -> Result<Vec<Byte>> {
        let table_name = insert.table.get_name();
        let table = match self.get_table(&table_name) {
            Ok(table) => table,
            Err(err) => return Err(err),
        };
        DiskHandler::do_insert(
            insert,
            &self.storage_addr,
            table,
            &self.default_keyspace_name,
        )?;
        Ok(self.create_result_void())
    }

    fn handle_statement(&mut self, statement: Statement, request: &[Byte]) -> Result<Vec<Byte>> {
        match statement {
            Statement::DdlStatement(ddl_statement) => self.handle_ddl_statement(ddl_statement),
            Statement::DmlStatement(dml_statement) => {
                self.handle_dml_statement(dml_statement, request)
            }
            Statement::UdtStatement(_udt_statement) => todo!(),
        }
    }

    fn handle_ddl_statement(&mut self, ddl_statement: DdlStatement) -> Result<Vec<u8>> {
        match ddl_statement {
            DdlStatement::UseStatement(keyspace_name) => self.process_use_statement(keyspace_name),
            DdlStatement::CreateKeyspaceStatement(create_keyspace) => {
                self.process_create_keyspace_statement(create_keyspace)
            }
            DdlStatement::AlterKeyspaceStatement(_alter_keyspace) => {
                todo!()
            }
            DdlStatement::DropKeyspaceStatement(_drop_keyspace) => {
                todo!()
            }
            DdlStatement::CreateTableStatement(create_table) => {
                self.process_create_table_statement(create_table)
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
    ) -> Result<Vec<u8>> {
        match dml_statement {
            DmlStatement::SelectStatement(select) => self.select_with_other_nodes(select, request),
            DmlStatement::InsertStatement(insert) => self.insert_with_other_nodes(insert, request),
            DmlStatement::UpdateStatement(_update) => {
                todo!()
            }
            DmlStatement::DeleteStatement(_delete) => {
                todo!()
            }
            DmlStatement::BatchStatement(_batch) => {
                todo!()
            }
        }
    }

    fn insert_with_other_nodes(&mut self, insert: Insert, request: &[u8]) -> Result<Vec<Byte>> {
        let table_name: String = insert.table.get_name();
        let partitions_keys_to_nodes = self.get_partition_keys_values(&table_name)?.clone();
        let mut response: Vec<Byte> = Vec::new();
        for partition_key_value in partitions_keys_to_nodes {
            let node_id = self.select_node(&partition_key_value);
            let replication_factor = self.get_replicas_from_table_name(&table_name)?;
            for i in 0..replication_factor - 1 {
                let node_to_replicate = self.next_node_to_replicate_data(
                    node_id,
                    i as u8,
                    START_ID,
                    START_ID + N_NODES,
                );
                response = if node_id != self.id {
                    self.send_message_and_wait_response(
                        SvAction::InternalQuery(request.to_vec()).as_bytes(),
                        node_to_replicate,
                        PortType::Priv,
                    )?
                } else {
                    self.process_insert(&insert)?
                }
            }
        }
        Ok(response)
    }

    fn next_node_to_replicate_data(
        &self,
        first_node_to_replicate: u8,
        node_iterator: u8,
        min: u8,
        max: u8,
    ) -> u8 {
        let nodes_range = max - min + 1;
        min + ((first_node_to_replicate - min + node_iterator) % nodes_range)
    }

    fn select_with_other_nodes(&mut self, select: Select, request: &[Byte]) -> Result<Vec<Byte>> {
        let mut results_from_another_nodes: Vec<u8> = Vec::new();
        let partitions_keys_to_nodes = self.get_partition_keys_values(&select.from.get_name())?;
        let mut consulted_nodes: Vec<String> = Vec::new();
        for partition_key_value in partitions_keys_to_nodes {
            let node_id = self.select_node(partition_key_value);
            if !consulted_nodes.contains(partition_key_value) {
                if node_id == self.id {
                    self.handle_result_from_node(
                        &mut results_from_another_nodes,
                        self.process_select(&select)?,
                        &select,
                    )?;
                } else {
                    let res = self.send_message_and_wait_response(
                        SvAction::InternalQuery(request.to_vec()).as_bytes(),
                        node_id,
                        PortType::Priv,
                    )?;
                    match Opcode::try_from(res[4])? {
                        Opcode::RequestError => return Err(Error::try_from(res[9..].to_vec())?),
                        Opcode::Result => self.handle_result_from_node(
                            &mut results_from_another_nodes,
                            res,
                            &select,
                        )?,
                        _ => {
                            return Err(Error::ServerError(
                                "Nodo manda opcode inesperado".to_string(),
                            ))
                        }
                    };
                    consulted_nodes.push(partition_key_value.to_string());
                }
            }
        }
        Ok(results_from_another_nodes)
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
        results_from_another_nodes: &mut Vec<u8>,
        mut result_from_actual_node: Vec<u8>,
        select: &Select,
    ) -> Result<()> {
        if results_from_another_nodes.is_empty() {
            results_from_another_nodes.append(&mut result_from_actual_node);
            return Ok(());
        }
        let size = Length::try_from(result_from_actual_node[5..9].to_vec())?;

        let total_length_from_metadata =
            self.get_columns_metadata_length(results_from_another_nodes);

        let mut new_res =
            results_from_another_nodes[total_length_from_metadata..size.len as usize].to_vec();
        results_from_another_nodes.append(&mut new_res);

        let mut new_ordered_res_bytes = self.get_ordered_new_res_bytes(
            results_from_another_nodes,
            total_length_from_metadata,
            select,
        )?;

        // le agrego el body de las filas a las que ya tenia
        results_from_another_nodes.truncate(total_length_from_metadata);
        results_from_another_nodes.append(&mut new_ordered_res_bytes);

        Ok(())
    }

    fn get_columns_metadata_length(&self, results_from_another_nodes: &mut [u8]) -> usize {
        let mut total_length_from_metadata: usize = 12;
        let column_quantity = &results_from_another_nodes[13..17];
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
            total_length_from_metadata += (name_length as usize) + 2 + 2; // Sumamos el largo del <col_spec_i>
        }
        total_length_from_metadata
    }

    fn get_ordered_new_res_bytes(
        &self,
        results_from_another_nodes: &[Byte],
        total_length_from_metadata: usize,
        select: &Select,
    ) -> Result<Vec<Byte>> {
        let table_name = select.from.get_name();
        let table = match self.get_table(&table_name) {
            Ok(table) => table,
            Err(err) => return Err(err),
        };

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
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.endpoint_state.eq(&other.endpoint_state)
    }
}
