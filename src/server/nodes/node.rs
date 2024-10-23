//! Módulo de nodos.

use std::{
    cmp::PartialEq,
    collections::{HashMap, HashSet},
    io::Read,
    net::{TcpListener, TcpStream},
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
        main_parser::make_parse,
        statements::{
            ddl_statement::ddl_statement_parser::DdlStatement,
            dml_statement::dml_statement_parser::DmlStatement, statement::Statement,
        },
    },
    protocol::headers::{
        flags::Flag, length::Length, opcode::Opcode, stream::Stream, version::Version,
    },
};

use super::disk_handler::DiskHandler;

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
}

impl Node {
    /// Crea un nuevo nodo.
    pub fn new(id: NodeId, mode: ConnectionMode) -> Self {
        let storage_addr = DiskHandler::new_node_storage(id);

        Self {
            id,
            neighbours_states: NodesMap::new(),
            endpoint_state: EndpointState::with_id_and_mode(id, mode),
            storage_addr,
        }
    }

    /// Consulta el ID del nodo.
    pub fn get_id(&self) -> &NodeId {
        &self.id
    }

    /// Consulta el estado del nodo.
    pub fn get_endpoint_state(&self) -> &EndpointState {
        &self.endpoint_state
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
            PortType::Priv.into(),
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
    pub fn has_endpoint_state(&self, state: &EndpointState) -> bool {
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
    pub fn get_gossip_info(&self) -> GossipInfo {
        let mut gossip_info = GossipInfo::new();
        for (node_id, endpoint_state) in &self.neighbours_states {
            gossip_info.insert(node_id.to_owned(), endpoint_state.clone_heartbeat());
        }

        gossip_info
    }

    /// Ve si el nodo es un nodo "hoja".
    pub fn leaf(&self) -> bool {
        self.neighbours_states.is_empty()
    }

    /// Consulta el modo de conexión del nodo.
    pub fn mode(&self) -> &ConnectionMode {
        self.endpoint_state.get_appstate().get_mode()
    }

    /// Consulta si el nodo todavía esta booteando.
    pub fn is_bootstraping(&self) -> bool {
        matches!(
            self.endpoint_state.get_appstate().get_status(),
            AppStatus::Bootstrap
        )
    }

    /// Consulta el estado de _heartbeat_.
    pub fn get_beat(&mut self) -> (GenType, VerType) {
        self.endpoint_state.get_heartbeat().as_tuple()
    }

    /// Avanza el tiempo para el nodo.
    pub fn beat(&mut self) -> VerType {
        self.endpoint_state.beat()
    }

    /// Inicia un intercambio de _gossip_ con los vecinos dados.
    pub fn gossip(&mut self, neighbours: HashSet<NodeId>) -> Result<()> {
        for neighbour_id in neighbours {
            if let Err(err) = send_to_node(
                neighbour_id,
                SvAction::Syn(self.get_id().to_owned(), self.get_gossip_info()).as_bytes(),
                PortType::Priv.into(),
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
    pub fn syn(&mut self, emissor_id: NodeId, gossip_info: GossipInfo) -> Result<()> {
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
            PortType::Priv.into(),
        ) {
            println!(
                "Ocurrió un error al mandar un mensaje ACK al nodo [{}]:\n\n{}",
                emissor_id, err
            );
        }
        Ok(())
    }

    /// Se recibe un mensaje [ACK](crate::server::actions::opcode::SvAction::Ack).
    pub fn ack(
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
            PortType::Priv.into(),
        ) {
            println!(
                "Ocurrió un error al mandar un mensaje ACK2 al nodo [{}]:\n\n{}",
                receptor_id, err
            );
        }
        Ok(())
    }

    /// Se recibe un mensaje [ACK2](crate::server::actions::opcode::SvAction::Ack2).
    pub fn ack2(&mut self, nodes_map: NodesMap) -> Result<()> {
        self.update_neighbours(nodes_map)
    }

    /// Escucha por los eventos que recibe del cliente.
    pub fn cli_listen(&mut self) -> Result<()> {
        let socket = self.endpoint_state.socket(PortType::Cli);
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
                    return Err(Error::ServerError(format!(
                        "Un cliente no pudo conectarse al nodo con ID {}",
                        self.id
                    )))
                }
                Ok(tcp_stream) => match self.process_tcp(tcp_stream) {
                    Ok(stop) => {
                        if stop {
                            break;
                        }
                    }
                    Err(err) => {
                        println!("Ocurrió un error al procesar el stream:\n\n{}", err)
                    }
                },
            }
        }
        Ok(())
    }

