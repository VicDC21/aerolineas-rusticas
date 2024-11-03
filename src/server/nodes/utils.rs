//! Módulo para funciones auxiliares relacionadas a nodos.

use std::{
    io::{Read, Write},
    net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4, TcpStream},
};

use crate::protocol::{
    aliases::{results::Result, types::Byte},
    errors::error::Error,
};
use crate::server::nodes::{node::NodeId, port_type::PortType};

/// Genera una dirección de socket a partir de un ID.
pub fn guess_socket(id: NodeId, port_type: PortType) -> SocketAddr {
    SocketAddr::V4(SocketAddrV4::new(
        Ipv4Addr::new(127, 0, 0, id),
        port_type.to_num(),
    ))
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

/// Manda un mensaje a un nodo específico y espera por la respuesta de este.
pub fn send_to_node_and_wait_response(
    id: NodeId,
    bytes: Vec<Byte>,
    port_type: PortType,
) -> Result<Vec<u8>> {
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
    // para asegurarse de que se vacía el stream antes de escuchar de nuevo.
    if let Err(err) = stream.flush() {
        println!("Error haciendo flush desde el servidor:\n\n{}", err);
    }
    let mut buf = Vec::<Byte>::new();
    match stream.read_to_end(&mut buf) {
        Err(err) => println!("Error recibiendo response de un nodo:\n\n{}", err),
        Ok(i) => {
            println!("Nodo {} recibió {} bytes - {:?}", id, i, buf);
        }
    }
    Ok(buf)
}

/// Adivina el ID del nodo a partir de una IP.
pub fn guess_id(ipaddr: &IpAddr) -> NodeId {
    match ipaddr {
        IpAddr::V4(ipv4) => {
            let [_, _, _, id] = ipv4.octets();
            id
        }
        IpAddr::V6(ipv6) => {
            let [_, _, _, _, _, _, _, _, _, _, _, _, _, _, _, id] = ipv6.octets();
            id
        }
    }
}

/// Divide un rango en `n` partes iguales.
pub fn divide_range(start: u64, end: u64, n: usize) -> Vec<(u64, u64)> {
    let range_length = end - start;
    let part_length = range_length / n as u64;
    let remainder = range_length % n as u64;

    (0..n)
        .map(|i| {
            let part_start = start + i as u64 * part_length + remainder.min(i as u64);
            let part_end = part_start + part_length + if i < remainder as usize { 1 } else { 0 };
            (part_start, part_end)
        })
        .collect()
}
