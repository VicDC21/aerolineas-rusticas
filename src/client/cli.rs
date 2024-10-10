//! Módulo del cliente.

use std::io::{stdin, BufRead, BufReader, Write};
use std::net::{SocketAddr, TcpStream};

use crate::protocol::aliases::{results::Result, types::Byte};
use crate::protocol::errors::error::Error;

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
                "Could not connect to socket '{}'",
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

        for line in reader.lines().map_while(|e| e.ok()) {
            if line.eq_ignore_ascii_case("q") {
                break;
            }

            println!("Enviando: {:?}", line);
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
