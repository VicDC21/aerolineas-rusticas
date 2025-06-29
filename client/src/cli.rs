//! Módulo del cliente.

use {
    data::{col_data::ColData, login_info::LoginInfo, protocol_result::ProtocolResult},
    parser::{main_parser::make_parse, statements::statement::Statement},
    protocol::{
        aliases::{
            results::Result,
            types::{Byte, Double, Int, Long, ShortInt, Uint},
        },
        errors::error::Error,
        headers::{flags::Flag, length::Length, opcode::Opcode, stream::Stream, version::Version},
        messages::responses::{
            result::{col_type::ColType, rows_flags::RowsFlag},
            result_kinds::ResultKind,
        },
        notations::consistency::Consistency,
        traits::Byteable,
        utils::{encode_string_map_to_bytes, encode_string_to_bytes, parse_bytes_to_string},
    },
    rustls::{
        pki_types::{pem::PemObject, CertificateDer},
        ClientConfig, ClientConnection, RootCertStore, StreamOwned as LsStream,
    },
    server::{
        cql_frame::frame::Frame,
        nodes::{
            actions::opcode::SvAction, addr::loader::AddrLoader, port_type::PortType,
            table_metadata::column_data_type::ColumnDataType,
        },
    },
    std::{
        collections::HashSet,
        io::{stdin, BufRead, BufReader, Read, Write},
        net::{SocketAddr, TcpStream},
        str::FromStr,
        sync::Arc,
        time::{Duration, Instant},
    },
    tokenizer::tok::tokenize_query,
    utils::get_root_path::get_root_path,
};

/// Un stream TLS.
pub type TlsStream = LsStream<ClientConnection, TcpStream>;

/// La cantidad máxima de intentos de reconexión.
const MAX_RETRIES: Uint = 2;
/// La cantidad (en bytes) del _header_ de un mensaje.
const HEADER_SIZE: usize = 9;

/// Estructura principal de un cliente.
#[derive(Clone)]
pub struct Client {
    /// El cargador de las direcciones disponibles.
    addr_loader: AddrLoader,

    /// Un contador interno para llevar la cuenta de IDs de conexiones.
    requests_stream: HashSet<ShortInt>,

    /// El _Consistency Level_ de las queries.
    consistency_level: Consistency,

    /// Información de logueo, a usar en caso de necesitar reconectarse.
    login_info: LoginInfo,
}

impl Client {
    /// Crea una nueva instancia de cliente.
    ///
    /// El _Consistency Level_ será `Quorum` por defecto.
    pub fn new(addr_loader: AddrLoader, requests_stream: HashSet<ShortInt>) -> Self {
        Self {
            addr_loader,
            requests_stream,
            consistency_level: Consistency::Quorum,
            login_info: LoginInfo::default(),
        }
    }

