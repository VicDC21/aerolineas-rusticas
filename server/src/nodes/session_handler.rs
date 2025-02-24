//! Módulo para el manejo de una sesión del cliente, para decidir el bloqueo o no de un nodo.

use {
    crate::{
        cql_frame::query_body::QueryBody,
        modes::ConnectionMode,
        nodes::{
            actions::opcode::{GossipInfo, SvAction},
            disk_operations::disk_handler::DiskHandler,
            node::{Node, NodeId, NodesMap /*N_NODES*/},
            port_type::PortType,
            states::{appstatus::AppStatus, endpoints::EndpointState, heartbeat::HeartbeatState},
            table_metadata::table::Table,
            utils::{
                hash_value, next_node_in_the_cluster, send_to_node,
                send_to_node_and_wait_response_with_timeout,
            },
        },
        utils::printable_bytes,
    },
    chrono::Utc,
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
    protocol::{
        aliases::{
            results::Result,
            types::{Byte, Long, Uint, Ulong},
        },
        errors::error::Error,
        headers::{
            flags::Flag, length::Length, msg_headers::Headers, opcode::Opcode, stream::Stream,
            version::Version,
        },
        notations::consistency::Consistency,
        traits::Byteable,
        utils::{parse_bytes_to_string, parse_bytes_to_string_map},
    },
    std::{
        collections::{HashMap, HashSet},
        io::{Read, Write},
        sync::{Arc, RwLock},
    },
    tokenizer::tok::tokenize_query,
};

/// El tiempo de espera _(en segundos)_ por una respuesta.
pub const TIMEOUT_SECS: Ulong = 1;

/// Se encarga de procesar todo lo relacionado a una sesión de un cliente.
///
/// Guarda una referencia compartida a un nodo, con la posibilidad de decidir si se quiere
/// lockear o no al nodo durante las operaciones que correspondan.
pub struct SessionHandler {
    /// ID del nodo.
    pub id: NodeId,
    /// Referencia compartida del lock conteniendo al nodo.
    pub lock: Arc<RwLock<Node>>,
}

impl SessionHandler {
    /// Crea un nuevo `SessionHandler` con un nodo específico.
    pub fn new(id: NodeId, node: Node) -> Self {
        SessionHandler {
            id,
            lock: Arc::new(RwLock::new(node)),
        }
    }

    /// Accede al nodo para escritura mutable, es lockeado para otros.
    fn write(&self) -> Result<std::sync::RwLockWriteGuard<Node>> {
        match self.lock.write() {
            Ok(guard) => Ok(guard),
            Err(poisoned) => {
                /*let err = Err(Error::ServerError(format!(
                    "Lock envenenado desde nodo con ID {} para escritura: {}",
                    self.id, &poisoned
                )));
                let _unused: std::sync::RwLockWriteGuard<'_, Node> = poisoned.into_inner();
                err*/
                println!(
                    "Lock envenenado detectado desde el nodo con ID {} para escritura: {}",
                    self.id, &poisoned
                );
                self.lock.clear_poison();

                let unpoisoned_guard = poisoned.into_inner();
                Ok(unpoisoned_guard)
            }
        }
    }

    /// Accede al nodo para lectura inmutable, sigue siendo accesible para otros.
    fn read(&self) -> Result<std::sync::RwLockReadGuard<Node>> {
        match self.lock.read() {
            Ok(guard) => Ok(guard),
            Err(poisoned) => {
                println!(
                    "Lock envenenado detectado desde el nodo con ID {} para lectura: {}",
                    self.id, &poisoned
                );

                self.lock.clear_poison();
                let unpoisoned_guard = poisoned.into_inner();
                Ok(unpoisoned_guard)
            }
        }
    }

    // ##########################################################################################
    // ################################ PROCESAMIENTO DEL STREAM ################################
    // ##########################################################################################

