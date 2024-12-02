//! Paquete para funciones públicas comunes entre tests de integración.
//!
//! Dichas funciones se definen directamente en este archivo, o sino corremos el riesgo
//! de que cargo crea que los archivos son archivos de tests en sí.
#![allow(dead_code)] // Las funciones sí se usan, pero no lo descubre por no estar en la lib

use std::{
    fs::remove_dir_all,
    io::{ErrorKind, Result as IOResult},
    thread::{sleep, spawn, JoinHandle},
    time::Duration,
};

use aerolineas_rusticas::{
    protocol::aliases::results::Result,
    server::nodes::{graph::NodesGraph, node::Node},
};

/// Un handle común en nuestra librería.
pub type ThreadHandle<T> = JoinHandle<Result<T>>;
/// Lista de handles simples.
pub type HandlesVec = Vec<Option<JoinHandle<()>>>;

/// La ruta para el almacenamiento de las keyspaces y tablas de los nodos.
pub const STORAGE_PATH: &str = "storage";
/// La ruta para el almacenamiento de los metadatos de los nodos.
pub const NODES_METADATA_PATH: &str = "nodes_metadata";

/// Crea un [grafo](NodesGraph) en modo de [DEBUG](aerolineas_rusticas::server::modes::ConnectionMode::Echo)
/// y lo corre en un hilo aparte.
pub fn init_graph_echo() -> ThreadHandle<()> {
    let mut echo_graph = NodesGraph::echo_mode();
    spawn(move || echo_graph.init())
}

/// Crea un [grafo](NodesGraph) en modo de [PARSING](aerolineas_rusticas::server::modes::ConnectionMode::Parsing)
/// y lo corre en un hilo aparte.
pub fn init_graph_parsing() -> ThreadHandle<()> {
    let mut parsing_graph = NodesGraph::parsing_mode();
    spawn(move || parsing_graph.init())
}

/// Borra todos los archivos y directorios de metadatos relevantes,
/// tal que quede limpio de corridas anteriores.
pub fn clean_nodes() -> IOResult<()> {
    rmdir(STORAGE_PATH)?;
    rmdir(NODES_METADATA_PATH)?;

    Ok(())
}

/// Remueve un directorio, e ignora el error si el mismo no existe.
fn rmdir(path: &str) -> IOResult<()> {
    if let Err(err) = remove_dir_all(path) {
        if !matches!(err.kind(), ErrorKind::NotFound) {
            return Err(err);
        }
    }

    Ok(())
}

/// Crea una lista de nodos en modo ECHO.
pub fn create_echo_nodes(nodes: u8, duration: Duration) -> HandlesVec {
    let mut handles = HandlesVec::with_capacity(nodes as usize);
    for i in 0..nodes {
        handles.push(Some(spawn(move || {
            if let Err(err) = Node::init_in_echo_mode(10 + i) {
                println!("Error:\n{}", err);
            };
        })));
        sleep(duration);
    }

    handles
}

/// Crea una lista de nodos en modo PARSING.
pub fn create_parsing_nodes(nodes: u8, duration: Duration) -> HandlesVec {
    let mut handles = HandlesVec::with_capacity(nodes as usize);
    for i in 0..nodes {
        handles.push(Some(spawn(move || {
            if let Err(err) = Node::init_in_parsing_mode(10 + i) {
                println!("Error:\n{}", err);
            };
        })));
        sleep(duration);
    }

    handles
}

/// Espera a que todos los handles terminen.
pub fn handles_join(handles: &mut HandlesVec) {
    for handle_opt in handles {
        if let Some(handle) = handle_opt.take() {
            let _ = handle.join();
        }
    }
}
