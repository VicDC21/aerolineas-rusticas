//! Paquete para funciones públicas comunes entre tests de integración.
//!
//! Dichas funciones se definen directamente en este archivo, o sino corremos el riesgo
//! de que cargo crea que los archivos son archivos de tests en sí.

use std::{
    fs::remove_dir_all,
    io::{ErrorKind, Result as IOResult},
    thread::{spawn, JoinHandle},
};

use aerolineas_rusticas::{
    protocol::aliases::results::Result,
    server::nodes::{
        disk_operations::disk_handler::{NODES_METADATA_PATH, STORAGE_PATH},
        graph::NodesGraph,
    },
};

/// Un handle común en nuestra librería.
pub type ThreadHandle<T> = JoinHandle<Result<T>>;

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
///
/// Ignoramos el error específico de si no se encuentra.
pub fn clean_nodes() -> IOResult<()> {
    if let Err(err) = remove_dir_all(STORAGE_PATH) {
        if !matches!(err.kind(), ErrorKind::NotFound) {
            return Err(err);
        }
    }

    if let Err(err) = remove_dir_all(NODES_METADATA_PATH) {
        if !matches!(err.kind(), ErrorKind::NotFound) {
            return Err(err);
        }
    }

    Ok(())
}
