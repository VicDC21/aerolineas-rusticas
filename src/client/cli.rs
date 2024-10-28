//! Módulo del cliente.

use std::{
    collections::HashSet,
    io::{stdin, BufRead, BufReader, Read, Write},
    net::{SocketAddr, TcpStream},
    time::Duration,
};

use crate::{
    parser::{main_parser::make_parse, statements::statement::Statement},
    protocol::{aliases::results::Result, errors::error::Error, traits::Byteable},
    server::{actions::opcode::SvAction, utils::get_available_sockets},
    tokenizer::tokenizer::tokenize_query,
};

use crate::client::frame::Frame;

/// Flags específicas para queries CQL
#[derive(Clone, Copy)]
pub enum QueryFlags {
    /// Para vincular valores a la query
    Values = 0x01,
    /// Si se quiere saltar los metadatos en la respuesta
    SkipMetadata = 0x02,
    /// Tamaño deseado de la página si se setea
    PageSize = 0x04,
    /// Estado de paginación
    WithPagingState = 0x08,
    /// Consistencia serial para actualizaciones de datos condicionales
    WithSerialConsistency = 0x10,
    /// Timestamp por defecto (en microsegundos)
    WithDefaultTimestamp = 0x20,
    /// Solo tiene sentido si se usa `Values`, para tener nombres de columnas en los valores
    WithNamesForValues = 0x40,
    /// Keyspace donde debe ejecutarse la query
    WithKeyspace = 0x80,
    /// Tiempo actual en segundos
    WithNowInSeconds = 0x100,
}

/// Estructura principal de un cliente.
pub struct Client {
    /// La dirección del _socket_ al que conectarse al mandar cosas.
    addrs: Vec<SocketAddr>,
    requests_stream: HashSet<i16>,
}

impl Client {
    /// Crea una nueva instancia de cliente.
    pub fn new(addrs: Vec<SocketAddr>, requests_stream: HashSet<i16>) -> Self {
        Self {
            addrs,
            requests_stream,
        }
    }

    /// Conecta con alguno de los _sockets_ guardados.
    pub fn connect(&self) -> Result<TcpStream> {
        Self::connect_to(&self.addrs[..])
    }

    /// Conecta con alguno de los _sockets_ dados.
    pub fn connect_to(sockets: &[SocketAddr]) -> Result<TcpStream> {
        match TcpStream::connect(sockets) {
            Ok(tcp_stream) => Ok(tcp_stream),
            Err(_) => Err(Error::ServerError(
                "No se pudo conectar con ningún socket.".to_string(),
            )),
        }
    }

    /// Conecta con alguno de los _sockets_ guardados usando `stdin` como _stream_ de entrada.
    ///
    /// <div class="warning">
    ///
    /// **Esto genera un loop infinito** hasta que el usuario ingrese `q` para salir.
    ///
    /// </div>

    pub fn echo(&mut self) -> Result<()> {
        let mut tcp_stream = self.connect()?;
        let _ = tcp_stream.set_nonblocking(true);
        let stream = &mut stdin();
        let reader = BufReader::new(stream);

        println!(
            "ECHO MODE:\n \
            ----------\n \
            Escribe tus queries. Cada línea se enviará al presionar Enter.\n \
            ----------\n \
            'q' o línea vacía para salir\n \
            'shutdown' para mandar un mensaje de apagado al servidor (y salir)\n \
            ----------\n"
        );

        for line in reader.lines() {
            match line {
                Ok(input) => {
                    if input.eq_ignore_ascii_case("q") || input.is_empty() {
                        break;
                    }
                    if input.eq_ignore_ascii_case("shutdown") {
                        let _ = tcp_stream.write_all(&SvAction::Shutdown.as_bytes()[..]);
                        return Ok(());
                    }

                    match self.send_query(input, &mut tcp_stream) {
                        Ok(_) => (),
                        Err(e) => {
                            eprintln!("Error al enviar la query: {}", e);
                            tcp_stream = match self.connect() {
                                Ok(stream) => {
                                    let _ = stream.set_nonblocking(true);
                                    stream
                                }
                                Err(e) => {
                                    eprintln!("No se pudo reconectar: {}", e);
                                    break;
                                }
                            };
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error leyendo la entrada: {}", e);
                    break;
                }
            }
        }

        Ok(())
    }

    fn send_query(&mut self, query: String, tcp_stream: &mut TcpStream) -> Result<()> {
        let mut stream_id: i16 = 0;
        while self.requests_stream.contains(&stream_id) {
            stream_id += 1;
        }
        self.requests_stream.insert(stream_id);

        match make_parse(&mut tokenize_query(&query)) {
            Ok(statement) => {
                let frame = match statement {
                    Statement::DmlStatement(_statement) => Frame::query(stream_id, query),
                    Statement::DdlStatement(_statement) => Frame::ddl(stream_id, query),
                    Statement::UdtStatement(_statement) => todo!(),
                };

                let _ = tcp_stream.write_all(&frame.as_bytes());
                let _ = tcp_stream.flush();

                // Buffer para leer la respuesta
                let mut response = Vec::new();
                let mut buffer = [0; 1024];
                let mut timeout_count = 0;
                const MAX_TIMEOUT: u32 = 50; // 5 segundos máximo de espera (50 * 100ms)

                // Leemos la respuesta con timeout
                loop {
                    match tcp_stream.read(&mut buffer) {
                        Ok(0) => {
                            // Conexión cerrada por el servidor
                            if response.is_empty() {
                                return Err(Error::ServerError(
                                    "Conexión cerrada por el servidor".into(),
                                ));
                            }
                            break;
                        }
                        Ok(n) => {
                            response.extend_from_slice(&buffer[..n]);
                            if Self::is_response_complete(&response) {
                                break;
                            }
                        }
                        Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                            // No hay datos disponibles en este momento
                            std::thread::sleep(Duration::from_millis(100));
                            timeout_count += 1;
                            if timeout_count >= MAX_TIMEOUT {
                                return Err(Error::ServerError(
                                    "Timeout esperando respuesta".into(),
                                ));
                            }
                            continue;
                        }
                        Err(e) => return Err(Error::ServerError(e.to_string())),
                    }
                }

                self.requests_stream.remove(&stream_id);
                println!("Received {} bytes: {:?}", response.len(), response);
                Ok(())
            }
            Err(err) => {
                println!("Error parsing query: {}", err);
                Err(Error::ServerError(err.to_string()))
            }
        }
    }

    /// Verifica si la respuesta está completa basándose en el protocolo CQL
    fn is_response_complete(response: &[u8]) -> bool {
        if response.len() < 9 {
            return false;
        }

        // Los primeros 9 bytes contienen el header
        // Los bytes 5-8 contienen la longitud del cuerpo
        let body_length =
            u32::from_be_bytes([response[5], response[6], response[7], response[8]]) as usize;

        // Verificamos si tenemos el header completo + el cuerpo
        response.len() >= 9 + body_length
    }
}

impl Default for Client {
    fn default() -> Self {
        Self::new(get_available_sockets(), HashSet::<i16>::new())
    }
}
