//! Módulo para cargar IPs de nodos.

use std::{
    collections::HashMap,
    fs::OpenOptions,
    io::{BufRead, BufReader, BufWriter, Result as IOResult, Write},
    net::{IpAddr, SocketAddr, SocketAddrV4, SocketAddrV6},
};

use crate::{
    protocol::{aliases::results::Result, errors::error::Error},
    server::nodes::{node::NodeId, port_type::PortType},
};

/// El mapa de los IDs de nodos y sus IPs asociadas.
pub type NodeIPs = HashMap<Option<NodeId>, IpAddr>;

const ADDR_FILE: &str = "node_ips.csv";

/// Un cargador que serializa o deserializa la información sobre  IPs de los nodos.
///
/// También mantiene una relación entre el ID del nodo y la IP del mismo.
#[derive(Clone)]
pub struct AddrLoader {
    /// La ruta al archivo donde están las IPs de nodos.
    path: String,

    /// Un mapa de los nodos cargado en memoria.
    node_ips: Option<NodeIPs>,
}

impl AddrLoader {
    /// Crea una nueva instancia del cargador.
    pub fn new(path: &str, node_ips: Option<NodeIPs>) -> Self {
        Self {
            path: path.to_string(),
            node_ips,
        }
    }

    /// Crea una nueva instancia del cargador, tratando de cargar la info al menos una vez.
    pub fn loaded(path: &str) -> Self {
        let mut unloaded = Self::new(path, None);
        let _ = unloaded.reset();

        unloaded
    }

    /// Crea una nueva instancia del cargador, tratando de cargar la info al menos una vez.
    ///
    /// Utiliza la ruta predeterminada.
    pub fn default_loaded() -> Self {
        Self::loaded(ADDR_FILE)
    }

    /// Carga el mapa de IDs de nodos más las IPs.
    pub fn load(&self) -> Result<NodeIPs> {
        let mut node_ips = NodeIPs::new();

        let file = match OpenOptions::new().write(false).read(true).open(&self.path) {
            Ok(exists) => exists,
            Err(io_err) => {
                return Err(Error::ServerError(format!(
                    "Archivo en '{}' no se pudo abrir o no existe:\n\n{}",
                    &self.path, io_err
                )));
            }
        };
        let bufreader = BufReader::new(file);

        for line in bufreader.lines().skip(1).map_while(IOResult::ok) {
            let splitted = line.trim().split(",").collect::<Vec<&str>>();
            if splitted.len() != 2 {
                continue;
            }
            let node_id_str = splitted[0];
            let ip_str = splitted[1];

            let node_id = match node_id_str.parse::<NodeId>() {
                Ok(valid) => Some(valid),
                Err(_) => None,
            };
            let ip = match ip_str.parse::<IpAddr>() {
                Ok(valid) => valid,
                Err(parse_err) => {
                    return Err(Error::ServerError(format!(
                        "IP de nodo malformada. '{}' no es un valor válido:\n\n{}",
                        ip_str, parse_err
                    )));
                }
            };

            node_ips.insert(node_id, ip);
        }

        Ok(node_ips)
    }

    /// Intenta volver a cargar en memoria la info de IPs.
    pub fn reset(&mut self) -> Result<()> {
        self.node_ips = Some(self.load()?);
        Ok(())
    }

    /// Guarda en disco la configuración actual.
    ///
    /// <div class="warning">
    ///
    /// _No hay garantía_ de que las filas se escriban siempre en el mismo orden.
    ///
    /// </div>
    pub fn save(&self) -> Result<()> {
        if let Some(node_ips) = &self.node_ips {
            let file = match OpenOptions::new()
                .create(true)
                .truncate(true)
                .write(true)
                .read(true)
                .open(&self.path)
            {
                Ok(opened) => opened,
                Err(io_err) => {
                    return Err(Error::ServerError(format!(
                        "No se pudo abrir el archivo en '{}':\n\n{}",
                        &self.path, io_err
                    )));
                }
            };
            let mut bufwriter = BufWriter::new(file);

            let _ = bufwriter.write_all("node_id,ip".as_bytes());
            for (node_id, ip) in node_ips {
                let node_id_str = match node_id {
                    Some(id) => id.to_string(),
                    None => "".to_string(),
                };
                let _ = bufwriter.write_all(format!("\n{},{}", node_id_str, ip).as_bytes());
            }
        }

        Ok(())
    }

    /// Carga las IPs de nodos, descartando los IDs.
    pub fn get_ips(&self) -> Vec<IpAddr> {
        let mut ips = Vec::<IpAddr>::new();

        if let Some(ip_values) = &self.node_ips {
            for ip in ip_values.values() {
                ips.push(*ip);
            }
        }

        ips
    }

    /// Carga los _sockets_ con un tipo de purto de [cliente](PortType::Cli).
    pub fn get_sockets_cli(&self) -> Vec<SocketAddr> {
        self.get_sockets(&PortType::Cli)
    }

    /// Carga los _sockets_ con un tipo de purto [privado](PortType::Priv).
    pub fn get_sockets_priv(&self) -> Vec<SocketAddr> {
        self.get_sockets(&PortType::Priv)
    }

    /// Carga los _sockets_ con un tipo de puerto dado.
    fn get_sockets(&self, port_type: &PortType) -> Vec<SocketAddr> {
        let mut sockets = Vec::<SocketAddr>::new();

        if let Some(node_ips) = &self.node_ips {
            for ip in node_ips.values() {
                sockets.push(Self::ip_to_socket(ip, port_type));
            }
        }

        sockets
    }

    /// Convierte un [IpAddr] a un [SocketAddr] según un [tipo](PortType) de puerto dado.
    pub fn ip_to_socket(ip: &IpAddr, port_type: &PortType) -> SocketAddr {
        match ip {
            IpAddr::V4(ipv4) => SocketAddr::V4(SocketAddrV4::new(*ipv4, port_type.to_num())),
            IpAddr::V6(ipv6) => SocketAddr::V6(SocketAddrV6::new(*ipv6, port_type.to_num(), 0, 0)),
        }
    }
}

impl Default for AddrLoader {
    fn default() -> Self {
        Self::new(ADDR_FILE, None)
    }
}
