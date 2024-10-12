//! Módulo del cliente.

use std::io::{stdin, BufRead, BufReader, Write};
use std::net::{SocketAddr, TcpStream};

use crate::protocol::aliases::{results::Result, types::Byte};
use crate::protocol::errors::error::Error;
use crate::protocol::traits::Byteable;
use crate::server::actions::opcode::SvAction;

/// Estructura principal de un cliente.
pub struct Client {
    /// La dirección del _socket_ al que conectarse al mandar cosas.
    addr: SocketAddr,
}

impl Client {
    /// Crea una nueva instancia de cliente.
    pub fn new(addr: SocketAddr) -> Self {
        Self { addr }
    }

    /// Conecta con el _socket_ guardado.
    pub fn connect(&self) -> Result<TcpStream> {
        match TcpStream::connect(self.addr) {
            Ok(tcp_stream) => Ok(tcp_stream),
            Err(_) => Err(Error::ServerError(format!(
                "No se pudo conectar al socket '{}'",
                self.addr
            ))),
        }
    }

    /// Conecta con el _socket_ guardado usando `stdin` como _stream_ de entrada.
    ///
    /// <div class="warning">
    ///
    /// **Esto genera un loop infinito** hasta que el usuario ingrese `q` para salir.
    ///
    /// </div>
    pub fn echo(&self) -> Result<()> {
        let mut tcp_stream = self.connect()?;
        let stream = &mut stdin();
        let reader = BufReader::new(stream);

        println!("ECHO MODE:\n----------\nEscribe lo que necesites.\nCuando salgas de este modo, se mandará todo de una al servidor.\n----------\n'q' en una línea sola para salir\n'shutdown' para mandar un mensaje de apagado al servidor\n----------\n");

        for line in reader.lines().map_while(|e| e.ok()) {
            if line.eq_ignore_ascii_case("q") {
                break;
            }
            if line.eq_ignore_ascii_case("shutdown") {
                let _ = tcp_stream.write_all(&SvAction::Exit.as_bytes()[..]);
                break;
            }

            let _ = tcp_stream.write_all(line.as_bytes());
            let _ = tcp_stream.write_all("\n".as_bytes());
        }
        Ok(())
    }

    /// Intenta un objeto al _socket_ guardado.
    pub fn send_bytes(&self, bytes: &[Byte]) -> Result<()> {
        let mut tcp_stream = self.connect()?;

        if tcp_stream.write_all(bytes).is_err() {
            return Err(Error::ServerError(format!(
                "No se pudo escribir el contenido en {}",
                self.addr
            )));
        }
        Ok(())
    }
}
