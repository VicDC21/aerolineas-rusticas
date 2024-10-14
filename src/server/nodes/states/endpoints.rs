//! Módulo para el _Endpoint State_ de un nodo.

use std::cmp::PartialEq;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4};

use crate::protocol::aliases::types::{Byte, Int};
use crate::protocol::errors::error::Error;
use crate::protocol::traits::Byteable;
use crate::protocol::utils::{encode_ipaddr_to_bytes, parse_bytes_to_ipaddr};
use crate::server::modes::ConnectionMode;
use crate::server::nodes::node::NodeId;
use crate::server::nodes::states::{
    application::AppState, appstatus::AppStatus, heartbeat::HeartbeatState, heartbeat::VerType,
};

/// El puerto preferido para las IPs
pub const PORT: u16 = 8080;

/// Las propiedades de un nodo.
#[derive(Clone)]
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
    fn generate_ipaddr(id: NodeId) -> SocketAddr {
        SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, id), PORT))
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
    pub fn with_id(id: NodeId) -> Self {
        Self::new(
            Self::generate_ipaddr(id),
            HeartbeatState::default(),
            AppState::default(),
        )
    }

    /// Crea una instancia dado un ID y modo de conexión.
    pub fn with_id_and_mode(id: NodeId, mode: ConnectionMode) -> Self {
        Self::new(
            Self::generate_ipaddr(id),
            HeartbeatState::default(),
            AppState::new(AppStatus::Bootstrap, mode),
        )
    }

    /// Adivina el ID del nodo a partir de la IP del estado.
    pub fn guess_id(&self) -> NodeId {
        match self.addr.ip() {
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

    /// Compara si el _heartbeat_ de este estado es más nuevo que otro.
    pub fn is_newer(&self, other: &Self) -> bool {
        self.heartbeat > other.heartbeat
    }

    /// Consulta la dirección del _socket_.
    pub fn get_addr(&self) -> &SocketAddr {
        &self.addr
    }

    /// Consulta el estado _Heartbeat_.
    pub fn get_heartbeat(&mut self) -> &HeartbeatState {
        &self.heartbeat
    }

    /// Consulta el estado de aplicación.
    pub fn get_appstate(&self) -> &AppState {
        &self.application
    }

    /// Aumenta el estado de _heartbeat_.
    pub fn beat(&mut self) -> VerType {
        self.heartbeat.beat()
    }
}

impl PartialEq for EndpointState {
    fn eq(&self, other: &Self) -> bool {
        self.addr == other.addr && self.heartbeat.eq(&other.heartbeat)
    }
}

impl Byteable for EndpointState {
    fn as_bytes(&self) -> Vec<Byte> {
        let mut bytes = Vec::new();
        bytes.extend(encode_ipaddr_to_bytes(&self.addr.ip()));
        bytes.extend((self.addr.port() as Int).to_be_bytes());

        bytes.extend(self.heartbeat.as_bytes());
        bytes.extend(self.application.as_bytes());
        bytes
    }
}

impl TryFrom<&[Byte]> for EndpointState {
    type Error = Error;

    fn try_from(bytes: &[Byte]) -> Result<Self, Self::Error> {
        let mut i = 0;

        let ipaddr = parse_bytes_to_ipaddr(bytes, &mut i)?;
        let port = Int::from_be_bytes([bytes[i], bytes[i + 1], bytes[i + 2], bytes[i + 3]]);
        i += 4;

        let addr = SocketAddr::new(ipaddr, port as u16);
        let heartbeat = HeartbeatState::try_from(&bytes[i..])?;
        i += heartbeat.as_bytes().len();

        let application = AppState::try_from(&bytes[i..])?;
        Ok(Self::new(addr, heartbeat, application))
    }
}
