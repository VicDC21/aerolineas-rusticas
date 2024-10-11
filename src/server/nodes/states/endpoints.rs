//! Módulo para el _Endpoint State_ de un nodo.

use std::cmp::{Ordering, PartialEq, PartialOrd};
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

use crate::server::nodes::states::{application::AppState, heartbeat::HeartbeatState};

/// Las propiedades de un nodo.
pub struct EndpointState {
    /// La dirección de _socket_ del nodo.
    addr: SocketAddr,

    /// La info de un nodo que cambia a cada instante.
    heartbeat: HeartbeatState,

    /// Otra información relacionada al nodo.
    application: AppState,
}

impl EndpointState {
    /// Instancia las propiedades del nodo.
    pub fn new(addr: SocketAddr, heartbeat: HeartbeatState, application: AppState) -> Self {
        Self {
            addr,
            heartbeat,
            application,
        }
    }

    /// Crea una instancia dado un ID.
    pub fn with_id(id: u8) -> Self {
        Self::new(
            SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, id), 8080)),
            HeartbeatState::default(),
            AppState::default(),
        )
    }
}

impl PartialEq for EndpointState {
    fn eq(&self, other: &Self) -> bool {
        self.heartbeat.eq(&other.heartbeat)
    }
}

impl PartialOrd for EndpointState {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.heartbeat.partial_cmp(&other.heartbeat)
    }
}
