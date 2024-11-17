//! Módulo para funciones auxiliares relacionadas a nodos.

use std::{
    collections::HashMap,
    fs::{read_dir, File},
    hash::{DefaultHasher, Hash, Hasher},
    io::{BufRead, BufReader, Read, Result as IOResult, Write},
    net::TcpStream,
    path::PathBuf,
    str::FromStr,
};

use crate::protocol::{
    aliases::{results::Result, types::Byte},
    errors::error::Error,
};
use crate::server::nodes::{addr::loader::AddrLoader, node::NodeId, port_type::PortType};

/// La ruta de _queries_ iniciales.
const INIT_QUERIES_PATH: &str = "scripts/init";
/// Extensión preferida para _queries_ de CQL, sin el punto de prefijo.
const QUERY_EXT: &str = "cql";

/// Hashea el valor recibido.
///
/// En esta función es determinístico, es decir, siempre devolverá el mismo valor para el mismo input.
/// Esto es así porque cada vez vuelve a instanciar un `DefaultHasher` nuevo, manteniendo la misma semilla.
pub fn hash_value<T: Hash>(value: T) -> u64 {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

/// Devuelve el ID del siguiente nodo donde se deberían replicar datos.
pub fn next_node_to_replicate_data(
    first_node_to_replicate: Byte,
    node_iterator: Byte,
    min: Byte,
    max: Byte,
) -> Byte {
    let nodes_range = max - min;
    min + ((first_node_to_replicate - min + node_iterator) % nodes_range)
}

/// Manda un mensaje a un nodo específico.
pub fn send_to_node(id: NodeId, bytes: Vec<Byte>, port_type: PortType) -> Result<()> {
    let addr = AddrLoader::default_loaded().get_socket(&id, &port_type)?;
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
    wait_response: bool,
) -> Result<Vec<Byte>> {
    let addr = AddrLoader::default_loaded().get_socket(&id, &port_type)?;
    let mut stream = match TcpStream::connect(addr) {
        Ok(tcpstream) => tcpstream,
        Err(_) => {
            return Err(Error::ServerError(format!(
                "No se pudo conectar al nodo con ID {}",
                id
            )))
        }
    };
    println!("Le escribe al nodo: {} la data: {:?}", id, bytes);

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

    if wait_response {
        println!("empieza a esperar respuesta");
        match stream.read_to_end(&mut buf) {
            Err(err) => println!("Error recibiendo response de un nodo:\n\n{}", err),
            Ok(i) => {
                print!("Se recibió del nodo [{}] {} bytes: [ ", id, i);
                for byte in &buf[..] {
                    print!("{:#X} ", byte);
                }
                println!("]");
            }
        }
    }

    Ok(buf)
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
pub fn queries_from_source(path: &str) -> Result<Vec<String>> {
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

/// Carga todas las _queries_ iniciales de la carpeta correspondiente.
pub fn load_init_queries() -> Vec<String> {
    let mut queries = Vec::<String>::new();
    let mut queries_paths = Vec::<PathBuf>::new();

    match read_dir(INIT_QUERIES_PATH) {
        Err(err) => {
            println!("Ocurrió un error al buscar las queries iniciales:\n\n{}\nSe utilizará un vector vacío.", err);
        }
        Ok(paths) => {
            for dir_entry in paths.map_while(IOResult::ok) {
                let path = dir_entry.path();
                if !path.exists() || path.is_dir() {
                    continue;
                }

                if let Some(ext) = path.extension() {
                    if ext.eq_ignore_ascii_case(QUERY_EXT) {
                        queries_paths.push(path);
                    }
                }
            }
        }
    };

    // para asegurar el orden
    queries_paths.sort();

    for path in queries_paths {
        let path_str = match path.to_str() {
            Some(utf_8_valid) => utf_8_valid,
            None => {
                // El nombre contiene caracteres no encodificables en UTF-8
                continue;
            }
        };
        let mut cur_queries = match queries_from_source(path_str) {
            Ok(valid_ones) => valid_ones,
            Err(err) => {
                println!(
                    "No se pudo agregar las queries en '{}':\n\n{}",
                    path_str, err
                );
                continue;
            }
        };
        queries.append(&mut cur_queries);
    }

    queries
}

/// Convierte un hashmap a un string.
pub fn hashmap_to_string<T: ToString, R: ToString>(map: &HashMap<T, R>) -> String {
    let mut res = String::new();

    for (key, value) in map {
        res.push_str(&key.to_string());
        res.push('.');
        res.push_str(&value.to_string());
        res.push(';');
    }
    res.pop();

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
    res.pop();

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
