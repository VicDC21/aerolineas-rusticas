//! Módulo para funciones auxiliares del servidor.

use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

use crate::server::nodes::{
    graph::{N_NODES, START_ID},
    port_type::PortType,
};

/// Consigue las direcciones a las que intentará conectarse.
///
/// ~~_(Medio hardcodeado pero sirve por ahora)_~~
pub fn get_available_sockets() -> Vec<SocketAddr> {
    let mut sockets = Vec::<SocketAddr>::new();
    for i in 0..N_NODES {
        sockets.push(SocketAddr::V4(SocketAddrV4::new(
            Ipv4Addr::new(127, 0, 0, START_ID + i),
            PortType::Cli.into(),
        )));
    }
    sockets
}
