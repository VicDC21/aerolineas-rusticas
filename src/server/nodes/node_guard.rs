//! Módulo para el manejo del bloqueo o no de un nodo.

use std::{io::{Read, Write}, sync::{Arc, RwLock}};

use crate::{protocol::{aliases::{results::Result, types::Byte}, errors::error::Error}, server::{actions::opcode::SvAction, modes::ConnectionMode}};

use super::{disk_operations::disk_handler::DiskHandler, node::{Node, NodeId}};

/// Guarda una referencia compartida a un nodo, con la posibilidad de decidir si se quiere
/// lockear o no al nodo durante las operaciones que correspondan.
pub struct NodeGuard {
    pub id: NodeId,
    pub lock: Arc<RwLock<Node>>,
}

impl NodeGuard {
    /// Crea un nuevo `NodeGuard` con un nodo específico.
    pub fn new(id: NodeId, node: Node) -> Self {
        NodeGuard {
            id,
            lock: Arc::new(RwLock::new(node)),
        }
    }

    /// Accede al nodo para escritura mutable, es lockeado para otros.
    fn write(&self) -> Result<std::sync::RwLockWriteGuard<Node>> {
        match self.lock.write() {
            Ok(guard) => Ok(guard),
            Err(poisoned) => {
                let err = Err(Error::ServerError(format!(
                    "Lock envenenado del nodo con ID {} para escritura: {}",
                    self.id, &poisoned
                )));
                poisoned.into_inner();
                err
            }
        }
    }

    /// Accede al nodo para lectura inmutable, sigue siendo accesible para otros.
    fn read(&self) -> Result<std::sync::RwLockReadGuard<Node>> {
        match self.lock.read() {
            Ok(guard) => Ok(guard),
            Err(poisoned) => {
                let err = Err(Error::ServerError(format!(
                    "Lock envenenado del nodo con ID {} para lectura: {}",
                    self.id, &poisoned
                )));
                poisoned.into_inner();
                err
            }
        }
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
                self.write()?.beat();
            }
            SvAction::Gossip(neighbours) => {
                self.write()?.gossip(neighbours)?;
            }
            SvAction::Syn(emissor_id, gossip_info) => {
                self.read()?.syn(emissor_id, gossip_info)?;
            }
            SvAction::Ack(receptor_id, gossip_info, nodes_map) => {
                self.write()?.ack(receptor_id, gossip_info, nodes_map)?;
            }
            SvAction::Ack2(nodes_map) => {
                self.write()?.ack2(nodes_map)?;
            }
            SvAction::NewNeighbour(state) => {
                self.write()?.add_neighbour_state(state)?;
            }
            SvAction::SendEndpointState(id) => {
                self.read()?.send_endpoint_state(id);
            }
            SvAction::InternalQuery(bytes) => {
                let response = self.handle_request(&bytes, true, true);
                let _ = tcp_stream.write_all(&response[..]);
                if let Err(err) = tcp_stream.flush() {
                    return Err(Error::ServerError(err.to_string()));
                };
            }
            SvAction::StoreMetadata => {
                todo!();
                /*if let Err(err) = DiskHandler::store_node_metadata(&self.write()?) {
                    return Err(Error::ServerError(format!(
                        "Error guardando metadata del nodo {}: {}",
                        &self.id, err
                    )));
                }*/
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

    fn match_kind_of_conection_mode<S>(
        &mut self,
        bytes: Vec<Byte>,
        mut stream: S,
        is_logged: bool,
    ) -> Result<Vec<Byte>>
    where
        S: Read + Write,
    {
        match self.read()?.mode() {
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
}

impl Clone for NodeGuard {
    fn clone(&self) -> Self {
        NodeGuard {
            id: self.id,
            lock: Arc::clone(&self.lock),
        }
    }
}
