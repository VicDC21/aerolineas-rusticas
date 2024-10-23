//! Módulo para funciones auxiliares relacionadas a nodos.

use std::{
    io::Write,
    net::{Ipv4Addr, SocketAddr, SocketAddrV4, TcpStream},
};

use crate::protocol::{
    aliases::{results::Result, types::Byte},
    errors::error::Error,
};
use crate::server::nodes::{
    node::NodeId,
    port_type::PortType,
};

/// Genera una dirección de socket a partir de un ID.
pub fn guess_socket(id: NodeId, port_type: PortType) -> SocketAddr {
    SocketAddr::V4(
        SocketAddrV4::new(
            Ipv4Addr::new(127, 0, 0, id), 
            port_type.into())
    )
}

/// Manda un mensaje a un nodo específico.
pub fn send_to_node(id: NodeId, bytes: Vec<Byte>, port_type: PortType) -> Result<()> {
    let addr = guess_socket(id, port_type);
    let mut stream = match TcpStream::connect(addr) {
        Ok(tcpstream) => tcpstream,
        Err(_) => {
            return Err(Error::ServerError(format!(
                "No se pudo conectar al nodo con ID {}",
                id
            )))
        }
    };
    if stream.write_all(&bytes[..]).is_err() {
        return Err(Error::ServerError(format!(
            "No se pudo escribir el contenido en {}",
            addr
        )));
    }
    Ok(())
}
