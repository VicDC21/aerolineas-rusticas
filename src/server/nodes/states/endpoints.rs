//! Módulo para el _Endpoint State_ de un nodo.

use std::{
    cmp::PartialEq,
    net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4, SocketAddrV6},
};

use crate::protocol::{
    aliases::types::Byte,
    errors::error::Error,
    traits::Byteable,
    utils::{encode_ipaddr_to_bytes, parse_bytes_to_ipaddr},
};
use crate::server::{
    modes::ConnectionMode,
    nodes::{
        node::NodeId,
        port_type::PortType,
        states::{
            application::AppState, appstatus::AppStatus, heartbeat::HeartbeatState,
            heartbeat::VerType,
        },
    },
};

/// Las propiedades de un nodo.
#[derive(Clone)]
pub struct EndpointState {
    /// La dirección de _socket_ del nodo.
    ipaddr: IpAddr,

    /// La info de un nodo que cambia a cada instante.
    heartbeat: HeartbeatState,

    /// Otra información relacionada al nodo.
    application: AppState,
}

impl EndpointState {
    /// Genera un socket basado en un id dado.
    fn generate_ipaddr(id: NodeId) -> IpAddr {
        IpAddr::V4(Ipv4Addr::new(127, 0, 0, id))
    }

    /// Instancia las propiedades del nodo.
    pub fn new(ipaddr: IpAddr, heartbeat: HeartbeatState, application: AppState) -> Self {
        Self {
            ipaddr,
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
        Self {
            application: AppState::new(AppStatus::Bootstrap, mode),
            ..Self::with_id(id)
        }
    }

    /// Compara si el _heartbeat_ de este estado es más nuevo que otro.
    pub fn is_newer(&self, other: &Self) -> bool {
        self.heartbeat > other.heartbeat
    }

    /// Consulta la dirección de la IP.
    pub fn get_addr(&self) -> &IpAddr {
        &self.ipaddr
    }

    /// Consulta el estado _heartbeat_.
    pub fn get_heartbeat(&self) -> &HeartbeatState {
        &self.heartbeat
    }

    /// Devuelve una copia del estado _heartbeat_.
    pub fn clone_heartbeat(&self) -> HeartbeatState {
        self.heartbeat.clone()
    }

    /// Consulta el estado de aplicación.
    pub fn get_appstate(&self) -> &AppState {
        &self.application
    }

    /// Gets a socket depending of the selected port.
    pub fn socket(&self, port_type: &PortType) -> SocketAddr {
        let ipaddr = self.ipaddr.clone();
        match ipaddr {
            IpAddr::V4(ipv4) => SocketAddr::V4(SocketAddrV4::new(ipv4, port_type.into())),
            IpAddr::V6(ipv6) => SocketAddr::V6(SocketAddrV6::new(ipv6, port_type.into(), 0, 0)),
        }
    }

    /// Aumenta el estado de _heartbeat_.
    pub fn beat(&mut self) -> VerType {
        self.heartbeat.beat()
    }
}

impl PartialEq for EndpointState {
    fn eq(&self, other: &Self) -> bool {
        self.ipaddr == other.ipaddr && self.heartbeat.eq(&other.heartbeat)
    }
}

impl Byteable for EndpointState {
    fn as_bytes(&self) -> Vec<Byte> {
        let mut bytes = Vec::new();
        bytes.extend(encode_ipaddr_to_bytes(&self.ipaddr));

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

        let heartbeat = HeartbeatState::try_from(&bytes[i..])?;
        i += heartbeat.as_bytes().len();

        let application = AppState::try_from(&bytes[i..])?;
        Ok(Self::new(ipaddr, heartbeat, application))
    }
}
