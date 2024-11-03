//! Módulo para funciones auxiliares relacionadas a nodos.

use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader, Read, Result as IOResult, Write},
    net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4, TcpStream},
    str::FromStr,
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

/// Detecta _queries_ desde un archivo.
pub fn query_from_source(path: &str) -> Result<Vec<String>> {
    let mut queries = Vec::<String>::new();
    let file = match File::open(path) {
        Ok(f) => f,
        Err(file_err) => {
            return Err(Error::ServerError(format!(
                "Error abriendo el archivo:\n\n{}",
                file_err
            )));
        }
    };
    let bufreader = BufReader::new(file);

    // Asumimos que cada línea es una query completa
    for line in bufreader.lines().map_while(IOResult::ok) {
        queries.push(line);
    }

    Ok(queries)
}

/// Convierte un hashmap a un string.
pub fn hashmap_to_string<T: ToString>(map: &HashMap<String, T>) -> String {
    let mut res = String::new();

    for (key, value) in map {
        res.push_str(&key.to_string());
        res.push('.');
        res.push_str(&value.to_string());
        res.push(';');
    }

    res
}

/// Convierte un string a un hashmap.
pub fn string_to_hashmap<T: FromStr>(str: &str) -> Result<HashMap<String, T>> {
    let mut res = HashMap::new();

    for pair in str.split(';') {
        if pair.is_empty() {
            continue;
        }

        let parts: Vec<&str> = pair.split('.').collect();
        if parts.len() != 2 {
            return Err(Error::ServerError(
                "No se pudo parsear el hashmap".to_string(),
            ));
        }

        let key = parts[0].to_string();
        let value = parts[1].parse().map_err(|_| {
            Error::ServerError("No se pudo parsear el valor del hashmap".to_string())
        })?;

        res.insert(key, value);
    }

    Ok(res)
}

/// Convierte un hashmap de vectores a un string.
pub fn hashmap_vec_to_string<T: ToString>(map: &HashMap<String, Vec<T>>) -> String {
    let mut res = String::new();

    for (key, value) in map {
        res.push_str(key);
        res.push('.');
        res.push_str(
            &value
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join("-"),
        );
        res.push(';');
    }

    res
}

/// Convierte un string a un hashmap de vectores.
pub fn string_to_hashmap_vec<T: FromStr>(str: &str) -> Result<HashMap<String, Vec<T>>> {
    let mut res = HashMap::new();

    for pair in str.split(';') {
        if pair.is_empty() {
            continue;
        }

        let parts: Vec<&str> = pair.split('.').collect();
        if parts.len() != 2 {
            return Err(Error::ServerError(
                "No se pudo parsear el hashmap".to_string(),
            ));
        }

        let key = parts[0].to_string();
        let value = parts[1]
            .split('-')
            .map(|v| {
                v.parse().map_err(|_| {
                    Error::ServerError("No se pudo parsear el valor del hashmap".to_string())
                })
            })
            .collect::<Result<Vec<T>>>()?;

        res.insert(key, value);
    }

    Ok(res)
}