    /// Procesa una _request_ en forma de [Byte]s.
    /// También devuelve un [bool] indicando si se debe parar el hilo.
    pub fn process_stream<S>(
        &self,
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
    fn handle_sv_action<S>(&self, action: SvAction, mut tcp_stream: S) -> Result<bool>
    where
        S: Read + Write,
    {
        let mut stop = false;
        match action {
            SvAction::Exit => stop = true, // La comparación para salir ocurre en otro lado
            SvAction::Beat => {
                self.write()?.beat();
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
            SvAction::NewNeighbour(id, state) => {
                self.write()?.add_neighbour_state(id, state)?;
            }
            SvAction::SendEndpointState(id, ip) => {
                self.read()?.send_endpoint_state(id, ip);
            }
            SvAction::InternalQuery(bytes) => {
                let response = self.handle_request(&bytes, true, true);
                let _ = tcp_stream.write_all(&response[..]);
                if let Err(err) = tcp_stream.flush() {
                    return Err(Error::ServerError(err.to_string()));
                };
            }
            SvAction::StoreMetadata => {
                if let Err(err) = DiskHandler::store_node_metadata(self.write()?) {
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
                let node_reader = self.read()?;
                let table = node_reader.get_table(&table_name)?;
                let has_partition_value = node_reader.check_if_has_new_partition_value(
                    partition_value,
                    &table.get_name().to_string(),
                )?;
                drop(node_reader);
                match has_partition_value {
                    Some(new_partition_values) => self
                        .write()?
                        .tables_and_partitions_keys_values
                        .insert(table_name, new_partition_values),
                    None => None,
                };
            }
            SvAction::SendMetadata(node_id) => {
                let response = self.read()?.get_metadata_to_new_node_as_bytes()?;
                send_to_node(
                    node_id,
                    SvAction::ReceiveMetadata(response).as_bytes(),
                    PortType::Priv,
                )?;
            }
            SvAction::ReceiveMetadata(metadata) => {
                let mut node_writer = self.write()?;
                node_writer.receive_metadata(metadata)?;
                // Además continuamos el proceso de adaptación del clúster.
                node_writer.create_necessary_dirs_and_csvs()?;
                // node_writer.endpoint_state.set_appstate_status(AppStatus::NewNode);
                node_writer.notify_update_replicas(false)?;
                node_writer
                    .endpoint_state
                    .set_appstate_status(AppStatus::RelocationIsNeeded);
            }
            SvAction::RelocationNeeded => self.write()?.relocation_needed(),
            SvAction::UpdateReplicas(node_id, is_deletion) => {
                self.write()?.update_node_replicas(node_id, is_deletion)?;
            }
            SvAction::AddRelocatedRows(node_id, rows) => {
                self.write()?.add_relocated_rows(node_id, rows)?
            }
            SvAction::GetAllTablesOfReplica(node_id, only_farthest_replica) => {
                let res = self.read()?.copy_tables(node_id, only_farthest_replica)?;
                let _ = tcp_stream.write_all(&res);
                if let Err(err) = tcp_stream.flush() {
                    return Err(Error::ServerError(err.to_string()));
                };
            }
            SvAction::DeleteNode => {
                let mut node_writer = self.write()?;
                node_writer.node_to_deletion()?;
                node_writer.notify_update_replicas(true)?;
            }
            SvAction::NodeIsLeaving(node_id) => {
                self.write()?.node_leaving(node_id, AppStatus::Left)?;
            }
            SvAction::NodeDeleted(node_id) => {
                self.write()?.node_leaving(node_id, AppStatus::Remove)?;
            }
            SvAction::NodeToDelete(node_id) => {
                self.read()?.notify_node_is_gonna_be_deleted(node_id)?;
            }
        };
        Ok(stop)
    }

    fn match_kind_of_conection_mode<S>(
        &self,
        bytes: Vec<Byte>,
        mut stream: S,
        is_logged: bool,
    ) -> Result<Vec<Byte>>
    where
        S: Read + Write,
    {
        let node_reader = self.read()?;
        let mode = node_reader.mode().clone();
        drop(node_reader);
        match mode {
            ConnectionMode::Echo => {
                println!("[{} - ECHO] {}", self.id, printable_bytes(&bytes));
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

    // ###########################################################################################
    // ################################ PROCESAMIENTO DE REQUESTS ################################
    // ###########################################################################################

    /// Maneja una request.
    fn handle_request(
        &self,
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
        &self,
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
                            read_metadata_from_internal_request(internal_metadata);
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

    fn handle_auth_response(&self, request: &[Byte], lenght: &Length) -> Result<Vec<Byte>> {
        let req: &[Byte] = &request[9..(lenght.len as usize) + 9];
        let node_reader = self.read()?;
        let users = DiskHandler::read_admitted_users(&node_reader.storage_addr)?;
        drop(node_reader);

        let mut response: Vec<Byte> = Vec::new();
        let mut i = 0;
        let user_from_req = parse_bytes_to_string(req, &mut i)?;
        let password_from_req = parse_bytes_to_string(&req[i..], &mut i)?;
        let mut node_writer = self.write()?;
        for user in users {
            if user.0 == user_from_req && user.1 == password_from_req {
                response.append(&mut Version::ResponseV5.as_bytes());
                response.append(&mut Flag::Default.as_bytes());
                response.append(&mut Stream::new(0).as_bytes());
                response.append(&mut Opcode::AuthSuccess.as_bytes());
                response.append(&mut Length::new(0).as_bytes());
                // REVISAR AL TESTEAR
                if !node_writer
                    .users_default_keyspace_name
                    .contains_key(&user.0)
                {
                    node_writer
                        .users_default_keyspace_name
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

    // ###########################################################################################
    // ############################### PROCESAMIENTO DE STATEMENTS ###############################
    // ###########################################################################################

    fn handle_statement(
        &self,
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
            Statement::Startup => Err(Error::Invalid(
                "No se deberia haber mandado el startup por este canal".to_string(),
            )),
            Statement::LoginUser(_) => Err(Error::Invalid(
                "No se deberia haber mandado el login por este canal".to_string(),
            )),
        }
    }

    // ###########################################################################################
    // ##################################### DDL STATEMENTS ######################################
    // ###########################################################################################

    fn handle_ddl_statement(
        &self,
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
            DdlStatement::AlterTableStatement(_alter_table) => Err(Error::Invalid(
                "Alter Table Statement no está soportado.".to_string(),
            )),
            DdlStatement::DropTableStatement(_drop_table) => Err(Error::Invalid(
                "Drop Table Statement no está soportado.".to_string(),
            )),
            DdlStatement::TruncateStatement(_truncate) => Err(Error::Invalid(
                "Truncate Statement no está soportado.".to_string(),
            )),
        }
    }

    fn process_use_statement(
        &self,
        keyspace_name: KeyspaceName,
        request: &[Byte],
    ) -> Result<Vec<Byte>> {
        let mut response: Vec<Byte> = Vec::new();
        let mut actual_node_id = self.id;
        let node_reader = self.read()?;
        let nodes_ids = node_reader.get_nodes_ids();
        let nodes_quantity = node_reader.get_actual_n_nodes();
        drop(node_reader);
        for _ in 0..nodes_quantity {
            response = if actual_node_id != self.id {
                send_to_node_and_wait_response_with_timeout(
                    actual_node_id,
                    SvAction::InternalQuery(request.to_vec()).as_bytes(),
                    PortType::Priv,
                    true,
                    Some(TIMEOUT_SECS),
                )?
            } else {
                let mut node_writer = self.write()?;
                node_writer.process_internal_use_statement(&keyspace_name)?
            };
            actual_node_id = next_node_in_the_cluster(actual_node_id, &nodes_ids);
        }
        Ok(response)
    }

    fn process_create_keyspace_statement(
        &self,
        create_keyspace: CreateKeyspace,
        request: &[Byte],
    ) -> Result<Vec<Byte>> {
        let mut response: Vec<Byte> = Vec::new();
        let mut actual_node_id = self.id;
        let node_reader = self.read()?;
        let nodes_ids = node_reader.get_nodes_ids();
        let nodes_quantity = node_reader.get_actual_n_nodes();
        drop(node_reader);
        for _ in 0..nodes_quantity {
            response = if actual_node_id != self.id {
                send_to_node_and_wait_response_with_timeout(
                    actual_node_id,
                    SvAction::InternalQuery(request.to_vec()).as_bytes(),
                    PortType::Priv,
                    true,
                    Some(TIMEOUT_SECS),
                )?
            } else {
                let mut node_writer = self.write()?;
                node_writer.process_internal_create_keyspace_statement(&create_keyspace)?
            };
            actual_node_id = next_node_in_the_cluster(actual_node_id, &nodes_ids);
        }
        Ok(response)
    }

    fn process_alter_statement(
        &self,
        alter_keyspace: AlterKeyspace,
        request: &[Byte],
    ) -> Result<Vec<Byte>> {
        let keyspace_name = alter_keyspace.name.get_name();
        let node_reader = self.read()?;
        if !node_reader.keyspaces.contains_key(keyspace_name) && !alter_keyspace.if_exists {
            return Err(Error::ServerError(format!(
                "El keyspace {} no existe",
                keyspace_name
            )));
        }
        let nodes_quantity = node_reader.get_actual_n_nodes();
        let nodes_ids = node_reader.get_nodes_ids();
        drop(node_reader);
        let mut responses = Vec::new();
        let mut actual_node_id = self.id;
        for _ in 0..nodes_quantity {
            let response = if actual_node_id != self.id {
                send_to_node_and_wait_response_with_timeout(
                    actual_node_id,
                    SvAction::InternalQuery(request.to_vec()).as_bytes(),
                    PortType::Priv,
                    true,
                    Some(TIMEOUT_SECS),
                )?
            } else {
                let mut node_writer = self.write()?;
                node_writer.process_internal_alter_keyspace_statement(&alter_keyspace)?
            };
            responses.push(response);
            actual_node_id = next_node_in_the_cluster(actual_node_id, &nodes_ids);
        }
        Ok(Node::create_result_void())
    }

    fn process_drop_keyspace_statement(
        &self,
        drop_keyspace: DropKeyspace,
        request: &[Byte],
    ) -> Result<Vec<Byte>> {
        let keyspace_name = drop_keyspace.name.get_name();
        let node_reader = self.read()?;
        if !node_reader.keyspaces.contains_key(keyspace_name) && !drop_keyspace.if_exists {
            return Err(Error::ServerError(format!(
                "El keyspace {} no existe",
                keyspace_name
            )));
        }
        let nodes_ids = node_reader.get_nodes_ids();
        let nodes_quantity = node_reader.get_actual_n_nodes();
        drop(node_reader);
        let mut responses = Vec::new();
        let mut actual_node_id = self.id;
        for _ in 0..nodes_quantity {
            let response = if actual_node_id != self.id {
                send_to_node_and_wait_response_with_timeout(
                    actual_node_id,
                    SvAction::InternalQuery(request.to_vec()).as_bytes(),
                    PortType::Priv,
                    true,
                    Some(TIMEOUT_SECS),
                )?
            } else {
                let mut node_writer = self.write()?;
                node_writer.process_internal_drop_keyspace_statement(&drop_keyspace)?
            };
            responses.push(response);
            actual_node_id = next_node_in_the_cluster(actual_node_id, &nodes_ids);
        }
        Ok(Node::create_result_void())
    }

    fn process_create_table_statement(
        &self,
        create_table: CreateTable,
        request: &[Byte],
    ) -> Result<Vec<Byte>> {
        let node_reader = self.read()?;
        let keyspace_name =
            node_reader.choose_available_keyspace_name(create_table.name.get_keyspace())?;
        let keyspace = node_reader.get_keyspace_from_name(&keyspace_name)?;
        let quantity_replicas: Uint =
            node_reader.get_quantity_of_replicas_from_keyspace(keyspace)?;
        let nodes_ids = node_reader.get_nodes_ids();
        drop(node_reader);
        let mut response: Vec<Byte> = Vec::new();
        for actual_node_id in &nodes_ids {
            let mut next_node_id = *actual_node_id;
            for _ in 0..quantity_replicas {
                response = if next_node_id == self.id {
                    let mut node_writer = self.write()?;
                    node_writer
                        .process_internal_create_table_statement(&create_table, *actual_node_id)?
                } else {
                    let request_with_metadata = add_metadata_to_internal_request_of_any_kind(
                        SvAction::InternalQuery(request.to_vec()).as_bytes(),
                        None,
                        Some(*actual_node_id),
                    );
                    send_to_node_and_wait_response_with_timeout(
                        next_node_id,
                        request_with_metadata,
                        PortType::Priv,
                        true,
                        Some(TIMEOUT_SECS),
                    )?
                };
                next_node_id = next_node_in_the_cluster(next_node_id, &nodes_ids);
            }
        }
        Ok(response)
    }

    // ##########################################################################################
    // ##################################### DML STATEMENTS #####################################
    // ##########################################################################################

    fn handle_dml_statement(
        &self,
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
        }
    }

    // ###########################################################################################
    // ######################################### SELECT ##########################################
    // ###########################################################################################
    fn select_with_other_nodes(
        &self,
        select: Select,
        request: &[Byte],
        consistency_level: &Consistency,
    ) -> Result<Vec<Byte>> {
        let table_name = select.from.get_name();
        let mut results_from_another_nodes: Vec<Byte> = Vec::new();
        let mut consulted_nodes: Vec<Byte> = Vec::new();
        let node_reader = self.read()?;
        let replication_factor_quantity = node_reader.get_replicas_from_table_name(&table_name)?;
        let consistency_number =
            consistency_level.as_usize(replication_factor_quantity as usize)?;
        let partitions_keys_to_nodes = node_reader.get_partition_keys_values(&table_name)?.clone(); // Tuve que agregar un clone para que no me tire error de referencia mutable e inmutable al mismo tiempo
        drop(node_reader);

        for partition_key_value in partitions_keys_to_nodes {
            let node_reader = self.read()?;
            let node_id = node_reader.select_node(&partition_key_value);
            drop(node_reader);

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
                match self.consult_replica_nodes_consistency(
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
        &self,
        node_id: NodeId,
        select_and_request: (&Select, &[Byte]),
        wait_response: bool,
        responsive_replica: &mut NodeId,
        replicas_asked: &mut usize,
        replication_factor_quantity: Uint,
    ) -> Result<Vec<Byte>> {
        let (select, request) = select_and_request;
        let actual_result = if node_id == self.id {
            let node_writer = self.write()?;
            node_writer.process_select(select, node_id)?
        } else {
            let request_with_metadata = add_metadata_to_internal_request_of_any_kind(
                SvAction::InternalQuery(request.to_vec()).as_bytes(),
                None,
                Some(node_id),
            );
            let mut result: Vec<Byte> = Vec::new();
            *responsive_replica = node_id;
            *replicas_asked = 0;
            if self.neighbour_is_responsive(node_id)? {
                result = match send_to_node_and_wait_response_with_timeout(
                    node_id,
                    request_with_metadata,
                    PortType::Priv,
                    wait_response,
                    Some(TIMEOUT_SECS),
                ) {
                    Ok(res) => res,
                    Err(err) => {
                        return Err(Error::ServerError(format!(
                            "Error al enviar la query al nodo {}: {}",
                            node_id, err
                        )));
                    }
                }
            }
            *replicas_asked += 1;

            // Si hubo error al enviar el mensaje y habia que esperar la respuesta, se asume que
            // el vecino está apagado, entonces se intenta con las replicas
            if result.is_empty() && wait_response {
                let mut node_writer = self.write()?;
                node_writer.acknowledge_offline_neighbour(node_id);
                drop(node_writer);

                result = self.forward_select_request_to_replicas(
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

    fn forward_select_request_to_replicas(
        &self,
        node_id: NodeId,
        select_and_request: (&Select, &[Byte]),
        wait_response: bool,
        responsive_replica: &mut NodeId,
        replicas_asked: &mut usize,
        replication_factor_quantity: Uint,
    ) -> Result<Vec<Byte>> {
        let (select, request) = select_and_request;
        let mut result: Vec<Byte> = Vec::new();
        let node_reader = self.read()?;
        let nodes_ids = node_reader.get_nodes_ids();
        drop(node_reader);
        let mut node_replica = next_node_in_the_cluster(node_id, &nodes_ids);

        for _ in 1..replication_factor_quantity {
            if self.neighbour_is_responsive(node_replica)? {
                let request_with_metadata = add_metadata_to_internal_request_of_any_kind(
                    SvAction::InternalQuery(request.to_vec()).as_bytes(),
                    None,
                    Some(node_id),
                );
                let replica_response = if node_replica == self.id {
                    let node_writer = self.write()?;
                    node_writer.process_select(select, node_id)?
                } else {
                    send_to_node_and_wait_response_with_timeout(
                        node_replica,
                        request_with_metadata,
                        PortType::Priv,
                        wait_response,
                        Some(TIMEOUT_SECS),
                    )?
                };
                *replicas_asked += 1;

                if replica_response.is_empty() && wait_response {
                    let mut node_writer = self.write()?;
                    node_writer.acknowledge_offline_neighbour(node_replica);
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

    fn decide_how_to_request_the_digest_read_request(
        &self,
        node_to_consult: Byte,
        request: &[Byte],
        node_id: Byte,
    ) -> Result<Vec<Byte>> {
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
            send_to_node_and_wait_response_with_timeout(
                node_to_consult,
                request_with_metadata,
                PortType::Priv,
                true,
                Some(TIMEOUT_SECS),
            )?
        };
        Ok(opcode_with_hashed_value)
    }

    fn get_digest_read_request_value(&self, opcode_with_hashed_value: &[Byte]) -> Result<Ulong> {
        if opcode_with_hashed_value.len() != 9 {
            // OpCode + Long
            return Err(Error::ServerError(
                "Se esperaba un vec de largo 9".to_string(),
            ));
        }
        let array = match opcode_with_hashed_value[1..9].try_into().ok() {
            Some(value) => value,
            None => {
                return Err(Error::ServerError(
                    "No se pudo transformar el vector a Long".to_string(),
                ))
            }
        };
        let res_hashed_value = Ulong::from_be_bytes(array);
        Ok(res_hashed_value)
    }

    // ###########################################################################################
    // ####################################### READ-REPAIR #######################################
    // ###########################################################################################

    /// Revisa si se cumple el _Consistency Level_ y además si es necesario ejecutar _read-repair_, si es el caso, lo ejecuta.
    ///
    /// Devuelve un booleano indicando si _read-repair_ fue ejecutado o no.
    fn consult_replica_nodes_consistency(
        &self,
        id_and_replicas_asked: (NodeId, usize),
        request_and_table_name: (&[Byte], &str),
        consistency_counter: &mut usize,
        consistency_number: usize,
        first_responsive_id_and_response: (NodeId, &[Byte]),
        replication_factor_quantity: Uint,
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
        let node_reader = self.read()?;
        let nodes_ids = node_reader.get_nodes_ids();
        drop(node_reader);
        let mut node_to_consult = next_node_in_the_cluster(responsive_replica, &nodes_ids);
        let mut inconsistent_digest_request = false;
        for _ in (replicas_asked as Uint)..replication_factor_quantity {
            let opcode_with_hashed_value = match self.decide_how_to_request_the_digest_read_request(
                node_to_consult,
                request,
                node_id,
            ) {
                Ok(res) => res,
                Err(err) => {
                    return Err(Error::ServerError(format!(
                        "Error al enviar la query al nodo {}: {}",
                        node_to_consult, err
                    )));
                }
            };
            if opcode_with_hashed_value.is_empty() {
                node_to_consult = next_node_in_the_cluster(node_to_consult, &nodes_ids);
                continue;
            }
            let res_hashed_value = self.get_digest_read_request_value(&opcode_with_hashed_value)?;
            self.check_consistency_of_the_responses(
                opcode_with_hashed_value,
                first_hashed_value,
                res_hashed_value,
                consistency_counter,
                &mut responses,
                &mut inconsistent_digest_request,
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
            inconsistent_digest_request,
        );
        if exec_read_repair && self.neighbour_is_responsive(node_id)? {
            return self.start_read_repair(
                node_id,
                request,
                table_name,
                replication_factor_quantity,
            );
        }
        Ok(false)
    }

    fn check_consistency_of_the_responses(
        &self,
        opcode_with_hashed_value: Vec<Byte>,
        first_hashed_value: Ulong,
        res_hashed_value: Ulong,
        consistency_counter: &mut usize,
        responses: &mut Vec<Vec<Byte>>,
        inconsistent_digest_request: &mut bool,
    ) -> Result<()> {
        if Opcode::try_from(opcode_with_hashed_value[0])? == Opcode::Result
            && first_hashed_value == res_hashed_value
        {
            *consistency_counter += 1;
            responses.push(opcode_with_hashed_value[1..].to_vec());
        } else {
            *inconsistent_digest_request = true
        };
        Ok(())
    }

    fn start_read_repair(
        &self,
        node_id: Byte,
        request: &[Byte],
        table_name: &str,
        replication_factor_quantity: Uint,
    ) -> Result<bool> {
        let mut rows_of_nodes: Vec<Vec<Vec<String>>> = vec![];
        let mut req_with_node_replica = request[9..].to_vec();
        req_with_node_replica.push(node_id);
        let node_reader = self.read()?;
        let nodes_ids = node_reader.get_nodes_ids();
        drop(node_reader);
        let mut node_to_consult = node_id;
        for _ in 0..replication_factor_quantity {
            let res = if node_to_consult == self.id {
                self.exec_direct_read_request(req_with_node_replica.clone())?
            } else {
                let extern_response = send_to_node_and_wait_response_with_timeout(
                    node_to_consult,
                    SvAction::DirectReadRequest(req_with_node_replica.clone()).as_bytes(),
                    PortType::Priv,
                    true,
                    Some(TIMEOUT_SECS),
                )?;
                create_utf8_string_from_bytes(extern_response)?
            };
            add_rows(res, &mut rows_of_nodes);
            node_to_consult = next_node_in_the_cluster(node_to_consult, &nodes_ids);
        }
        self.execute_read_repair(
            node_id,
            &nodes_ids,
            table_name,
            rows_of_nodes,
            replication_factor_quantity,
        )?;

        Ok(true)
    }

    fn execute_read_repair(
        &self,
        replica_to_repair: NodeId,
        nodes_ids: &[NodeId],
        table_name: &str,
        rows_of_nodes: Vec<Vec<Vec<String>>>,
        replication_factor_quantity: Uint,
    ) -> Result<()> {
        let rows_as_string = self.get_most_recent_rows_as_string(rows_of_nodes, table_name)?;
        let mut node_to_repair = replica_to_repair;
        for _ in 0..replication_factor_quantity {
            if node_to_repair == self.id {
                let node_writer = self.write()?;
                let table = node_writer.get_table(table_name)?;
                DiskHandler::repair_rows(
                    &node_writer.storage_addr,
                    table,
                    &node_writer.default_keyspace_name,
                    replica_to_repair,
                    &rows_as_string,
                )?;
            } else {
                let sv_action = SvAction::RepairRows(
                    table_name.to_string(),
                    replica_to_repair,
                    rows_as_string.as_bytes().to_vec(),
                )
                .as_bytes();
                send_to_node_and_wait_response_with_timeout(
                    node_to_repair,
                    sv_action,
                    PortType::Priv,
                    false,
                    Some(TIMEOUT_SECS),
                )?;
            };
            node_to_repair = next_node_in_the_cluster(node_to_repair, nodes_ids);
        }
        Ok(())
    }

    // ###########################################################################################
    // ######################################### INSERT ##########################################
    // ###########################################################################################

    fn insert_with_other_nodes(
        &self,
        insert: Insert,
        request: &[Byte],
        consistency_level: &Consistency,
    ) -> Result<Vec<Byte>> {
        let timestamp = Utc::now().timestamp();
        let table_name: String = insert.table.get_name();
        // let partitions_keys_to_nodes = self.get_partition_keys_values(&table_name)?.clone();
        let mut response: Vec<Byte> = Vec::new();
        let node_reader = self.read()?;
        let partition_key_value = get_partition_key_value_from_insert_statement(
            &insert,
            node_reader.get_table(&table_name)?,
        )?;
        let node_id = node_reader.select_node(&partition_key_value);
        let replication_factor_quantity = node_reader.get_replicas_from_table_name(&table_name)?;
        let consistency_number =
            consistency_level.as_usize(replication_factor_quantity as usize)?;
        let mut consistency_counter = 0;
        let mut wait_response = true;
        let nodes_ids = node_reader.get_nodes_ids();
        let mut node_to_replicate = node_id;
        let nodes_quantity = node_reader.get_actual_n_nodes();
        drop(node_reader);

        for i in 0..nodes_quantity {
            if (i as Uint) < replication_factor_quantity {
                response = if node_to_replicate == self.id {
                    let mut node_writer = self.write()?;
                    node_writer.process_insert(&insert, timestamp, node_id)?
                } else {
                    self.forward_insert_to_replica(
                        node_id,
                        node_to_replicate,
                        request,
                        timestamp,
                        wait_response,
                    )?
                }
            } else if node_to_replicate == self.id {
                self.add_partition_value_if_new(&table_name, &insert)?;
            } else {
                self.forward_insert_request_to_other_nodes_table(
                    node_to_replicate,
                    &table_name,
                    &insert,
                    wait_response,
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

    fn forward_insert_to_replica(
        &self,
        node_id: NodeId,
        node_to_replicate: NodeId,
        request: &[Byte],
        timestamp: Long,
        wait_response: bool,
    ) -> Result<Vec<Byte>> {
        let request_with_metadata = add_metadata_to_internal_request_of_any_kind(
            SvAction::InternalQuery(request.to_vec()).as_bytes(),
            Some(timestamp),
            Some(node_id),
        );
        let mut res: Vec<Byte> = Vec::new();
        if self.neighbour_is_responsive(node_to_replicate)? {
            res = match send_to_node_and_wait_response_with_timeout(
                node_to_replicate,
                request_with_metadata,
                PortType::Priv,
                wait_response,
                Some(TIMEOUT_SECS),
            ) {
                Ok(res) => res,
                Err(err) => {
                    return Err(err);
                }
            };
        }
        if res.is_empty() && wait_response {
            self.write()?
                .acknowledge_offline_neighbour(node_to_replicate);
        }
        Ok(res)
    }

    fn add_partition_value_if_new(&self, table_name: &str, insert: &Insert) -> Result<()> {
        let node_reader = self.read()?;
        let table = node_reader.get_table(table_name)?;
        let partition_value = get_partition_key_value_from_insert_statement(insert, table)?;
        let partition_values = node_reader
            .check_if_has_new_partition_value(partition_value, &table.get_name().to_string())?;
        drop(node_reader);

        match partition_values {
            Some(new_partition_values) => {
                let mut node_writer = self.write()?;
                node_writer
                    .tables_and_partitions_keys_values
                    .insert(insert.table.get_name().to_string(), new_partition_values)
            }
            None => None,
        };
        Ok(())
    }

    fn forward_insert_request_to_other_nodes_table(
        &self,
        node_to_replicate: NodeId,
        table_name: &str,
        insert: &Insert,
        wait_response: bool,
    ) -> Result<()> {
        let node_reader = self.read()?;
        let partition_value =
            get_partition_value_from_insert(insert, node_reader.get_table(table_name)?)?;
        drop(node_reader);

        let request_with_metadata = add_metadata_to_internal_request_of_any_kind(
            SvAction::AddPartitionValueToMetadata(table_name.to_string(), partition_value)
                .as_bytes(),
            None,
            None,
        );
        if self.neighbour_is_responsive(node_to_replicate)?
            && send_to_node_and_wait_response_with_timeout(
                node_to_replicate,
                request_with_metadata,
                PortType::Priv,
                wait_response,
                Some(TIMEOUT_SECS),
            )
            .is_err()
            && wait_response
        {
            self.write()?
                .acknowledge_offline_neighbour(node_to_replicate);
        }
        Ok(())
    }

    // ###########################################################################################
    // ######################################### UPDATE ##########################################
    // ###########################################################################################

    fn update_with_other_nodes(
        &self,
        update: Update,
        request: &[Byte],
        consistency_level: &Consistency,
    ) -> Result<Vec<Byte>> {
        let timestamp = Utc::now().timestamp();
        let table_name = update.table_name.get_name();
        let node_reader = self.read()?;
        let partitions_keys_to_nodes = node_reader.get_partition_keys_values(&table_name)?.clone();
        let mut consulted_nodes: Vec<String> = Vec::new();
        let replication_factor_quantity = node_reader.get_replicas_from_table_name(&table_name)?;
        let consistency_number =
            consistency_level.as_usize(replication_factor_quantity as usize)?;
        drop(node_reader);

        for partition_key_value in partitions_keys_to_nodes {
            let mut consistency_counter = 0;
            let node_reader = self.read()?;
            let node_id = node_reader.select_node(&partition_key_value);
            drop(node_reader);

            if !consulted_nodes.contains(&partition_key_value) {
                let current_response = if node_id == self.id {
                    let mut node_writer = self.write()?;
                    node_writer.process_update(&update, timestamp, self.id)?
                } else {
                    self.forward_update(node_id, request, timestamp)?
                };
                if verify_succesful_response(&current_response) {
                    consistency_counter += 1;
                }

                consulted_nodes.push(partition_key_value.clone());
                let node_reader = self.read()?;
                let replication_factor = node_reader.get_replicas_from_table_name(&table_name)?;
                drop(node_reader);

                self.replicate_update_in_other_nodes(
                    replication_factor,
                    node_id,
                    request,
                    &update,
                    timestamp,
                    &mut consistency_counter,
                )?;

                if consistency_counter < consistency_number {
                    return Err(Error::ServerError(format!(
                        "No se pudo cumplir con el nivel de consistencia {}, solo se logró con {} de {}",
                        consistency_level, consistency_counter, consistency_number,
                    )));
                }
            }
        }

        Ok(Node::create_result_void())
    }

    fn forward_update(
        &self,
        node_id: NodeId,
        request: &[Byte],
        timestamp: Long,
    ) -> Result<Vec<Byte>> {
        let request_with_metadata = add_metadata_to_internal_request_of_any_kind(
            SvAction::InternalQuery(request.to_vec()).as_bytes(),
            Some(timestamp),
            Some(node_id),
        );
        let mut res: Vec<Byte> = Vec::new();
        if self.neighbour_is_responsive(node_id)? {
            res = match send_to_node_and_wait_response_with_timeout(
                node_id,
                request_with_metadata,
                PortType::Priv,
                true,
                Some(TIMEOUT_SECS),
            ) {
                Ok(res) => res,
                Err(err) => {
                    return Err(err);
                }
            };
        }
        if res.is_empty() {
            self.write()?.acknowledge_offline_neighbour(node_id);
        }
        Ok(res)
    }

    fn replicate_update_in_other_nodes(
        &self,
        replication_factor: Uint,
        node_id: Byte,
        request: &[Byte],
        update: &Update,
        timestamp: Long,
        consistency_counter: &mut usize,
    ) -> Result<()> {
        let node_reader = self.read()?;
        let nodes_ids = node_reader.get_nodes_ids();
        drop(node_reader);
        let mut node_to_replicate = next_node_in_the_cluster(node_id, &nodes_ids);
        for _ in 1..replication_factor {
            let current_response = if node_to_replicate == self.id {
                let mut node_writer = self.write()?;
                node_writer.process_update(update, timestamp, node_id)?
            } else {
                let request_with_metadata = add_metadata_to_internal_request_of_any_kind(
                    SvAction::InternalQuery(request.to_vec()).as_bytes(),
                    Some(timestamp),
                    Some(node_id),
                );
                let mut replica_response: Vec<Byte> = Vec::new();
                if self.neighbour_is_responsive(node_to_replicate)? {
                    replica_response = match send_to_node_and_wait_response_with_timeout(
                        node_to_replicate,
                        request_with_metadata,
                        PortType::Priv,
                        true,
                        Some(TIMEOUT_SECS),
                    ) {
                        Ok(res) => res,
                        Err(err) => {
                            return Err(err);
                        }
                    };
                }

                if replica_response.is_empty() {
                    let mut node_writer = self.write()?;
                    node_writer.acknowledge_offline_neighbour(node_to_replicate);
                }
                replica_response
            };
            node_to_replicate = next_node_in_the_cluster(node_to_replicate, &nodes_ids);

            if verify_succesful_response(&current_response) {
                *consistency_counter += 1;
            }
        }
        Ok(())
    }

    // ###########################################################################################
    // ######################################### DELETE ##########################################
    // ###########################################################################################

    fn delete_with_other_nodes(
        &self,
        delete: Delete,
        request: &[Byte],
        consistency_level: &Consistency,
    ) -> Result<Vec<Byte>> {
        let table_name = delete.from.get_name();
        let mut consulted_nodes: Vec<String> = Vec::new();
        let node_reader = self.write()?;
        let partitions_keys_to_nodes = node_reader.get_partition_keys_values(&table_name)?.clone();
        let replication_factor_quantity = node_reader.get_replicas_from_table_name(&table_name)?;
        let consistency_number =
            consistency_level.as_usize(replication_factor_quantity as usize)?;
        drop(node_reader);

        for partition_key_value in partitions_keys_to_nodes {
            let node_reader = self.read()?;
            let node_id = node_reader.select_node(&partition_key_value);
            drop(node_reader);

            if !consulted_nodes.contains(&partition_key_value) {
                consulted_nodes.push(partition_key_value.clone());
                let node_reader = self.read()?;
                let replication_factor = node_reader.get_replicas_from_table_name(&table_name)?;
                drop(node_reader);

                self.replicate_delete_in_other_nodes(
                    replication_factor,
                    node_id,
                    request,
                    &delete,
                    consistency_number,
                )?;
            }
        }
        Ok(Node::create_result_void())
    }

    // Función auxiliar para replicar el delete en otros nodos
    fn replicate_delete_in_other_nodes(
        &self,
        replication_factor: Uint,
        node_id: Byte,
        request: &[Byte],
        delete: &Delete,
        consistency_number: usize,
    ) -> Result<()> {
        let mut consistency_counter = 0;
        let mut wait_response = true;
        let node_reader = self.read()?;
        let nodes_ids = node_reader.get_nodes_ids();
        drop(node_reader);
        let mut node_to_replicate = node_id;
        for _ in 0..replication_factor {
            let current_response = if node_to_replicate == self.id {
                let mut node_writer = self.write()?;
                node_writer.process_delete(delete, node_id)?
            } else {
                self.forward_delete(node_id, node_to_replicate, request, wait_response)?
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

    fn forward_delete(
        &self,
        node_id: NodeId,
        node_to_replicate: NodeId,
        request: &[Byte],
        wait_response: bool,
    ) -> Result<Vec<Byte>> {
        let request_with_metadata = add_metadata_to_internal_request_of_any_kind(
            SvAction::InternalQuery(request.to_vec()).as_bytes(),
            None,
            Some(node_id),
        );
        let mut res: Vec<Byte> = Vec::new();
        if self.neighbour_is_responsive(node_to_replicate)? {
            res = match send_to_node_and_wait_response_with_timeout(
                node_to_replicate,
                request_with_metadata,
                PortType::Priv,
                wait_response,
                Some(TIMEOUT_SECS),
            ) {
                Ok(res) => res,
                Err(err) => {
                    return Err(err);
                }
            }
        };

        if res.is_empty() && wait_response {
            let mut node_writer = self.write()?;
            node_writer.acknowledge_offline_neighbour(node_to_replicate);
        }
        Ok(res)
    }

    // ###########################################################################################
    // ################################### STATEMENTS INTERNOS ###################################
    // ###########################################################################################

    /// Maneja una declaración interna.
    fn handle_internal_statement(
        &self,
        statement: Statement,
        internal_metadata: (Option<Long>, Option<Byte>),
    ) -> Result<Vec<Byte>> {
        let mut node_writer = self.write()?;
        match statement {
            Statement::DdlStatement(ddl_statement) => {
                node_writer.handle_internal_ddl_statement(ddl_statement, internal_metadata)
            }
            Statement::DmlStatement(dml_statement) => {
                node_writer.handle_internal_dml_statement(dml_statement, internal_metadata)
            }
            Statement::Startup => Err(Error::Invalid(
                "No se deberia haber mandado el startup por este canal".to_string(),
            )),
            Statement::LoginUser(_) => Err(Error::Invalid(
                "No se deberia haber mandado el login por este canal".to_string(),
            )),
        }
    }

    // ###########################################################################################
    // ##################################### ROUND DE GOSSIP #####################################
    // ###########################################################################################

    /// Consulta si el nodo ya está listo para recibir _queries_. Si lo está, actualiza su estado.
    fn is_bootstrap_done(&self) -> Result<()> {
        let node_reader = self.read()?;
        let node_status = node_reader.endpoint_state.get_appstate_status();
        if node_reader.neighbours_states.len() == node_reader.get_actual_n_nodes()
            // && (*node_status == AppStatus::Bootstrap || *node_status == AppStatus::Offline)
        && *node_status != AppStatus::Normal
        && *node_status != AppStatus::RelocationIsNeeded
        && *node_status != AppStatus::RelocatingData
        && *node_status != AppStatus::Ready
        && *node_status != AppStatus::Left
        && *node_status != AppStatus::Remove
        && *node_status != AppStatus::NewNode
        && *node_status != AppStatus::UpdatingReplicas
        {
            drop(node_reader);

            let mut node_writer = self.write()?;
            node_writer
                .endpoint_state
                .set_appstate_status(AppStatus::Normal);
            println!("El nodo {} fue iniciado correctamente.", self.id);

            if node_writer.is_new_node {
                Node::request_previous_metadata_for_new_node(self.id);
            }
        }
        Ok(())
    }

    /// Consulta si la relocalización es necesaria, si es cierto, inicia el proceso de relocalización.
    fn is_relocation_needed(&self) -> Result<()> {
        let node_reader = self.read()?;
        if *node_reader.endpoint_state.get_appstate_status() != AppStatus::RelocationIsNeeded {
            return Ok(());
        }
        let n_nodes = node_reader.get_actual_n_nodes();
        let mut waiting_relocate_nodes_counter = 0;
        for endpoint_state in node_reader.neighbours_states.values() {
            if *endpoint_state.get_appstate_status() == AppStatus::RelocationIsNeeded
                || *endpoint_state.get_appstate_status() == AppStatus::RelocatingData
                || *endpoint_state.get_appstate_status() == AppStatus::Ready
            {
                waiting_relocate_nodes_counter += 1;
            }
        }
        drop(node_reader);
        if waiting_relocate_nodes_counter == n_nodes {
            let mut node_writer = self.write()?;
            node_writer
                .endpoint_state
                .set_appstate_status(AppStatus::RelocatingData);
            println!("Iniciando relocalización.");
            node_writer.run_relocation()?;
        }
        Ok(())
    }

    /// Consulta si la relocalización finalizó, si es cierto, actualiza el estado del nodo.
    fn is_relocation_done(&self) -> Result<()> {
        let node_reader = self.read()?;
        if *node_reader.endpoint_state.get_appstate_status() != AppStatus::Ready {
            return Ok(());
        }
        let n_nodes = Node::get_all_n_nodes();
        let mut ready_nodes_counter = 0;
        let mut node_deleted = -1;
        for (node_id, endpoint_state) in &node_reader.neighbours_states {
            if *endpoint_state.get_appstate_status() == AppStatus::Ready
                || *endpoint_state.get_appstate_status() == AppStatus::Normal
            {
                ready_nodes_counter += 1;
            }
            if *endpoint_state.get_appstate_status() == AppStatus::Remove
                || *endpoint_state.get_appstate_status() == AppStatus::Offline
            {
                ready_nodes_counter += 1;
                node_deleted = *node_id as i32;
            }
        }
        drop(node_reader);
        if ready_nodes_counter == n_nodes {
            if node_deleted != -1 {
                DiskHandler::delete_node_id_and_ip(node_deleted as u8)?;
            }
            self.write()?.finish_relocation()?;
        }
        Ok(())
    }

    /// Consulta si el nodo es uno que debe darse de baja, si lo es, espera a que
    /// el resto de nodos terminen de relocalizar sus datos, y termina con el proceso
    /// de baja.
    fn leaving_node_has_to_relocate(&self) -> Result<()> {
        let node_reader = self.read()?;
        if *node_reader.endpoint_state.get_appstate_status() != AppStatus::Left {
            return Ok(());
        }
        let n_nodes = node_reader.get_actual_n_nodes();
        let mut waiting_relocate_nodes_counter = 0;
        for endpoint_state in node_reader.neighbours_states.values() {
            if *endpoint_state.get_appstate_status() == AppStatus::Ready {
                waiting_relocate_nodes_counter += 1;
            }
        }
        drop(node_reader);

        if waiting_relocate_nodes_counter == n_nodes {
            let mut node_writer = self.write()?;
            node_writer.relocate_rows()?;
            node_writer.stop_gossiper_and_beater();
            node_writer
                .endpoint_state
                .set_appstate_status(AppStatus::Remove);

            let neighbours = node_writer.get_nodes_ids();
            for neighbour_id in neighbours {
                if neighbour_id == self.id {
                    continue;
                }
                send_to_node(
                    neighbour_id,
                    SvAction::NodeDeleted(self.id).as_bytes(),
                    PortType::Priv,
                )?;
            }
        }
        Ok(())
    }

    /// Consigue la información de _gossip_ que contiene este nodo.
    fn get_gossip_info(&self) -> Result<GossipInfo> {
        let mut gossip_info = GossipInfo::new();
        for (node_id, endpoint_state) in &self.read()?.neighbours_states {
            gossip_info.insert(node_id.to_owned(), endpoint_state.clone_heartbeat());
        }

        Ok(gossip_info)
    }

    /// Inicia un intercambio de _gossip_ con los vecinos dados.
    pub fn gossip(&self, neighbours: HashSet<NodeId>) -> Result<()> {
        self.is_bootstrap_done()?;
        self.leaving_node_has_to_relocate()?;
        self.is_relocation_needed()?;
        self.is_relocation_done()?;

        for neighbour_id in neighbours {
            // if *neighbour_state.get_appstate_status() != AppStatus::Left
            // && *neighbour_state.get_appstate_status() != AppStatus::Remove{
            if send_to_node(
                neighbour_id,
                SvAction::Syn(self.id.to_owned(), self.get_gossip_info()?).as_bytes(),
                PortType::Priv,
            )
            .is_err()
            {
                // No devolvemos error porque no se considera un error que un vecino
                // no responda en esta instancia, sino que esta apagado.
                self.write()?.acknowledge_offline_neighbour(neighbour_id);
                println!("Se pone al nodo {} en estado Offline", neighbour_id);
            }
            // }
        }
        Ok(())
    }

    /// Se recibe un mensaje [SYN](crate::actions::opcode::SvAction::Syn).
    pub fn syn(&self, emissor_id: NodeId, emissor_gossip_info: GossipInfo) -> Result<()> {
        let mut own_gossip_info = GossipInfo::new(); // quiero info de estos nodos
        let mut response_nodes = NodesMap::new(); // doy info de estos nodos

        self.classify_nodes_in_gossip(
            &emissor_gossip_info,
            &mut own_gossip_info,
            &mut response_nodes,
        )?;

        // Ahora rondamos nuestros vecinos para ver si tenemos uno que el nodo emisor no
        for (own_node_id, endpoint_state) in &self.read()?.neighbours_states {
            if !emissor_gossip_info.contains_key(own_node_id) {
                response_nodes.insert(*own_node_id, endpoint_state.clone());
            }
        }

        if let Err(err) = send_to_node(
            emissor_id,
            SvAction::Ack(self.id.to_owned(), own_gossip_info, response_nodes).as_bytes(),
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
        &self,
        emissor_gossip_info: &HashMap<Byte, HeartbeatState>,
        own_gossip_info: &mut HashMap<Byte, HeartbeatState>,
        response_nodes: &mut HashMap<Byte, EndpointState>,
    ) -> Result<()> {
        let neighbours_states = &self.read()?.neighbours_states;
        for (node_id, emissor_heartbeat) in emissor_gossip_info {
            match neighbours_states.get(node_id) {
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
        Ok(())
    }

    /// Se recibe un mensaje [ACK](crate::actions::opcode::SvAction::Ack).
    pub fn ack(
        &self,
        receptor_id: NodeId,
        receptor_gossip_info: GossipInfo,
        response_nodes: NodesMap,
    ) -> Result<()> {
        // Poblamos un mapa con los estados que pide el receptor
        let mut nodes_for_receptor = NodesMap::new();
        let node_reader = self.read()?;
        let neighbours_states = &node_reader.neighbours_states;
        for (node_id, receptor_heartbeat) in &receptor_gossip_info {
            let own_endpoint_state = &neighbours_states[node_id];
            if own_endpoint_state.get_heartbeat() > receptor_heartbeat {
                // Hacemos doble chequeo que efectivamente tenemos información más nueva
                nodes_for_receptor.insert(*node_id, own_endpoint_state.clone());
            }
        }
        drop(node_reader);
        // Reemplazamos la información de nuestros vecinos por la más nueva que viene del nodo receptor
        // Asumimos que es más nueva ya que fue previamente verificada
        self.write()?.update_neighbours(response_nodes)?;

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

    /// Se recibe un mensaje [ACK2](crate::actions::opcode::SvAction::Ack2).
    pub fn ack2(&self, nodes_map: NodesMap) -> Result<()> {
        self.write()?.update_neighbours(nodes_map)
    }

    // ###########################################################################################
    // ######################################### ACTIONS #########################################
    // ###########################################################################################

    fn exec_direct_read_request(&self, mut bytes: Vec<Byte>) -> Result<String> {
        let node_number = match bytes.pop() {
            Some(node_number) => node_number,
            None => {
                return Err(Error::ServerError(
                    "No se especificó el ID del nodo al hacer read-repair".to_string(),
                ))
            }
        };
        let select = parse_select_from_query_body_as_bytes(&bytes)?;

        // Queremos lockear cuando entra a una operacion de DiskHandler ya que no queremos inconsistencias
        let node_writer = self.write()?;
        DiskHandler::get_rows_with_timestamp_as_string(
            &node_writer.storage_addr,
            &node_writer.get_default_keyspace_name()?,
            &select,
            node_number,
        )
    }

    fn exec_digest_read_request(&self, bytes: Vec<Byte>) -> Vec<Byte> {
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
        // Inicio aca el lockeo porque estas variables son referencias, entonces se dropearian con el reader
        // y no podria usarlas en el disk handler
        let node_writer = self.write()?;
        if !node_writer.table_exists(&table_name) {
            return Err(Error::ServerError(format!(
                "La tabla `{}` no existe",
                table_name
            )));
        }

        let table = node_writer.get_table(&table_name)?;
        let keyspace_name = table.get_keyspace();
        if !node_writer.keyspace_exists(keyspace_name) {
            return Err(Error::ServerError(format!(
                "El keyspace `{}` asociado a la tabla `{}` no existe",
                keyspace_name, table_name
            )));
        }
        let rows = String::from_utf8(rows_bytes)
            .map_err(|_| Error::ServerError("Error al castear de bytes a string".to_string()))?;

        DiskHandler::repair_rows(
            &node_writer.storage_addr,
            table,
            &node_writer.get_default_keyspace_name()?,
            node_id,
            &rows,
        )
    }

    // ###########################################################################################
    // ####################################### AUXILIARES ########################################
    // ###########################################################################################

    /// Consulta si un nodo vecino está listo para recibir _queries_.
    fn neighbour_is_responsive(&self, node_id: NodeId) -> Result<bool> {
        let mut is_ready = false;
        let node_reader = self.read()?;
        if let Some(endpoint_state) = node_reader.neighbours_states.get(&node_id) {
            is_ready = *endpoint_state.get_appstate_status() == AppStatus::Normal;
        }
        Ok(is_ready)
    }

    /// Consulta si el nodo contenido puede recibir consultas.
    pub fn node_is_responsive(&self) -> Result<bool> {
        Ok(self.read()?.is_responsive())
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
        let node_reader = self.read()?;
        let total_length_until_end_of_metadata = node_reader.get_columns_metadata_length(&res);
        let total_lenght_until_rows_content = total_length_until_end_of_metadata + 4;
        let mut quantity_rows = node_reader.get_quantity_of_rows(
            results_from_another_nodes,
            total_length_until_end_of_metadata,
        );
        let new_quantity_rows = node_reader
            .get_quantity_of_rows(result_from_actual_node, total_length_until_end_of_metadata);
        quantity_rows += new_quantity_rows;
        results_from_another_nodes
            [total_length_until_end_of_metadata..total_lenght_until_rows_content]
            .copy_from_slice(&quantity_rows.to_be_bytes());

        let mut new_res = result_from_actual_node[total_lenght_until_rows_content..].to_vec();
        results_from_another_nodes.append(&mut new_res);

        let final_length = (results_from_another_nodes.len() as Uint) - 9;
        results_from_another_nodes[5..9].copy_from_slice(&final_length.to_be_bytes());
        Ok(())
    }

    fn get_most_recent_rows_as_string(
        &self,
        rows_of_nodes: Vec<Vec<Vec<String>>>,
        table_name: &str,
    ) -> Result<String> {
        let node_reader = self.read()?;
        let table = node_reader.get_table(table_name)?;
        let primary_key_columns = table.get_position_of_primary_key()?;
        drop(node_reader);
        let mut merged_map: HashMap<Vec<String>, Vec<String>> = HashMap::new();
        for rows in rows_of_nodes {
            for row in rows {
                // Asegúrate de que la fila tenga suficientes columnas
                if primary_key_columns.iter().any(|&idx| idx >= row.len()) || row.is_empty() {
                    continue;
                }
                // Crear la clave dinámica
                let key: Vec<String> = primary_key_columns
                    .iter()
                    .map(|&idx| row[idx].clone())
                    .collect();
                let timestamp = &row[row.len() - 1]; // Última columna como timestamp

                // Revisar si ya existe una entrada en el mapa
                match merged_map.get(&key) {
                    Some(existing_row) => {
                        let existing_timestamp = &existing_row[existing_row.len() - 1];
                        // Actualizar si el timestamp actual es más reciente
                        if timestamp > existing_timestamp {
                            merged_map.insert(key, row.clone());
                        }
                    }
                    None => {
                        // Insertar nueva entrada
                        merged_map.insert(key, row.clone());
                    }
                }
            }
        }

        // Convertir el mapa a un Vec<Vec<String>>
        let newer_rows: Vec<Vec<String>> = merged_map.into_values().collect();
        let rows_as_string = newer_rows
            .iter()
            .map(|row| row.join(","))
            .collect::<Vec<String>>()
            .join("\n");
        Ok(rows_as_string)

        // let mut most_recent_timestamps: Vec<(usize, String)> = Vec::new();
        // let mut newer_rows: Vec<Vec<String>> = Vec::new();

        // for (i,rows) in ids_and_rows.iter().enumerate() {
        //     for (j, row) in rows.iter().enumerate() {
        //         if most_recent_timestamps.len() <= j {
        //             most_recent_timestamps.push((i, row[row.len() - 1].clone()));
        //         } else {
        //             let actual_timestamp = row[row.len() - 1].clone();
        //             if actual_timestamp > most_recent_timestamps[j].1 {
        //                 most_recent_timestamps[j] = (i, actual_timestamp);
        //             }
        //         }
        //     }
        // }
        // for (i, actual_timestamp) in most_recent_timestamps.iter().enumerate() {
        //     let new_row = &ids_and_rows[actual_timestamp.0][i];
        //     newer_rows.push(new_row.clone());
        // }
        // let rows_as_string = newer_rows
        //     .iter()
        //     .map(|row| row.join(","))
        //     .collect::<Vec<String>>()
        //     .join("\n");
        // rows_as_string
    }
}

impl Clone for SessionHandler {
    fn clone(&self) -> Self {
        SessionHandler {
            id: self.id,
            lock: Arc::clone(&self.lock),
        }
    }
}

// ##############################################################################################
// ################################## AUXILIARES INDEPENDIENTES #################################
// ##############################################################################################

pub fn make_error_response(err: Error) -> Vec<Byte> {
    let mut response: Vec<Byte> = Vec::new();
    let mut bytes_err = err.as_bytes();
    response.append(&mut Version::ResponseV5.as_bytes());
    response.append(&mut Flag::Default.as_bytes());
    response.append(&mut Stream::new(0).as_bytes());
    response.append(&mut Opcode::RequestError.as_bytes());
    response.append(&mut Length::new(bytes_err.len() as Uint).as_bytes());
    response.append(&mut bytes_err);
    response
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

fn parse_select_from_query_body_as_bytes(bytes: &[Byte]) -> Result<Select> {
    let statement = match QueryBody::try_from(bytes) {
        Ok(query_body) => match make_parse(&mut tokenize_query(query_body.get_query())) {
            Ok(statement) => statement,
            Err(_err) => {
                return Err(Error::ServerError(
                    "No se pudo parsear el statement, durante read-repair".to_string(),
                ))
            }
        },
        Err(_err) => {
            return Err(Error::ServerError(
                "No se pudo parsear el body de la query, durante read-repair".to_string(),
            ))
        }
    };
    match statement {
        Statement::DmlStatement(DmlStatement::SelectStatement(select)) => Ok(select),
        _ => Err(Error::ServerError(
            "La declaración no es un SELECT, durante read-repair".to_string(),
        )),
    }
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

/// Revisa si hay metadata extra necesaria para la query pedida
fn read_metadata_from_internal_request(
    internal_metadata: Vec<Byte>,
) -> (Option<Long>, Option<Byte>) {
    if internal_metadata.len() == 9 {
        let bytes: [Byte; 8] = match internal_metadata[0..8].try_into() {
            Ok(value) => value,
            Err(_err) => [5, 5, 5, 5, 5, 5, 5, 5], // nunca pasa
        };
        let timestamp = Long::from_be_bytes(bytes);
        let node_id = internal_metadata[8];
        return (Some(timestamp), Some(node_id));
    } else if internal_metadata.len() == 8 {
        let bytes: [Byte; 8] = match internal_metadata[0..8].try_into() {
            Ok(value) => value,
            Err(_err) => [5, 5, 5, 5, 5, 5, 5, 5], // nunca pasa
        };
        let timestamp = Long::from_be_bytes(bytes);
        return (Some(timestamp), None);
    } else if internal_metadata.len() == 1 {
        let node_id = internal_metadata[0];
        return (None, Some(node_id));
    }
    (None, None)
}

/// Agrega metadata, como el timestamp o el node_id si es necesario, sino no agrega estos campos.
fn add_metadata_to_internal_request_of_any_kind(
    mut sv_action_with_request: Vec<Byte>,
    timestamp: Option<Long>,
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

fn check_if_read_repair_is_neccesary(
    consistency_counter: &mut usize,
    consistency_number: usize,
    exec_read_repair: &mut bool,
    responses: Vec<Vec<Byte>>,
    first_hashed_value: Ulong,
    inconsistent_digest_request: bool,
) {
    if *consistency_counter < consistency_number || inconsistent_digest_request {
        *exec_read_repair = true
    };
    for hashed_value_vec in responses {
        if hashed_value_vec.len() < 8 {
            *exec_read_repair = true;
        }
        let mut array = [0u8; 8]; // 8 es el len del hashed_value
        array.copy_from_slice(&hashed_value_vec[0..8]);
        let hashed_value_of_response = Ulong::from_be_bytes(array);
        if first_hashed_value != hashed_value_of_response {
            *exec_read_repair = true;
        }
    }
}

fn create_utf8_string_from_bytes(extern_response: Vec<Byte>) -> Result<String> {
    Ok(match String::from_utf8(extern_response) {
        Ok(value) => value,
        Err(_err) => {
            return Err(Error::ServerError(
                "Error al castear de vector a string".to_string(),
            ))
        }
    })
}

fn add_rows(res: String, rows_of_nodes: &mut Vec<Vec<Vec<String>>>) {
    let rows: Vec<Vec<String>> = res
        .split("\n")
        .map(|row| row.split(",").map(|col| col.to_string()).collect())
        .collect();
    rows_of_nodes.push(rows);
}

fn get_partition_key_value_from_insert_statement(insert: &Insert, table: &Table) -> Result<String> {
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

pub fn get_partition_value_from_insert(insert: &Insert, table: &Table) -> Result<String> {
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