    /// Escucha por los eventos que recibe de otros nodos o estructuras internas.
    pub fn priv_listen(&mut self) -> Result<()> {
        let socket = self.endpoint_state.socket(PortType::Priv);
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
                    return Err(Error::ServerError(format!(
                        "Un cliente no pudo conectarse al nodo con ID {}",
                        self.id
                    )));
                }
                Ok(tcp_stream) => match self.process_tcp(tcp_stream) {
                    Ok(stop) => {
                        if stop {
                            break;
                        }
                    }
                    Err(err) => {
                        println!("Ocurrió un error al procesar el stream:\n\n{}", err)
                    }
                },
            }
        }

        Ok(())
    }

    /// Procesa una _request_ de un [TcpStream].
    ///
    /// Devuelve un [bool] indicando si se debe detener el stream.
    fn process_tcp(&mut self, tcp_stream: TcpStream) -> Result<bool> {
        let bytes: Vec<Byte> = tcp_stream.bytes().flatten().collect();
        match SvAction::get_action(&bytes[..]) {
            Some(action) => match self.handle_sv_action(action) {
                Ok(continue_loop) => Ok(!continue_loop),
                Err(err) => {
                    println!(
                        "[{} - ACTION] Error en la acción del servidor: {}",
                        self.id, err
                    );
                    Ok(false)
                }
            },
            None => {
                match self.mode() {
                    ConnectionMode::Echo => {
                        if let Ok(query) = String::from_utf8(bytes) {
                            println!("[{} - ECHO] {}", self.id, query)
                        }
                        Ok(false)
                    }
                    ConnectionMode::Parsing => {
                        println!("Deberia mandarse lo de abajo");
                        // tcp_stream.write_all(&mut self.handle_request(&mut bytes));
                        Ok(false)
                    }
                }
            }
        }
    }

    /// Maneja una acción de servidor.
    fn handle_sv_action(&mut self, action: SvAction) -> Result<bool> {
        match action {
            SvAction::Exit => Ok(false),
            SvAction::Beat => {
                self.beat();
                Ok(true)
            }
            SvAction::Gossip(neighbours) => {
                self.gossip(neighbours)?;
                Ok(true)
            }
            SvAction::Syn(emissor_id, gossip_info) => {
                self.syn(emissor_id, gossip_info)?;
                Ok(true)
            }
            SvAction::Ack(receptor_id, gossip_info, nodes_map) => {
                self.ack(receptor_id, gossip_info, nodes_map)?;
                Ok(true)
            }
            SvAction::Ack2(nodes_map) => {
                self.ack2(nodes_map)?;
                Ok(true)
            }
            SvAction::NewNeighbour(state) => {
                self.add_neighbour_state(state);
                Ok(true)
            }
            SvAction::SendEndpointState(id) => {
                self.send_endpoint_state(id);
                Ok(true)
            }
            SvAction::Shutdown => {
                for socket in get_available_sockets() {
                    let node_id = guess_id(&socket.ip());
                    if self.id == node_id {
                        // no mandarse el mensaje a sí mismo
                        continue;
                    }
                    send_to_node(node_id, SvAction::Exit.as_bytes(), PortType::Priv);
                }
                Ok(false)
            }
        }
    }

    /// Maneja una request.
    fn handle_request(&self, request: &mut [Byte]) -> Result<Vec<Byte>> {
        if request.len() < 9 {
            return Err(Error::ProtocolError(
                "No se cumple el protocolo del header".to_string(),
            ));
        }
        let _version = Version::try_from(request[0])?;
        let _flags = Flag::try_from(request[1])?;
        let _stream = Stream::try_from(request[2..4].to_vec())?;
        let opcode = Opcode::try_from(request[4])?;
        let lenght = Length::try_from(request[5..9].to_vec())?;
        let mut response: Vec<Byte> = Vec::new();
        response.append(&mut request[..4].to_vec());
        // VER QUE HACER CON LAS FLAGS

        // Cada handler deberia devolver un Vec<Byte> que contenga: que opcode de respuesta mandar,
        // con que lenght y el body si es que tiene
        let mut _left_response = match opcode {
            Opcode::Startup => self.handle_startup(request, lenght),
            Opcode::Options => self.handle_options(),
            Opcode::Query => self.handle_query(request, lenght),
            Opcode::Prepare => self.handle_prepare(),
            Opcode::Execute => self.handle_execute(),
            Opcode::Register => self.handle_register(),
            Opcode::Batch => self.handle_batch(),
            Opcode::AuthResponse => self.handle_auth_response(),
            _ => {
                return Err(Error::ProtocolError(
                    "El opcode recibido no es una request".to_string(),
                ))
            }
        };
        // response.append(&mut _left_response);
        // aca deberiamos mandar la response de alguna manera
        response.append(&mut _left_response);
        Ok(response)
    }

    fn handle_startup(&self, _request: &mut [Byte], _lenght: Length) -> Vec<Byte> {
        // El body es un [string map] con posibles opciones
        vec![0]
    }

    fn handle_options(&self) -> Vec<Byte> {
        // No tiene body
        // Responder con supported
        Opcode::Supported.as_bytes();
        vec![0]
    }

    fn handle_query(&self, request: &mut [Byte], lenght: Length) -> Vec<Byte> {
        if let Ok(query) = String::from_utf8(request[9..(lenght.len as usize) + 9].to_vec()) {
            let res = match make_parse(&mut tokenize_query(&query)) {
                Ok(statement) => match statement {
                    Statement::DdlStatement(ddl_statement) => {
                        self.handle_ddl_statement(ddl_statement)
                    }
                    Statement::DmlStatement(dml_statement) => {
                        self.handle_dml_statement(dml_statement)
                    }
                    Statement::UdtStatement(_udt_statement) => {
                        todo!();
                    }
                },
                Err(err) => {
                    return err.as_bytes();
                }
            };
            return res;
            // aca usariamos la query como corresponda
        }
        Error::ServerError("No se pudieron transformar los bytes a string".to_string()).as_bytes()
    }

    fn handle_prepare(&self) -> Vec<Byte> {
        // El body es <query><flags>[<keyspace>]
        vec![0]
    }

    fn handle_execute(&self) -> Vec<Byte> {
        // El body es <id><result_metadata_id><query_parameters>
        vec![0]
    }

    fn handle_register(&self) -> Vec<Byte> {
        vec![0]
    }

    fn handle_batch(&self) -> Vec<Byte> {
        vec![0]
    }

    fn handle_auth_response(&self) -> Vec<Byte> {
        vec![0]
    }

    /// Maneja una declaración DDL.
    fn handle_ddl_statement(&self, ddl_statement: DdlStatement) -> Vec<Byte> {
        match ddl_statement {
            DdlStatement::UseStatement(_keyspace_name) => {
                todo!()
            }
            DdlStatement::CreateKeyspaceStatement(_create_keyspace) => {
                todo!()
            }
            DdlStatement::AlterKeyspaceStatement(_alter_keyspace) => {
                todo!()
            }
            DdlStatement::DropKeyspaceStatement(_drop_keyspace) => {
                todo!()
            }
            DdlStatement::CreateTableStatement(_create_table) => {
                todo!()
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

    /// Maneja una declaración DML.
    fn handle_dml_statement(&self, dml_statement: DmlStatement) -> Vec<Byte> {
        let res: Result<Vec<Byte>> = match dml_statement {
            DmlStatement::SelectStatement(select) => {
                DiskHandler::do_select(select, &self.storage_addr)
            }
            DmlStatement::InsertStatement(insert) => {
                DiskHandler::do_insert(insert, &self.storage_addr)
            }
            DmlStatement::UpdateStatement(_update) => {
                todo!()
            }
            DmlStatement::DeleteStatement(_delete) => {
                todo!()
            }
            DmlStatement::BatchStatement(_batch) => {
                todo!()
            }
        };
        match res {
            Ok(value) => value,
            Err(err) => err.as_bytes(),
        }
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.endpoint_state.eq(&other.endpoint_state)
    }
}
