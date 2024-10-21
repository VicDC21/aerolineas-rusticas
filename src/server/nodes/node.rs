//! Módulo de nodos.

use std::{
    cmp::PartialEq,
    collections::{HashMap, HashSet},
    fs::{create_dir, OpenOptions},
    io::{BufRead, BufReader, Read, Seek, SeekFrom, Write},
    net::TcpListener,
    path::Path,
};

use crate::parser::{
    main_parser::make_parse,
    statements::{
        ddl_statement::ddl_statement_parser::DdlStatement,
        dml_statement::{dml_statement_parser::DmlStatement, main_statements::insert::Insert},
        statement::Statement,
    },
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
        states::{
            appstatus::AppStatus,
            endpoints::EndpointState,
            heartbeat::{
                HeartbeatState, {GenType, VerType},
            },
        },
        utils::send_to_node,
    },
};
use crate::tokenizer::tokenizer::tokenize_query;

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
        let storage_addr = Self::new_node_storage(id);

        Self {
            id,
            neighbours_states: NodesMap::new(),
            endpoint_state: EndpointState::with_id_and_mode(id, mode),
            storage_addr,
        }
    }

    /// Crea una carpeta de almacenamiento para el nodo.
    /// Devuelve la ruta a dicho almacenamiento.
    fn new_node_storage(id: NodeId) -> String {
        let path_folder = Path::new("storage");
        if !path_folder.exists() && !path_folder.is_dir() {
            create_dir(path_folder).expect("No se pudo crear la carpeta de almacenamiento");
        }
        let storage_addr: String = format!("storage/storage_node_{}", id);
        let path_folder = Path::new(&storage_addr);
        if !path_folder.exists() && !path_folder.is_dir() {
            create_dir(path_folder)
                .expect("No se pudo crear la carpeta de almacenamiento del nodo");
        }
        storage_addr
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
        self.neighbours_states.contains_key(&state.guess_id())
    }

    fn add_neighbour_state(&mut self, state: EndpointState) {
        if !self.has_endpoint_state(&state) {
            self.neighbours_states.insert(state.guess_id(), state);
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

        if let Err(err) = send_to_node(receptor_id, SvAction::Ack2(nodes_for_receptor).as_bytes()) {
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

    /// Escucha por los eventos que recibe.
    pub fn listen(&mut self) -> Result<()> {
        let listener = match TcpListener::bind(self.endpoint_state.get_addr()) {
            Ok(tcp_listener) => tcp_listener,
            Err(_) => {
                return Err(Error::ServerError(format!(
                    "No se pudo bindear a la dirección '{}'",
                    self.endpoint_state.get_addr()
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
                Ok(tcp_stream) => {
                    let bytes: Vec<Byte> = tcp_stream.bytes().flatten().collect();
                    match SvAction::get_action(&bytes[..]) {
                        Some(action) => match self.handle_sv_action(action) {
                            Ok(continue_loop) => {
                                if !continue_loop {
                                    break;
                                }
                            }
                            Err(err) => {
                                println!(
                                    "[{} - ACTION] Error en la acción del servidor: {}",
                                    self.id, err
                                );
                            }
                        },
                        None => {
                            if let Ok(query) = String::from_utf8(bytes) {
                                match self.mode() {
                                    ConnectionMode::Echo => {
                                        println!("[{} - ECHO] {}", self.id, query)
                                    }
                                    ConnectionMode::Parsing => {
                                        if let Err(err) = self.handle_query(query) {
                                            println!(
                                                "[{} - PARSING] Error en el query recibido: {}",
                                                self.id, err
                                            );
                                        }
                                    }
                                }
                            } else {
                                println!(
                                    "[{} - PARSING] Error en el query recibido: no es UTF-8",
                                    self.id
                                );
                            }
                        }
                    }
                }
            }
        }

        Ok(())
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
        }
    }

    /// Maneja un query.
    fn handle_query(&self, query: String) -> Result<()> {
        match make_parse(&mut tokenize_query(&query)) {
            Ok(statement) => match statement {
                Statement::DdlStatement(ddl_statement) => {
                    self.handle_ddl_statement(ddl_statement);
                }
                Statement::DmlStatement(dml_statement) => {
                    self.handle_dml_statement(dml_statement)?;
                }
                Statement::UdtStatement(_udt_statement) => {
                    todo!();
                }
            },
            Err(err) => {
                return Err(Error::ServerError(format!(
                    "[{} - PARSING] Error en el query tokenizado: {}",
                    self.id, err
                )));
            }
        }
        Ok(())
    }

    /// Maneja una declaración DDL.
    fn handle_ddl_statement(&self, ddl_statement: DdlStatement) {
        match ddl_statement {
            DdlStatement::UseStatement(_keyspace_name) => {}
            DdlStatement::CreateKeyspaceStatement(_create_keyspace) => {}
            DdlStatement::AlterKeyspaceStatement(_alter_keyspace) => {}
            DdlStatement::DropKeyspaceStatement(_drop_keyspace) => {}
            DdlStatement::CreateTableStatement(_create_table) => {}
            DdlStatement::AlterTableStatement(_alter_table) => {}
            DdlStatement::DropTableStatement(_drop_table) => {}
            DdlStatement::TruncateStatement(_truncate) => {}
        }
    }

    /// Maneja una declaración DML.
    fn handle_dml_statement(&self, dml_statement: DmlStatement) -> Result<()> {
        match dml_statement {
            DmlStatement::SelectStatement(_select) => {}
            DmlStatement::InsertStatement(insert) => {
                self.do_insert(insert)?;
            }
            DmlStatement::UpdateStatement(_update) => {}
            DmlStatement::DeleteStatement(_delete) => {}
            DmlStatement::BatchStatement(_batch) => {}
        }
        Ok(())
    }

    fn do_insert(&self, statement: Insert) -> Result<()> {
        let keyspace = statement.table_name.get_keyspace();
        let name = statement.table_name.get_name();
        let table_addr = match keyspace {
            Some(keyspace) => format!("{}/{}/{}.csv", self.storage_addr, keyspace, name),
            None => format!("{}/{}.csv", self.storage_addr, name),
        };

        let file = OpenOptions::new()
            .read(true)
            .open(&table_addr)
            .map_err(|e| Error::ServerError(e.to_string()))?;
        let mut reader = BufReader::new(&file);

        let mut line = String::new();
        let read_bytes = reader
            .read_line(&mut line)
            .map_err(|e| Error::ServerError(e.to_string()))?;
        if read_bytes == 0 {
            return Err(Error::ServerError(format!(
                "No se pudo leer la tabla con ruta {}",
                &table_addr
            )));
        }
        line = line.trim().to_string();

        let query_cols = statement.get_columns_names();
        let table_cols: Vec<&str> = line.split(",").collect();
        for col in &query_cols {
            if !table_cols.contains(&col.as_str()) {
                return Err(Error::ServerError(format!(
                    "La tabla con ruta {} no contiene la columna {}",
                    &table_addr, col
                )));
            }
        }

        let values = statement.get_values();
        let mut id_exists = false;
        let mut buffer = Vec::new();
        let mut position = 0;
        // Leo línea por línea y verifico si el ID de la fila ya existe
        while let Some(Ok(line)) = reader.by_ref().lines().next() {
            if line.starts_with(&values[0]) {
                id_exists = true;
                break;
            }
            position += line.len() + 1; // Actualizo la posicion a sobreescribir si existe el ID
            buffer.push(line);
        }
        // Si el ID existe y no se debe sobreescribir la línea, no hago nada.
        if id_exists && statement.if_not_exists {
            return Ok(());
        }

        // Abro el archivo nuevamente para escribir
        let mut writer = OpenOptions::new()
            .write(true)
            .open(&table_addr)
            .map_err(|e| Error::ServerError(e.to_string()))?;

        let new_row = Self::generate_row_to_insert(&values, &query_cols, &table_cols);
        if id_exists {
            // Si el ID ya existia, sobrescribo la linea
            writer
                .seek(SeekFrom::Start(position as u64))
                .map_err(|e| Error::ServerError(e.to_string()))?;
            writer
                .write_all(new_row.as_bytes())
                .map_err(|e| Error::ServerError(e.to_string()))?;
        } else {
            // Si no existia el ID, escribo al final del archivo
            writer
                .seek(SeekFrom::End(0))
                .map_err(|e| Error::ServerError(e.to_string()))?;
            writer
                .write_all(new_row.as_bytes())
                .map_err(|e| Error::ServerError(e.to_string()))?;
        }

        Ok(())
    }

    fn generate_row_to_insert(
        values: &[String],
        query_cols: &[String],
        table_cols: &[&str],
    ) -> String {
        let mut values_to_insert: Vec<&str> = vec![""; table_cols.len()];

        for i in 0..query_cols.len() {
            if let Some(j) = table_cols.iter().position(|c| *c == query_cols[i]) {
                values_to_insert[j] = values[i].as_str();
            }
        }

        values_to_insert.join(",") + "\n"
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.endpoint_state.eq(&other.endpoint_state)
    }
}
