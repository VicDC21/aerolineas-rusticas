//! Módulo del cliente.

use std::{
    collections::HashSet,
    io::{stdin, BufRead, BufReader, Read, Write},
    net::{SocketAddr, TcpStream},
    time::Duration,
};

use crate::{
    parser::{main_parser::make_parse, statements::statement::Statement},
    protocol::{
        aliases::{results::Result, types::Byte},
        errors::error::Error,
        headers::{flags::Flag, length::Length, opcode::Opcode, stream::Stream, version::Version},
        messages::responses::{
            result::{col_type::ColType, rows_flags::RowsFlag},
            result_kinds::ResultKind,
        },
        traits::Byteable,
        utils::parse_bytes_to_string,
    },
    server::{
        actions::opcode::SvAction, nodes::column_data_type::ColumnDataType,
        utils::get_available_sockets,
    },
    tokenizer::tokenizer::tokenize_query,
};

use crate::client::frame::Frame;

use super::{col_data::ColData, protocol_result::ProtocolResult};

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
#[derive(Clone)]
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

        println!(
            "ECHO MODE:\n \
            ----------\n \
            Escribe tus queries. Cada línea se enviará al presionar Enter.\n \
            ----------\n \
            'q' o línea vacía para salir\n \
            'shutdown' para mandar un mensaje de apagado al servidor (y salir)\n \
            ----------\n"
        );

        let reader = BufReader::new(stdin());
        for line in reader.lines() {
            match line {
                Ok(input) => {
                    if input.eq_ignore_ascii_case("q") || input.is_empty() {
                        break;
                    }
                    if input.eq_ignore_ascii_case("shutdown") {
                        let _ = tcp_stream.write_all(&SvAction::Shutdown.as_bytes());
                        return Ok(());
                    }

                    match self.send_query(&input, &mut tcp_stream) {
                        Ok(_) => (),
                        Err(e) => {
                            eprintln!("Error al enviar la query: {}", e);
                            // Intentar reconectar solo si es un error de conexión
                            match e {
                                Error::ServerError(msg) if msg.contains("conexión") => {
                                    match self.connect() {
                                        Ok(new_stream) => {
                                            let _ = new_stream.set_nonblocking(true);
                                            tcp_stream = new_stream;
                                            if let Err(retry_err) =
                                                self.send_query(&input, &mut tcp_stream)
                                            {
                                                eprintln!(
                                                    "Error al reintentar la query: {}",
                                                    retry_err
                                                );
                                            }
                                        }
                                        Err(e) => {
                                            eprintln!("No se pudo reconectar: {}", e);
                                            break;
                                        }
                                    }
                                }
                                _ => eprintln!("Error en la query: {}", e),
                            }
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

    /// Envía una query al servidor y devuelve la respuesta del mismo.
    pub fn send_query(
        &mut self,
        query: &str,
        tcp_stream: &mut TcpStream,
    ) -> Result<ProtocolResult> {
        let mut stream_id: i16 = 0;
        while self.requests_stream.contains(&stream_id) {
            stream_id += 1;
        }
        self.requests_stream.insert(stream_id);

        match make_parse(&mut tokenize_query(query)) {
            Ok(statement) => {
                // Crear el frame adecuado según el tipo de statement
                let frame = match statement {
                    Statement::DmlStatement(_) => Frame::query(stream_id, query.to_string()),
                    Statement::DdlStatement(_) => Frame::ddl(stream_id, query.to_string()),
                    Statement::UdtStatement(_) => {
                        return Err(Error::ServerError("UDT statements no soportados".into()))
                    }
                };

                // Enviar el frame
                let _ = tcp_stream.write_all(&frame.as_bytes());
                let _ = tcp_stream.flush();

                // Buffer para la respuesta
                let mut response = Vec::new();
                let mut buffer = [0; 1024];
                let mut timeout_count = 0;
                const MAX_TIMEOUT: u32 = 50;

                // Leer la respuesta con timeout
                loop {
                    match tcp_stream.read(&mut buffer) {
                        Ok(0) => {
                            if response.is_empty() {
                                self.requests_stream.remove(&stream_id);
                                return Err(Error::ServerError(
                                    "Conexión cerrada por el servidor sin recibir datos".into(),
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
                            std::thread::sleep(Duration::from_millis(100));
                            timeout_count += 1;
                            if timeout_count >= MAX_TIMEOUT {
                                self.requests_stream.remove(&stream_id);
                                return Err(Error::ServerError(
                                    "Timeout esperando respuesta".into(),
                                ));
                            }
                            continue;
                        }
                        Err(e) => {
                            self.requests_stream.remove(&stream_id);
                            return Err(Error::ServerError(e.to_string()));
                        }
                    }
                }
                // Procesar la respuesta
                if !response.is_empty() {
                    return self.handle_response(&response);
                }

                self.requests_stream.remove(&stream_id);
                Ok(ProtocolResult::Void)
            }
            Err(err) => {
                self.requests_stream.remove(&stream_id);
                Err(Error::ServerError(err.to_string()))
            }
        }
    }

    fn is_response_complete(response: &[u8]) -> bool {
        if response.len() < 9 {
            return false;
        }
        let body_length =
            u32::from_be_bytes([response[5], response[6], response[7], response[8]]) as usize;

        response.len() >= 9 + body_length
    }

    fn handle_response(&mut self, request: &[Byte]) -> Result<ProtocolResult> {
        if request.len() < 9 {
            return Err(Error::ProtocolError(
                "No se cumple el protocolo del header".to_string(),
            ));
        };
        let _version = Version::try_from(request[0])?;
        let _flags = Flag::try_from(request[1])?;
        let _stream = Stream::try_from(request[2..4].to_vec())?;
        let opcode = Opcode::try_from(request[4])?;
        let lenght = Length::try_from(request[5..9].to_vec())?;

        let result: Result<ProtocolResult> = match opcode {
            Opcode::RequestError => self.handle_request_error(lenght, request),
            Opcode::Ready => self.handle_ready(),
            Opcode::Authenticate => self.handle_authenticate(),
            Opcode::Supported => self.handle_supported(),
            Opcode::Result => self.handle_result(lenght, request),
            Opcode::Event => self.handle_event(),
            Opcode::AuthChallenge => self.handle_auth_challenge(),
            Opcode::AuthSuccess => self.handle_auth_success(),
            _ => {
                return Err(Error::ProtocolError(
                    "El opcode recibido no es una response".to_string(),
                ))
            }
        };
        result
    }

    fn handle_request_error(&self, _lenght: Length, request: &[Byte]) -> Result<ProtocolResult> {
        match Error::try_from(request[9..].to_vec()) {
            Ok(error) => Ok(ProtocolResult::QueryError(error)),
            Err(err) => Err(err),
        }
    }

    fn handle_ready(&self) -> Result<ProtocolResult> {
        todo!()
    }

    fn handle_authenticate(&self) -> Result<ProtocolResult> {
        todo!()
    }

    fn handle_supported(&self) -> Result<ProtocolResult> {
        todo!()
    }

    fn handle_result(&self, lenght: Length, request: &[Byte]) -> Result<ProtocolResult> {
        match ResultKind::try_from(request.to_vec())? {
            ResultKind::Void => Ok(ProtocolResult::Void),
            ResultKind::Rows => self.deserialize_rows(lenght, &request[13..]),
            ResultKind::SetKeyspace => self.set_keyspace(lenght, &request[13..]),
            ResultKind::Prepared => todo!(),
            ResultKind::SchemaChange => todo!(),
        }
    }

    fn handle_event(&self) -> Result<ProtocolResult> {
        todo!()
    }

    fn handle_auth_challenge(&self) -> Result<ProtocolResult> {
        todo!()
    }

    fn handle_auth_success(&self) -> Result<ProtocolResult> {
        todo!()
    }

    fn deserialize_rows(&self, _lenght: Length, request: &[Byte]) -> Result<ProtocolResult> {
        let _flags = RowsFlag::try_from(request[..4].to_vec())?;
        let columns_count = u32::from_be_bytes([request[4], request[5], request[6], request[7]]);
        let mut actual_position: usize = 8;
        let mut col_names: Vec<String> = Vec::new(); // usar col_names
        let mut col_types: Vec<ColType> = Vec::new(); // usar col_types que deberia ser ademas ColumnDataType
        for _ in 0..columns_count {
            let mut displacement: usize = 0;
            let col_name = parse_bytes_to_string(&request[actual_position..], &mut displacement)?;
            col_names.push(col_name);
            col_types.push(ColType::try_from(&request[actual_position..])?);
            actual_position += displacement + 2;
        }
        // <rows_count><rows_content>
        let rows_count = i32::from_be_bytes([
            request[actual_position],
            request[actual_position + 1],
            request[actual_position + 2],
            request[actual_position + 3],
        ]);
        actual_position += 4;
        let mut rows: Vec<Vec<ColData>> = Vec::new(); // usar las filas ya parseadas
        for _ in 0..rows_count {
            let mut columns: Vec<ColData> = Vec::new();
            for i in 0..columns_count {
                let col_data = match ColumnDataType::from(col_types[i as usize].clone()) {
                    ColumnDataType::String => {
                        let string_len = self.get_lenght(request, actual_position);
                        actual_position += 4;
                        let right_position = actual_position + string_len as usize;
                        let value_string = match String::from_utf8(
                            request[actual_position..right_position].to_vec(),
                        ) {
                            Ok(value) => value,
                            Err(_err) => {
                                return Err(Error::TruncateError(
                                    "Error al transformar bytes a utf8".to_string(),
                                ))
                            }
                        };
                        ColData::String(value_string)
                    }
                    ColumnDataType::Timestamp => {
                        let timestamp_len = self.get_lenght(request, actual_position);
                        actual_position += 4;
                        let right_position = actual_position + timestamp_len as usize;
                        let value_timestamp =
                            match std::str::from_utf8(&request[actual_position..right_position]) {
                                Ok(value) => match value.parse::<i64>() {
                                    Ok(parsed_value) => parsed_value,
                                    Err(_err) => {
                                        return Err(Error::TruncateError(
                                            "Error al parsear string a i64".to_string(),
                                        ))
                                    }
                                },
                                Err(_err) => {
                                    return Err(Error::TruncateError(
                                        "Error al transformar bytes a utf8".to_string(),
                                    ))
                                }
                            };
                        ColData::Timestamp(value_timestamp)
                    }
                    ColumnDataType::Double => {
                        let double_len = self.get_lenght(request, actual_position);
                        actual_position += 4;
                        let right_position = actual_position + double_len as usize;
                        let value_double =
                            match std::str::from_utf8(&request[actual_position..right_position]) {
                                Ok(value) => match value.parse::<f64>() {
                                    Ok(parsed_value) => parsed_value,
                                    Err(_err) => {
                                        return Err(Error::TruncateError(
                                            "Error al parsear string a f64".to_string(),
                                        ))
                                    }
                                },
                                Err(_err) => {
                                    return Err(Error::TruncateError(
                                        "Error al transformar bytes a utf8".to_string(),
                                    ))
                                }
                            };
                        ColData::Double(value_double)
                    }
                    ColumnDataType::Int => {
                        let int_len = self.get_lenght(request, actual_position);
                        actual_position += 4;
                        let right_position = actual_position + int_len as usize;
                        let value_int =
                            match std::str::from_utf8(&request[actual_position..right_position]) {
                                Ok(value) => match value.parse::<i32>() {
                                    Ok(parsed_value) => parsed_value,
                                    Err(_err) => {
                                        return Err(Error::TruncateError(
                                            "Error al parsear string a i32".to_string(),
                                        ))
                                    }
                                },
                                Err(_err) => {
                                    return Err(Error::TruncateError(
                                        "Error al transformar bytes a utf8".to_string(),
                                    ))
                                }
                            };
                        ColData::Int(value_int)
                    }
                };

                columns.push(col_data);
            }
            rows.push(columns);
        }

        Ok(ProtocolResult::Rows(rows))
    }

    fn set_keyspace(&self, lenght: Length, request: &[Byte]) -> Result<ProtocolResult> {
        match String::from_utf8(request[0..lenght.len as usize].to_vec()) {
            Ok(value) => Ok(ProtocolResult::SetKeyspace(value)),
            Err(_err) => Err(Error::TruncateError(
                "Error al transformar bytes a utf8".to_string(),
            )),
        }
    }

    fn get_lenght(&self, request: &[Byte], actual_position: usize) -> i32 {
        i32::from_be_bytes([
            request[actual_position],
            request[actual_position + 1],
            request[actual_position + 2],
            request[actual_position + 3],
        ])
    }
}

impl Default for Client {
    fn default() -> Self {
        Self::new(get_available_sockets(), HashSet::<i16>::new())
    }
}