    /// Conecta con alguno de los _sockets_ guardados.
    pub fn connect(&self) -> Result<TcpStream> {
        let tcp_stream = Self::connect_to(&self.addr_loader.get_sockets_cli()[..])?;
        tcp_stream
            .set_nonblocking(true)
            .map_err(|e| Error::ServerError(format!("Error al configurar non-blocking: {}", e)))?;
        // Configurar timeouts explícitos
        tcp_stream
            .set_read_timeout(Some(Duration::from_secs(5)))
            .map_err(|e| Error::ServerError(format!("Error al configurar read timeout: {}", e)))?;
        tcp_stream
            .set_write_timeout(Some(Duration::from_secs(5)))
            .map_err(|e| Error::ServerError(format!("Error al configurar write timeout: {}", e)))?;
        Ok(tcp_stream)
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

    /// Modifica el _Consistency Level_ de las queries.
    ///
    /// Tipos de _Consistency Level_ reconocidos:
    /// - `Any` (sin uso)
    /// - `One`
    /// - `Two`
    /// - `Three`
    /// - `Quorum`
    /// - `All`
    /// - `LocalQuorum` (TODO)
    /// - `EachQuorum` (TODO)
    /// - `Serial` (sin uso)
    /// - `LocalSerial` (sin uso)
    /// - `LocalOne` (TODO)
    pub fn set_consistency_level(&mut self, s: &str) -> Result<()> {
        match Consistency::from_str(s) {
            Ok(consistency) => {
                self.consistency_level = consistency;
                Ok(())
            }
            Err(e) => Err(Error::ConfigError(e.to_string())),
        }
    }

    /// Crea una conexion tls
    pub fn create_tls_connection(
        &self,
        client_connection: ClientConnection,
        tcp_stream: TcpStream,
    ) -> Result<TlsStream> {
        tcp_stream
            .set_nonblocking(false)
            .map_err(|e| Error::ServerError(format!("Error al configurar non-blocking: {}", e)))?;
        tcp_stream
            .set_read_timeout(Some(Duration::from_secs(5)))
            .map_err(|e| Error::ServerError(format!("Error al configurar read timeout: {}", e)))?;
        tcp_stream
            .set_write_timeout(Some(Duration::from_secs(5)))
            .map_err(|e| Error::ServerError(format!("Error al configurar write timeout: {}", e)))?;
        Ok(TlsStream::new(client_connection, tcp_stream))
    }

    /// Intenta loguearse con un usuario específico.
    pub fn login(
        &mut self,
        login_info: LoginInfo,
        tls_stream: &mut TlsStream,
    ) -> Result<Option<TlsStream>> {
        if self.login_info != login_info {
            self.login_info = login_info;
        }

        let (protocol_result, tls_opt) = self.send_query(
            format!(
                "User: {} Password: {}",
                &self.login_info.user, &self.login_info.pass
            )
            .as_str(),
            tls_stream,
        )?;

        match protocol_result {
            ProtocolResult::AuthSuccess => Ok(tls_opt),
            ProtocolResult::QueryError(auth_err) => {
                Err(Error::AuthenticationError(format!(
                    "La autenticación con usuario '{}' y contraseña '{}' ha fallado:\n\n{}",
                    &self.login_info.user, &self.login_info.pass, auth_err,
                )))
            },
            _ => {
                Err(Error::AuthenticationError(format!(
                    "La autenticación con usuario '{}' y contraseña '{}' ha fallado.\nSe recibió un resultado de tipo {:?}.",
                    &self.login_info.user, &self.login_info.pass, protocol_result,
                )))
            }
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
        let client_connection = get_client_connection()?;
        let tcp_stream = self.connect()?;
        let tls_stream: TlsStream = self.create_tls_connection(client_connection, tcp_stream)?;
        print_initial_message();
        self.read_console_input(tls_stream)?;
        Ok(())
    }

    /// Lee la consola como input y se encarga de handelear lo que se escriba
    fn read_console_input(
        &mut self,
        mut tls_stream: LsStream<ClientConnection, TcpStream>,
    ) -> Result<()> {
        let reader = BufReader::new(stdin());
        for line in reader.lines() {
            match line {
                Ok(input) => {
                    if input.eq_ignore_ascii_case("q") || input.is_empty() {
                        break;
                    }
                    if input.eq_ignore_ascii_case("shutdown") {
                        self.send_shutdown()?;
                        return Ok(());
                    }
                    match self.send_query(&input, &mut tls_stream) {
                        Ok(res) => {
                            if let ProtocolResult::QueryError(err) = res.0 {
                                println!("{}", err)
                            }
                        }
                        Err(e) => {
                            eprintln!("Error al enviar la query: {}", e);
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
    ///
    /// La query será enviada con el _Consistency Level_ actual.
    pub fn send_query(
        &mut self,
        query: &str,
        tls_stream: &mut TlsStream,
    ) -> Result<(ProtocolResult, Option<TlsStream>)> {
        let mut stream_id: ShortInt = 0;
        while self.requests_stream.contains(&stream_id) {
            stream_id += 1;
        }
        self.requests_stream.insert(stream_id);
        let mut tls_opt: Option<TlsStream> = None;

        let result = match make_parse(&mut tokenize_query(query)) {
            Ok(statement) => {
                let frame = self.prepare_request_to_send(statement, stream_id, query)?;
                let mut last_error = None;
                for _ in 0..=MAX_RETRIES {
                    match self.write_to_server(tls_opt, &frame, tls_stream) {
                        Ok(value) => return Ok(value),
                        Err(e) => last_error = Some(e),
                    };
                    // A este punto sabemos que el TLS Stream algo tiene, hay que cambiarlo
                    let mut new_tls =
                        self.create_tls_connection(get_client_connection()?, self.connect()?)?;
                    self.login(self.login_info.to_owned(), &mut new_tls)?;
                    tls_opt = Some(new_tls);
                }
                match last_error {
                    Some(e) => Err(e),
                    None => Err(Error::ServerError("Error desconocido".into())),
                }
            }
            Err(err) => Err(Error::ServerError(err.to_string())),
        };
        self.requests_stream.remove(&stream_id);
        result
    }

    fn prepare_request_to_send(
        &mut self,
        statement: Statement,
        stream_id: ShortInt,
        query: &str,
    ) -> Result<Vec<Byte>> {
        let frame = match statement {
            Statement::DmlStatement(_) | Statement::DdlStatement(_) => {
                Frame::new(stream_id, query, self.consistency_level).as_bytes()
            }
            Statement::LoginUser(user) => {
                Client::prepare_auth_response_message(&user.user, &user.password)?
            }
            Statement::Startup => Client::prepare_startup_message()?,
        };
        Ok(frame)
    }

    fn write_to_server(
        &mut self,
        mut tls_opt: Option<TlsStream>,
        frame: &[Byte],
        tls_stream: &mut TlsStream,
    ) -> Result<(ProtocolResult, Option<TlsStream>)> {
        if let Some(cur_tls) = tls_opt.as_mut() {
            match cur_tls.write_all(frame) {
                Ok(_) => match cur_tls.flush() {
                    Ok(_) => match self.read_complete_response(cur_tls) {
                        Ok(response) => Ok((response, tls_opt)),
                        Err(e) => Err(e),
                    },
                    Err(e) => Err(Error::ServerError(format!("Error al flush: {}", e))),
                },
                Err(e) => Err(Error::ServerError(format!("Error al escribir: {}", e))),
            }
        } else {
            match tls_stream.write_all(frame) {
                Ok(_) => match tls_stream.flush() {
                    Ok(_) => match self.read_complete_response(tls_stream) {
                        Ok(response) => Ok((response, None)),
                        Err(e) => Err(e),
                    },
                    Err(e) => Err(Error::ServerError(format!("Error al flush: {}", e))),
                },
                Err(e) => Err(Error::ServerError(format!("Error al escribir: {}", e))),
            }
        }
    }

    /// Lee N _bytes_ del stream.
    pub fn read_n_bytes(
        &mut self,
        n_bytes: usize,
        tls_stream: &mut TlsStream,
        check_empty: bool,
    ) -> Result<Vec<Byte>> {
        let mut bytes = Vec::new();
        let mut buffer = vec![0; 8192];

        // Establecer un deadline absoluto
        let deadline = Instant::now() + Duration::from_secs(5);
        // Primero leer el header completo
        while bytes.len() < n_bytes {
            if Instant::now() > deadline {
                return Err(Error::ServerError("Timeout al leer header".into()));
            }

            match tls_stream.read(&mut buffer) {
                Ok(0) => {
                    if check_empty && bytes.is_empty() {
                        return Err(Error::ServerError(
                            "Conexión cerrada por el servidor".into(),
                        ));
                    }
                    break;
                }
                Ok(n) => {
                    bytes.extend_from_slice(&buffer[..n]);
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    std::thread::sleep(Duration::from_millis(50));
                    continue;
                }
                Err(e) => return Err(Error::ServerError(format!("Error de lectura: {}", e))),
            }
        }

        Ok(bytes)
    }

    fn read_complete_response(&mut self, tls_stream: &mut TlsStream) -> Result<ProtocolResult> {
        let mut response = Vec::new();
        let mut buffer = vec![0; 8192];

        // Establecer un deadline absoluto
        let deadline = Instant::now() + Duration::from_secs(5);
        // Primero leer el header completo
        while response.len() < HEADER_SIZE {
            if Instant::now() > deadline {
                return Err(Error::ServerError("Timeout al leer header".into()));
            }
            match tls_stream.read(&mut buffer) {
                Ok(0) => {
                    if response.is_empty() {
                        return Err(Error::ServerError(
                            "Conexión cerrada por el servidor".into(),
                        ));
                    }
                    break;
                }
                Ok(n) => {
                    response.extend_from_slice(&buffer[..n]);
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    std::thread::sleep(Duration::from_millis(50));
                    continue;
                }
                Err(e) => return Err(Error::ServerError(format!("Error de lectura: {}", e))),
            }
        }

        // Leer el cuerpo si hay header
        if response.len() >= HEADER_SIZE {
            let body_length = self.get_body_length(&response)?;
            let total_expected_length = HEADER_SIZE + body_length;

            while response.len() < total_expected_length {
                if std::time::Instant::now() > deadline {
                    return Err(Error::ServerError("Timeout al leer cuerpo".into()));
                }

                match tls_stream.read(&mut buffer) {
                    Ok(0) => break,
                    Ok(n) => {
                        response.extend_from_slice(&buffer[..n]);
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        std::thread::sleep(Duration::from_millis(50));
                        continue;
                    }
                    Err(e) => return Err(Error::ServerError(format!("Error de lectura: {}", e))),
                }
            }

            return self.handle_response(&response[..total_expected_length]);
        }

        Err(Error::ServerError("Respuesta incompleta".into()))
    }

    fn get_body_length(&self, response: &[Byte]) -> Result<usize> {
        if response.len() < HEADER_SIZE {
            return Err(Error::ServerError("Respuesta incompleta".into()));
        }

        let length_bytes = [response[5], response[6], response[7], response[8]];
        Ok(Uint::from_be_bytes(length_bytes) as usize)
    }

    fn handle_response(&mut self, request: &[Byte]) -> Result<ProtocolResult> {
        if request.len() < HEADER_SIZE {
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
        match Error::try_from(request[HEADER_SIZE..].to_vec()) {
            Ok(error) => Ok(ProtocolResult::QueryError(error)),
            Err(err) => Err(err),
        }
    }

    fn handle_ready(&self) -> Result<ProtocolResult> {
        Err(Error::Invalid(
            "La funcionalidad Ready no está soportada.".to_string(),
        ))
    }

    fn handle_authenticate(&self) -> Result<ProtocolResult> {
        Ok(ProtocolResult::Void)
    }

    fn handle_supported(&self) -> Result<ProtocolResult> {
        Err(Error::Invalid(
            "La funcionalidad Supported no está soportada.".to_string(),
        ))
    }

    fn handle_result(&self, lenght: Length, request: &[Byte]) -> Result<ProtocolResult> {
        match ResultKind::try_from(request[HEADER_SIZE..HEADER_SIZE + 4].to_vec())? {
            ResultKind::Void => Ok(ProtocolResult::Void),
            ResultKind::Rows => self.deserialize_rows(lenght, &request[13..]),
            ResultKind::SetKeyspace => self.set_keyspace(lenght, &request[13..]),
            ResultKind::Prepared => Err(Error::Invalid(
                "No se soporta la respuesta de tipo Prepared.".to_string(),
            )),
            ResultKind::SchemaChange => Err(Error::Invalid(
                "No se soporta la respuesta de tipo SchemaChange.".to_string(),
            )),
        }
    }

    fn handle_event(&self) -> Result<ProtocolResult> {
        Err(Error::Invalid(
            "La funcionalidad Event no está soportada.".to_string(),
        ))
    }

    fn handle_auth_challenge(&self) -> Result<ProtocolResult> {
        Err(Error::Invalid(
            "La funcionalidad AuthChallenge no está soportada.".to_string(),
        ))
    }

    fn handle_auth_success(&self) -> Result<ProtocolResult> {
        Ok(ProtocolResult::AuthSuccess)
    }

    fn deserialize_rows(&self, _lenght: Length, request: &[Byte]) -> Result<ProtocolResult> {
        let _flags = RowsFlag::try_from(request[..4].to_vec())?;
        let columns_count = Uint::from_be_bytes([request[4], request[5], request[6], request[7]]);
        let mut actual_position: usize = 8;
        let mut col_names: Vec<String> = Vec::new(); // usar col_names
        let mut col_types: Vec<ColType> = Vec::new(); // usar col_types que deberia ser ademas ColumnDataType
        for _ in 0..columns_count {
            let mut displacement: usize = 0;
            let col_name = parse_bytes_to_string(&request[actual_position..], &mut displacement)?;
            col_names.push(col_name);
            actual_position += displacement;
            col_types.push(ColType::try_from(&request[actual_position..])?);
            actual_position += 2;
        }
        let rows_count = self.read_bytes_to_int(request, actual_position)?;
        actual_position += 4;
        let mut rows: Vec<Vec<ColData>> = Vec::new(); // usar las filas ya parseadas
        for _ in 0..rows_count {
            let mut columns: Vec<ColData> = Vec::new();
            for i in 0..columns_count {
                let col_data = self.match_col_type(&col_types, i, request, &mut actual_position)?;
                columns.push(col_data);
            }
            rows.push(columns);
        }

        Ok(ProtocolResult::Rows(rows))
    }

    fn match_col_type(
        &self,
        col_types: &[ColType],
        i: Uint,
        request: &[Byte],
        actual_position: &mut usize,
    ) -> Result<ColData> {
        let col_data = match ColumnDataType::from(col_types[i as usize].clone()) {
            ColumnDataType::String => {
                let value = self.parse_string(request, actual_position)?;
                *actual_position += value.len();
                ColData::String(value)
            }
            ColumnDataType::Timestamp => {
                let value = self.parse_column_value::<Long>(request, actual_position)?;
                *actual_position += value.to_string().len();
                ColData::Timestamp(value)
            }
            ColumnDataType::Double => {
                let value = self.parse_column_value::<Double>(request, actual_position)?;
                *actual_position += value.to_string().len();
                ColData::Double(value)
            }
            ColumnDataType::Int => {
                let value = self.parse_column_value::<Int>(request, actual_position)?;
                *actual_position += value.to_string().len();
                ColData::Int(value)
            }
        };
        Ok(col_data)
    }

    fn set_keyspace(&self, lenght: Length, request: &[Byte]) -> Result<ProtocolResult> {
        match String::from_utf8(request[0..lenght.len as usize].to_vec()) {
            Ok(value) => Ok(ProtocolResult::SetKeyspace(value)),
            Err(_err) => Err(Error::TruncateError(
                "Error al transformar bytes a utf8".to_string(),
            )),
        }
    }

    fn read_bytes_to_int(&self, request: &[Byte], actual_position: usize) -> Result<Int> {
        if request.len() < actual_position + 4 {
            return Err(Error::Invalid(
                "No se recibio una query con el largo esperado".to_string(),
            ));
        }
        let number = Int::from_be_bytes([
            request[actual_position],
            request[actual_position + 1],
            request[actual_position + 2],
            request[actual_position + 3],
        ]);
        Ok(number)
    }

    fn parse_column_value<T>(&self, request: &[Byte], actual_position: &mut usize) -> Result<T>
    where
        T: std::str::FromStr,
        T::Err: std::fmt::Display,
    {
        let value_len = self.read_bytes_to_int(request, *actual_position)?;
        *actual_position += 4;
        let right_position = *actual_position + value_len as usize;
        let str_value = std::str::from_utf8(&request[*actual_position..right_position])
            .map_err(|_| Error::TruncateError("Error al transformar bytes a utf8".to_string()))?;

        str_value.parse::<T>().map_err(|e| {
            Error::TruncateError(format!("Error al parsear string '{}': {}", str_value, e))
        })
    }

    fn parse_string(&self, request: &[Byte], actual_position: &mut usize) -> Result<String> {
        let string_len = self.read_bytes_to_int(request, *actual_position)?;
        *actual_position += 4;
        let right_position = *actual_position + string_len as usize;

        String::from_utf8(request[*actual_position..right_position].to_vec())
            .map_err(|_| Error::TruncateError("Error al transformar bytes a utf8".to_string()))
    }

    /// Manda un mensaje aislado a una cierta dirección.
    fn send_message(socket: SocketAddr, bytes: &[Byte]) -> Result<()> {
        let mut tcp_stream = Self::connect_to(&[socket])?;
        let _ = tcp_stream.set_nonblocking(true);

        if let Err(err) = tcp_stream.write_all(bytes) {
            return Err(Error::ServerError(format!(
                "Error mandando mensaje aislado a {}:\n\n{}",
                socket, err
            )));
        }

        Ok(())
    }

    /// Manda a cada nodo un mensaje para que se [apague](crate::actions::opcode::SvAction::Exit).
    pub fn send_shutdown(&self) -> Result<()> {
        for addr in self.addr_loader.get_ips() {
            if let Err(err) = Self::send_message(
                AddrLoader::ip_to_socket(&addr, &PortType::Cli),
                &SvAction::Exit.as_bytes()[..],
            ) {
                println!("{}", err);
            }
            if let Err(err) = Self::send_message(
                AddrLoader::ip_to_socket(&addr, &PortType::Priv),
                &SvAction::Exit.as_bytes()[..],
            ) {
                println!("{}", err);
            }
        }

        Ok(())
    }

    /// Crea una request Startup para ser mandada
    pub fn prepare_startup_message() -> Result<Vec<Byte>> {
        let mut response = Vec::new();
        response.append(&mut Version::RequestV5.as_bytes());
        response.append(&mut Flag::Default.as_bytes());
        response.append(&mut Stream::new(0).as_bytes());
        response.append(&mut Opcode::Startup.as_bytes());
        response.append(&mut Length::new(0).as_bytes());
        let cql_version = vec![("CQL_VERSION".to_string(), "5.0.0".to_string())];
        let mut string_map_as_bytes = encode_string_map_to_bytes(cql_version);
        let length: Uint = string_map_as_bytes.len() as Uint;
        response.append(&mut string_map_as_bytes);
        response.splice(5..9, length.to_be_bytes());
        Ok(response)
    }

    /// Crea una request Startup para ser mandada
    pub fn prepare_auth_response_message(user: &str, password: &str) -> Result<Vec<Byte>> {
        let mut response = Vec::new();
        response.append(&mut Version::RequestV5.as_bytes());
        response.append(&mut Flag::Default.as_bytes());
        response.append(&mut Stream::new(0).as_bytes());
        response.append(&mut Opcode::AuthResponse.as_bytes());
        response.append(&mut Length::new(0).as_bytes());
        let mut user = encode_string_to_bytes(user);
        let mut password = encode_string_to_bytes(password);
        let length: Uint = (user.len() + password.len()) as Uint;
        response.append(&mut user);
        response.append(&mut password);
        response.splice(5..9, length.to_be_bytes());

        Ok(response)
    }
}

fn print_initial_message() {
    println!(
        "ECHO MODE:\n \
            ----------\n \
            Escribe tus queries. Cada línea se enviará al presionar Enter.\n \
            ----------\n \
            'q' o línea vacía para salir\n \
            'shutdown' para mandar un mensaje de apagado al servidor (y salir)\n \
            ----------\n"
    );
}

impl Default for Client {
    fn default() -> Self {
        Self::new(AddrLoader::default_loaded(), HashSet::<ShortInt>::new())
    }
}

/// Realiza el seteo del cliente para luego usarse en un tls_stream
pub fn get_client_connection() -> Result<rustls::ClientConnection> {
    let mut root_store = RootCertStore::empty();
    let certs = handle_pem_file_iter_client()?;
    for cert in certs {
        match root_store.add(cert) {
            Ok(_) => (),
            Err(_err) => return Err(Error::Invalid("Error al crear la conexion tls".to_string())),
        };
    }
    let config = ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_no_client_auth();
    let server_name = match "mydomain.com".try_into() {
        Ok(server_name) => server_name,
        Err(_) => {
            return Err(Error::Invalid(
                "No se pudo convertir el nombre del servidor".to_string(),
            ))
        }
    };
    let client_connection: rustls::ClientConnection =
        match rustls::ClientConnection::new(Arc::new(config), server_name) {
            Ok(client_connection) => client_connection,
            Err(_) => {
                return Err(Error::Invalid(
                    "No se pudo crear la conexion tls".to_string(),
                ))
            }
        };
    Ok(client_connection)
}

/// Maneja los resultados que se devuelven al cargar el certificado en el cliente
fn handle_pem_file_iter_client() -> Result<Vec<CertificateDer<'static>>> {
    let cert_file = get_root_path("cert.pem");
    let certs: Vec<CertificateDer<'_>> = match CertificateDer::pem_file_iter(cert_file) {
        Ok(certs_iter) => certs_iter
            .map(|cert_res| {
                cert_res.map_err(|_| Error::Invalid("No se pudo leer un certificado".to_string()))
            })
            .collect(),
        Err(err) => Err(Error::Invalid(format!(
            "No se pudo leer el archivo de certificados: {}",
            err
        ))),
    }?;
    Ok(certs)
}
