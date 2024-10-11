//! Módulo para el _Endpoint State_ de un nodo.

use std::cmp::{Ordering, PartialEq, PartialOrd};
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

use crate::server::modes::ConnectionMode;
use crate::server::nodes::states::appstatus::AppStatus;
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
    /// Genera un socket basado en un id dado.
    fn generate_ipaddr(id: u8) -> SocketAddr {
        SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, id), 8080))
    }

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
            Self::generate_ipaddr(id),
            HeartbeatState::default(),
            AppState::default(),
        )
    }

    /// Crea una instancia dado un ID y modo de conexión.
    pub fn with_id_and_mode(id: u8, mode: ConnectionMode) -> Self {
        Self::new(
            Self::generate_ipaddr(id),
            HeartbeatState::default(),
            AppState::new(AppStatus::Bootstrap, mode),
        )
    }

    /// Consulta la dirección del _socket_.
    pub fn get_addr(&self) -> &SocketAddr {
        &self.addr
    }

    /// Consulta el estado _Heartbeat_.
    pub fn get_heartbeat(&self) -> &HeartbeatState {
        &self.heartbeat
    }

    /// Consulta el estado de aplicación.
    pub fn get_appstate(&self) -> &AppState {
        &self.application
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
