//! Módulo de servidor.
use std::io::Read;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, TcpListener};
use std::thread::JoinHandle;

use crate::protocol::aliases::{results::Result, types::Byte};
use crate::protocol::errors::error::Error;
use crate::server::modes::ConnectionMode;
use crate::server::nodes::graph::NodeGraph;

/// Estructura principal de servidor.
pub struct Server {
    /// El endpoint del servidor.
    addr: SocketAddr,

    /// El grafo de nodos.
    graph: NodeGraph,
}

impl Server {
    /// Crea una nueva instancia del servidor.
    pub fn new(addr: SocketAddr, mode: ConnectionMode) -> Self {
        Self {
            addr,
            graph: NodeGraph::with_mode(mode),
        }
    }

    /// Genera el endpoint preferido del servidor.
    pub fn default_addr() -> SocketAddr {
        SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 8080))
    }

    /// Crea una instancia del servidor en modo de DEBUG.
    pub fn echo_mode() -> Self {
        Self::new(Self::default_addr(), ConnectionMode::Echo)
    }

    /// Crea una instancia del servidor en modo para parsear _queries_.
    pub fn parsing_mode() -> Self {
        Self::new(Self::default_addr(), ConnectionMode::Parsing)
    }

    /// Escucha por los eventos que recibe.
    pub fn listen(&mut self) -> Result<()> {
        let listener = match TcpListener::bind(self.addr) {
            Ok(tcp_listener) => tcp_listener,
            Err(_) => {
                return Err(Error::ServerError(format!(
                    "No se pudo bindear a la dirección '{}'",
                    self.addr
                )))
            }
        };

        let mut handlers: Vec<JoinHandle<Result<()>>> = Vec::new();
        handlers.push(self.graph.beater()?);
        handlers.extend(self.graph.bootup(5)?);

        for tcp_stream_res in listener.incoming() {
            match tcp_stream_res {
                Err(_) => {
                    return Err(Error::ServerError(
                        "Un cliente no pudo conectarse".to_string(),
                    ))
                }
                Ok(tcp_stream) => {
                    let bytes: Vec<Byte> = tcp_stream.bytes().flatten().collect();
                    self.graph.send_message(bytes)?;
                }
            }
        }

        for handler in handlers {
            if handler.join().is_err() {
                // Un hilo caído NO debería interrumpir el dropping de los demás
                println!("Ocurrió un error mientras se esperaba a que termine un hilo hijo.");
            }
        }

        Ok(())
    }
}
