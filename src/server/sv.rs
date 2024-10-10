//! Módulo de servidor.

use std::io::{BufRead, BufReader, Read};
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, TcpListener};

use crate::protocol::aliases::results::Result;
use crate::protocol::errors::error::Error;
use crate::server::sv_modes::ServerMode;

/// Corrida de prueba para un servidor.
pub fn run() -> std::io::Result<()> {
    let socket_addr = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 8080);
    let listener = TcpListener::bind(socket_addr)?;

    // bloquea hasta que le entra un request
    let (mut client_stream, socket_addr) = listener.accept()?;

    println!("La socket addr del client: {:?}", socket_addr);
    handle_client(&mut client_stream)?;
    Ok(())
}

fn handle_client(stream: &mut dyn Read) -> std::io::Result<()> {
    let reader = BufReader::new(stream);
    let mut lines = reader.lines();
    // iteramos las lineas que recibimos de nuestro cliente
    while let Some(Ok(line)) = lines.next() {
        println!("Recibido: {:?}", line);
    }
    Ok(())
}

/// Estructura principal de servidor.
pub struct Server {
    /// El endpoint del servidor.
    addr: SocketAddr,

    /// El modo de conexión al servidor.
    mode: ServerMode,
}

impl Server {
    /// Crea una nueva instancia del servidor.
    pub fn new(addr: SocketAddr, mode: ServerMode) -> Self {
        Self { addr, mode }
    }

    /// Genera el endpoint preferido del servidor.
    pub fn default_addr() -> SocketAddr {
        SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 8080))
    }

    /// Crea una instancia del servidor en modo de DEBUG.
    pub fn echo_mode() -> Self {
        Self::new(Self::default_addr(), ServerMode::Echo)
    }

    /// Crea una instancia del servidor en modo para parsear _queries_.
    pub fn parsing_mode() -> Self {
        Self::new(Self::default_addr(), ServerMode::Parsing)
    }

    /// Escucha por los eventos que recibe.
    pub fn listen(&self) -> Result<()> {
        let listener = match TcpListener::bind(self.addr) {
            Ok(tcp_listener) => tcp_listener,
            Err(_) => {
                return Err(Error::ServerError(format!(
                    "No se pudo bindear a la dirección '{}'",
                    self.addr
                )))
            }
        };

        for tcp_stream_res in listener.incoming() {
            match tcp_stream_res {
                Err(_) => {
                    return Err(Error::ServerError(
                        "Un cliente no pudo conectarse".to_string(),
                    ))
                }
                Ok(tcp_stream) => {
                    match self.mode {
                        ServerMode::Echo => {
                            let reader = BufReader::new(tcp_stream);
                            let mut lines = reader.lines();
                            // iteramos las lineas que recibimos de nuestro cliente
                            while let Some(Ok(line)) = lines.next() {
                                println!("echo:\t{:?}", line);
                            }
                        }
                        ServerMode::Parsing => {
                            // Delegar quilombo al grafo de nodos
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
