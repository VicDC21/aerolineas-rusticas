//! Módulo de nodos.

use crate::parser::{
    data_types::keyspace_name::KeyspaceName,
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
    },
};
use crate::protocol::{
    aliases::{results::Result, types::Byte},
    errors::error::Error,
    headers::{flags::Flag, length::Length, opcode::Opcode, stream::Stream, version::Version},
    messages::responses::result_kinds::ResultKind,
    traits::Byteable,
};
use crate::server::{modes::ConnectionMode, utils::load_json};

use serde::{Deserialize, Serialize};

use std::{collections::HashMap, net::TcpStream, path::Path, thread::JoinHandle};

use super::{
    actions::opcode::SvAction,
    addr::loader::AddrLoader,
    disk_operations::disk_handler::DiskHandler,
    internal_threads::{beater, create_client_and_private_conexion, gossiper},
    keyspace_metadata::keyspace::Keyspace,
    port_type::PortType,
    session_handler::get_partition_value_from_insert,
    states::{
        appstatus::AppStatus,
        endpoints::EndpointState,
        heartbeat::{GenType, VerType},
    },
    table_metadata::table::Table,
    utils::{divide_range, hash_value, send_to_node},
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
pub const N_NODES: Byte = 5;
/// El límite posible para los rangos de los nodos.
pub const NODES_RANGE_END: u64 = 18446744073709551615;

/// Un nodo es una instancia de parser que se conecta con otros nodos para procesar _queries_.
#[derive(Serialize, Deserialize)]
pub struct Node {
    /// El ID del nodo mismo.
    id: NodeId,

    /// Los estados de los nodos vecinos, incluyendo este mismo.
    ///
    /// No necesariamente serán todos los otros nodos del grafo, sólo los que este nodo conoce.
    #[serde(skip)]
    pub neighbours_states: NodesMap,

    /// Estado actual del nodo.
    #[serde(skip)]
    pub endpoint_state: EndpointState,

    /// Dirección de almacenamiento en disco.
    #[serde(skip)]
    pub storage_addr: String,

    /// Nombre del keyspace por defecto.
    pub default_keyspace_name: String,

    /// Nombre del keyspace por defecto de cada usuario.
    pub users_default_keyspace_name: HashMap<String, String>,

    /// Los keyspaces que tiene el nodo.
    /// (nombre, keyspace)
    pub keyspaces: HashMap<String, Keyspace>,

    /// Las tablas que tiene el nodo.
    /// (nombre, tabla)
    tables: HashMap<String, Table>,

    /// Rangos asignados a cada nodo para determinar la partición de los datos.
    #[serde(skip)]
    nodes_ranges: Vec<(u64, u64)>,

    /// Nombre de la tabla y los valores de las _partitions keys_ que contiene
    pub tables_and_partitions_keys_values: HashMap<String, Vec<String>>,

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
            if id == node_id {
                continue;
            }
            if send_to_node(
                node_id,
                SvAction::SendEndpointState(id).as_bytes(),
                PortType::Priv,
            )
            .is_err()
            {
                println!(
                    "El nodo {} se encontró apagado cuando el nodo {} intentó presentarse.",
                    id, node_id,
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

    /// Obtiene una tabla dado su nombre.
    pub fn get_table(&self, table_name: &str) -> Result<&Table> {
        match self.tables.get(table_name) {
            Some(table) => Ok(table),
            None => Err(Error::ServerError(format!(
                "La tabla llamada {} no existe",
                table_name
            ))),
        }
    }

    /// Responde si una tabla existe o no dado su nombre.
    pub fn table_exists(&self, table_name: &str) -> bool {
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
    pub fn get_keyspace_from_name(&self, keyspace_name: &str) -> Result<&Keyspace> {
        match self.keyspaces.get(keyspace_name) {
            Some(keyspace) => Ok(keyspace),
            None => Err(Error::ServerError(format!(
                "El keyspace `{}` no existe",
                keyspace_name
            ))),
        }
    }

    /// Respuesta si un keyspace existe o no dado su nombre.
    pub fn keyspace_exists(&self, keyspace_name: &str) -> bool {
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

    /// Obtiene el nombre del keyspace por defecto. Devuelve error si no se ha seleccionado uno.
    pub fn get_default_keyspace_name(&self) -> Result<String> {
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
    pub fn choose_available_keyspace_name(
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
    pub fn get_nodes_ids() -> Vec<NodeId> {
        let mut nodes_ids: Vec<NodeId> = AddrLoader::default_loaded().get_ids();
        nodes_ids.sort();
        nodes_ids
    }

    /// Selecciona un ID de nodo conforme al _hashing_ del valor del _partition key_ y los rangos de los nodos.
    pub fn select_node(&self, value: &str) -> NodeId {
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

    /// Compara si el _heartbeat_ de un nodo es más nuevo que otro.
    pub fn is_newer(&self, other: &Self) -> bool {
        self.endpoint_state.is_newer(&other.endpoint_state)
    }

    /// Envia su endpoint state al nodo del ID correspondiente.
    pub fn send_endpoint_state(&self, id: NodeId) {
        if send_to_node(
            id,
            SvAction::NewNeighbour(self.get_endpoint_state().clone()).as_bytes(),
            PortType::Priv,
        )
        .is_err()
        {
            println!(
                "El nodo {} se encontró apagado cuando el nodo {} intentó presentarse.",
                id, self.id,
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
    pub fn mode(&self) -> &ConnectionMode {
        self.endpoint_state.get_appstate().get_mode()
    }

    /// Agrega un nuevo vecino conocido por el nodo.
    pub fn add_neighbour_state(&mut self, state: EndpointState) -> Result<()> {
        let guessed_id = AddrLoader::default_loaded().get_id(state.get_addr())?;
        if !self.has_endpoint_state_by_id(&guessed_id) {
            self.neighbours_states.insert(guessed_id, state);
        }
        Ok(())
    }

    /// Actualiza la información de vecinos con otro mapa dado.
    ///
    /// No se comprueba si las entradas nuevas son más recientes o no: reemplaza todo sin preguntar.
    pub fn update_neighbours(&mut self, new_neighbours: NodesMap) -> Result<()> {
        for (node_id, endpoint_state) in new_neighbours {
            self.neighbours_states.insert(node_id, endpoint_state);
        }

        Ok(())
    }

    /// Actualiza el estado del nodo recibido a _Offline_.
    pub fn acknowledge_offline_neighbour(&mut self, node_id: NodeId) {
        if let Some(endpoint_state) = self.neighbours_states.get_mut(&node_id) {
            endpoint_state.set_appstate_status(AppStatus::Offline);
        }
    }

    /// Consulta el estado de _heartbeat_.
    pub fn get_beat(&self) -> (GenType, VerType) {
        self.endpoint_state.get_heartbeat().as_tuple()
    }

    /// Avanza el tiempo para el nodo.
    pub fn beat(&mut self) -> VerType {
        self.endpoint_state.beat();
        self.neighbours_states
            .insert(self.id, self.endpoint_state.clone());
        self.get_beat().1
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

    /// Maneja una declaración DDL interna.
    pub fn handle_internal_ddl_statement(
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

    /// Procesa una declaración USE interna.
    pub fn process_internal_use_statement(
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

    /// Procesa una declaración CREATE KEYSPACE interna.
    pub fn process_internal_create_keyspace_statement(
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

    /// Procesa una declaración ALTER KEYSPACE interna.
    pub fn process_internal_alter_keyspace_statement(
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

    /// Procesa una declaración DROP KEYSPACE interna.
    pub fn process_internal_drop_keyspace_statement(
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

    /// Procesa una declaración CREATE TABLE interna.
    pub fn process_internal_create_table_statement(
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

    /// Maneja una declaración DML interna.
    pub fn handle_internal_dml_statement(
        &mut self,
        dml_statement: DmlStatement,
        internal_metadata: (Option<i64>, Option<Byte>),
    ) -> Result<Vec<Byte>> {
        let node_number = get_node_replica_number_from_internal_metadata(internal_metadata)?;
        match dml_statement {
            DmlStatement::SelectStatement(select) => self.process_select(&select, node_number),
            DmlStatement::InsertStatement(insert) => {
                let timestamp = get_timestamp_from_internal_metadata(internal_metadata)?;
                self.process_insert(&insert, timestamp, node_number)
            }
            DmlStatement::UpdateStatement(update) => {
                let timestamp = get_timestamp_from_internal_metadata(internal_metadata)?;
                self.process_update(&update, timestamp, node_number)
            }
            DmlStatement::DeleteStatement(delete) => self.process_delete(&delete, node_number),
            DmlStatement::BatchStatement(_batch) => todo!(),
        }
    }

    /// Procesa una declaración SELECT.
    pub fn process_select(&self, select: &Select, node_id: Byte) -> Result<Vec<Byte>> {
        let table = match self.get_table(&select.from.get_name()) {
            Ok(table) => table,
            Err(err) => return Err(err),
        };
        // SIEMPRE ANTES DE UN DISKHANDLER HACER UN LOCK/WRITE
        let mut res = DiskHandler::do_select(
            select,
            &self.storage_addr,
            table,
            &self.get_default_keyspace_name()?,
            node_id,
        )?;

        Ok(Self::create_result_select(&mut res))
    }

    /// Crea un result de tipo select.
    pub fn create_result_select(res: &mut Vec<Byte>) -> Vec<Byte> {
        let mut response: Vec<Byte> = Vec::new();
        response.append(&mut Version::ResponseV5.as_bytes());
        response.append(&mut Flag::Default.as_bytes());
        response.append(&mut Stream::new(0).as_bytes());
        response.append(&mut Opcode::Result.as_bytes());
        response.append(&mut Length::new(res.len() as u32).as_bytes());
        response.append(res);
        response
    }

    /// Procesa una declaración INSERT.
    pub fn process_insert(
        &mut self,
        insert: &Insert,
        timestamp: i64,
        node_number: Byte,
    ) -> Result<Vec<Byte>> {
        println!(
            "Inserta internamente siendo el nodo {} a la tabla del nodo {}",
            self.id, node_number
        );
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
        let partition_value = get_partition_value_from_insert(insert, table)?;
        match self.check_if_has_new_partition_value(partition_value, &insert.get_table_name())? {
            Some(new_partition_values) => self
                .tables_and_partitions_keys_values
                .insert(insert.table.get_name().to_string(), new_partition_values),
            None => None,
        };
        Ok(Self::create_result_void())
    }

    /// Revisa si no tiene el partition value recibido, para el nombre de tabla dado.
    /// Si no lo tiene, lo agrega y lo devuelve junto al resto. Caso contrario, devuelve None.
    pub fn check_if_has_new_partition_value(
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
            Ok(Some(partition_values))
        } else {
            Ok(None)
        }
    }

    /// Procesa una declaración UPDATE.
    pub fn process_update(
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

    /// Procesa una declaración DELETE.
    pub fn process_delete(&mut self, delete: &Delete, node_number: Byte) -> Result<Vec<Byte>> {
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

    /// Crea un result de tipo void.
    pub fn create_result_void() -> Vec<Byte> {
        let mut response: Vec<Byte> = Vec::new();
        response.append(&mut Version::ResponseV5.as_bytes());
        response.append(&mut Flag::Default.as_bytes());
        response.append(&mut Stream::new(0).as_bytes());
        response.append(&mut Opcode::Result.as_bytes());
        response.append(&mut Length::new(4).as_bytes());
        response.append(&mut ResultKind::Void.as_bytes());
        response
    }

    /// Obtiene los valores de las _partition keys_ de una tabla dado su nombre.
    pub fn get_partition_keys_values(&self, table_name: &String) -> Result<&Vec<String>> {
        match self.tables_and_partitions_keys_values.get(table_name) {
            Some(partitions_keys_to_nodes) => Ok(partitions_keys_to_nodes),
            None => Err(Error::ServerError(
                "La tabla indicada no existe".to_string(),
            )),
        }
    }

    /// Dado el nombre de una tabla, obtiene la cantidad de replicación del keyspace al que pertenece.
    pub fn get_replicas_from_table_name(&self, table_name: &str) -> Result<u32> {
        let keyspace = self.get_keyspace(table_name)?;
        match keyspace.simple_replicas() {
            Some(replication_factor) => Ok(replication_factor),
            None => Err(Error::ServerError("No es una simple strategy".to_string())),
        }
    }

    /// Obtiene la cantidad de filas de un result.
    pub fn get_quantity_of_rows(
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

    /// Obtiene la cantidad de columnas de un result, que se encuentra en su metadata.
    pub fn get_columns_metadata_length(&self, results_from_another_nodes: &[Byte]) -> usize {
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

    /// Obtiene la cantidad de replicas de un keyspace, dado su nombre.
    /// Devuelve error si no se usa una estrategia de replicación simple.
    pub fn get_quantity_of_replicas_from_keyspace(&self, keyspace: &Keyspace) -> Result<u32> {
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

fn get_node_replica_number_from_internal_metadata(
    internal_metadata: (Option<i64>, Option<u8>),
) -> Result<u8> {
    let node_number = match internal_metadata.1 {
        Some(value) => value,
        None => {
            return Err(Error::ServerError(
                "No se paso la informacion del nodo en la metadata interna".to_string(),
            ))
        }
    };
    Ok(node_number)
}

fn get_timestamp_from_internal_metadata(
    internal_metadata: (Option<i64>, Option<u8>),
) -> Result<i64> {
    let timestamp = match internal_metadata.0 {
        Some(value) => value,
        None => {
            return Err(Error::ServerError(
                "No se paso la informacion del timestamp en la metadata interna".to_string(),
            ))
        }
    };
    Ok(timestamp)
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.endpoint_state.eq(&other.endpoint_state)
    }
}
