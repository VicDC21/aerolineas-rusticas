//! Módulo para funciones auxiliares relacionadas a nodos.

use {
    crate::{
        nodes::{addr::loader::AddrLoader, node::NodeId, port_type::PortType},
        utils::printable_bytes,
    },
    protocol::{
        aliases::{
            results::Result,
            types::{Byte, Ulong},
        },
        errors::error::Error,
    },
    std::{
        hash::{DefaultHasher, Hash, Hasher},
        io::{Read, Write},
        net::TcpStream,
    },
};

/// Hashea el valor recibido.
///
/// En esta función es determinístico, es decir, siempre devolverá el mismo valor para el mismo input.
/// Esto es así porque cada vez vuelve a instanciar un `DefaultHasher` nuevo, manteniendo la misma semilla.
pub fn hash_value<T: Hash>(value: T) -> Ulong {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

/// Divide un rango en `n` partes iguales.
pub fn divide_range(start: Ulong, end: Ulong, n: usize) -> Vec<(Ulong, Ulong)> {
    let range_length = end - start;
    let part_length = range_length / n as Ulong;
    let remainder = range_length % n as Ulong;

    (0..n)
        .map(|i| {
            let part_start = start + i as Ulong * part_length + remainder.min(i as Ulong);
            let part_end = part_start + part_length + if i < remainder as usize { 1 } else { 0 };
            (part_start, part_end)
        })
        .collect()
}

/// Devuelve el ID del siguiente nodo del cluster.
///
/// Se asume que el vector de IDs de los nodos está ordenado de menor a mayor.
pub fn next_node_in_the_cluster(current_id: Byte, nodes_ids: &[Byte]) -> Byte {
    let current_index = match nodes_ids.binary_search(&current_id) {
        Ok(index) => index,
        Err(_) => return nodes_ids[0], // si no se encuentra, se asume que es el primer nodo
    };
    if current_index + 1 == nodes_ids.len() {
        nodes_ids[0]
    } else {
        nodes_ids[current_index + 1]
    }
}

/// Devuelve el ID del nodo `n`-ésimo en el cluster, tomando como punto de partida `current_id`.
/// Si `reverse` es `true`, se devuelve el `n`-ésimo nodo en sentido contrario.
///
/// Se asume que el vector de IDs de los nodos está ordenado de menor a mayor.
pub fn n_th_node_in_the_cluster(
    current_id: Byte,
    nodes_ids: &[Byte],
    n: usize,
    reverse: bool,
) -> Byte {
    let current_index = match nodes_ids.binary_search(&current_id) {
        Ok(index) => index as i8,
        Err(_) => return nodes_ids[0], // si no se encuentra, se asume que es el primer nodo
    };

    let mut new_index: i8 = if reverse {
        current_index - n as i8
    } else {
        current_index + n as i8
    };

    let nodes_ids_len = nodes_ids.len() as i8;
    if new_index >= nodes_ids_len {
        new_index -= nodes_ids_len;
    }
    if new_index < 0 {
        new_index += nodes_ids_len;
    }
    nodes_ids[new_index as usize]
}

/// Manda un mensaje a un nodo específico.
pub fn send_to_node(id: NodeId, bytes: Vec<Byte>, port_type: PortType) -> Result<()> {
    let addr = AddrLoader::default_loaded().get_socket(&id, &port_type)?;
    let mut stream = match TcpStream::connect(addr) {
        Ok(tcpstream) => tcpstream,
        Err(err) => {
            return Err(Error::ServerError(format!(
                "No se pudo conectar al nodo con ID {id} el error fue {err:?}"
            )));
        }
    };
    if stream.write_all(&bytes[..]).is_err() {
        return Err(Error::ServerError(format!(
            "No se pudo escribir el contenido en {addr}"
        )));
    }
    Ok(())
}

/// Manda un mensaje a un nodo específico y espera por la respuesta de este, con un timeout.
/// Si el timeout se alcanza, se devuelve un buffer vacío.
///
/// `timeout` es medido en segundos.
pub fn send_to_node_and_wait_response_with_timeout(
    id: NodeId,
    bytes: Vec<Byte>,
    port_type: PortType,
    wait_response: bool,
    timeout: Option<Ulong>,
) -> Result<Vec<Byte>> {
    let addr = AddrLoader::default_loaded().get_socket(&id, &port_type)?;
    let mut stream = match TcpStream::connect(addr) {
        Ok(tcpstream) => tcpstream,
        Err(_) => {
            return Err(Error::ServerError(format!(
                "No se pudo conectar al nodo con ID {id}"
            )));
        }
    };
    println!(
        "Le escribe al nodo: {} la data: {}",
        id,
        printable_bytes(&bytes)
    );

    if stream.write_all(&bytes[..]).is_err() {
        return Err(Error::ServerError(format!(
            "No se pudo escribir el contenido en {addr}"
        )));
    }
    // para asegurarse de que se vacía el stream antes de escuchar de nuevo.
    if let Err(err) = stream.flush() {
        println!("Error haciendo flush desde el servidor:\n\n{err}");
    }
    let mut buf = Vec::<Byte>::new();

    if wait_response {
        if let Some(timeout_secs) = timeout {
            if let Err(err) =
                stream.set_read_timeout(Some(std::time::Duration::from_secs(timeout_secs)))
            {
                println!("Error estableciendo timeout en el nodo:\n\n{err}")
            }
        }
        match stream.read_to_end(&mut buf) {
            Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                println!("Timeout alcanzado al esperar respuesta del nodo {id}");
            }
            Err(err) => println!("Error recibiendo response del nodo {id}:\n\n{err}"),
            Ok(i) => {
                println!(
                    "Se recibió del nodo [{}] {} bytes: {}",
                    id,
                    i,
                    printable_bytes(&buf)
                );
            }
        }
    }

    Ok(buf)
}
