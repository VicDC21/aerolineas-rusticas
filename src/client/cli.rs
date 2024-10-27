//! Módulo del cliente.

use std::{
    collections::HashSet,
    io::{stdin, BufRead, BufReader, Read, Write},
    net::{SocketAddr, TcpStream},
};

use crate::{
    parser::{
        main_parser::make_parse,
        statements::{
            ddl_statement::ddl_statement_parser::DdlStatement,
            dml_statement::dml_statement_parser::DmlStatement, statement::Statement,
        },
    },
    protocol::{
        aliases::{results::Result, types::Byte},
        errors::error::Error,
        headers::{
            flags::Flag, length::Length, msg_headers::Headers, opcode::Opcode, stream::Stream,
            version::Version,
        },
        notations::consistency::Consistency,
        traits::Byteable,
        utils::encode_string_to_bytes,
    },
    server::{actions::opcode::SvAction, utils::get_available_sockets},
    tokenizer::tokenizer::tokenize_query,
};

/// Flags específicas para queries CQL
#[derive(Clone, Copy, PartialEq)]
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

/// Estructura para el cuerpo de una query CQL
pub struct QueryBody {
    // [long string] - La query en sí
    query: String,
    // [consistency] - Nivel de consistencia
    consistency: Consistency,
    // [byte] - Flags
    flags: Vec<QueryFlags>,
    // Valores opcionales según las flags:
    // [n] - Número de valores si Values flag
    values: Option<Vec<Vec<u8>>>,
    // [i32] - Page size si PageSize flag
    page_size: Option<i32>,
    // [bytes] - Estado de paginación si WithPagingState flag
    paging_state: Option<Vec<u8>>,
    // [consistency] - Consistencia serial si WithSerialConsistency flag
    serial_consistency: Option<Consistency>,
    // [long] - Timestamp si WithDefaultTimestamp flag
    timestamp: Option<i64>,
}

impl QueryBody {
    /// Crea una nueva instancia de `QueryBody`. Por defecto, la consistencia es `ONE`.
    pub fn new(query: String) -> Self {
        Self {
            query,
            consistency: Consistency::One,
            flags: Vec::new(),
            values: None,
            page_size: None,
            paging_state: None,
            serial_consistency: None,
            timestamp: None,
        }
    }

    fn as_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        let query_bytes = encode_string_to_bytes(&self.query);
        bytes.extend((query_bytes.len() as i32).to_be_bytes());
        bytes.extend(query_bytes);
        bytes.extend((self.consistency as u16).to_be_bytes());

        let flags_byte = self.flags.iter().fold(0u8, |acc, flag| acc | *flag as u8);
        bytes.push(flags_byte);

        for flag in &self.flags {
            match flag {
                QueryFlags::Values => {
                    if let Some(values) = &self.values {
                        bytes.extend((values.len() as i16).to_be_bytes());
                        for value in values {
                            bytes.extend((value.len() as i32).to_be_bytes());
                            bytes.extend(value);
                        }
                    }
                }
                QueryFlags::PageSize => {
                    if let Some(size) = self.page_size {
                        bytes.extend(size.to_be_bytes());
                    }
                }
                QueryFlags::WithPagingState => {
                    if let Some(state) = &self.paging_state {
                        bytes.extend((state.len() as i32).to_be_bytes());
                        bytes.extend(state);
                    }
                }
                QueryFlags::WithSerialConsistency => {
                    if let Some(consistency) = &self.serial_consistency {
                        bytes.extend((*consistency as u16).to_be_bytes());
                    }
                }
                QueryFlags::WithDefaultTimestamp => {
                    if let Some(ts) = self.timestamp {
                        bytes.extend(ts.to_be_bytes());
                    }
                }
                _ => {}
            }
        }
        bytes
    }
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
        let stream = &mut stdin();
        let reader = BufReader::new(stream);
        let mut sendable_lines = Vec::<String>::new();

        println!(
            "ECHO MODE:\n \
                  ----------\n \
                  Escribe lo que necesites.\n \
                  Cuando salgas de este modo, se mandará todo de una al servidor.\n \
                  ----------\n \
                  'q' o línea vacía para salir\n \
                  'shutdown' para mandar un mensaje de apagado al servidor (y salir)\n \
                  ----------\n"
        );

        for line in reader.lines().map_while(|e| e.ok()) {
            if line.eq_ignore_ascii_case("q") || line.is_empty() {
                break;
            }
            if line.eq_ignore_ascii_case("shutdown") {
                if let Err(err) = tcp_stream.write_all(&SvAction::Shutdown.as_bytes()[..]) {
                    println!("Error mandando el mensaje de shutdown:\n\n{}", err);
                }
                return Ok(());
            }
            sendable_lines.push(line);
        }
        if sendable_lines.is_empty() {
            return Ok(());
        }

        let query = sendable_lines.join(" ");
        let mut stream_id: i16 = 0;
        while self.requests_stream.contains(&stream_id) {
            stream_id += 1;
        }
        self.requests_stream.insert(stream_id);
        let mut header = Vec::<Byte>::new();
        // flags que despues vemos como las agregamos, en principio para la entrega intermedia no afecta
        // Numero de stream, tiene que ser positivo en cliente
        if self.parse_request(&query, &mut header, stream_id) {
            if let Err(err) = tcp_stream.write(&header) {
                println!("Error al escribir en el TCPStream:\n\n{}", err);
            }

            // para asegurarse de que se vacía el stream antes de escuchar de nuevo.
            if let Err(err) = tcp_stream.flush() {
                println!("Error haciendo flush desde el cliente:\n\n{}", err);
            }
            let mut buf = Vec::<Byte>::new();
            match tcp_stream.read_to_end(&mut buf) {
                Err(err) => println!("Error recibiendo response de un nodo:\n\n{}", err),
                Ok(i) => {
                    println!("{} bytes - {:?}", i, buf);
                }
            }
        }

        Ok(())
    }

    fn parse_request(&mut self, line: &str, header: &mut Vec<Byte>, stream_id: i16) -> bool {
        match make_parse(&mut tokenize_query(line)) {
            Ok(statement) => {
                let query_body = QueryBody::new(line.to_string());
                let body_bytes = query_body.as_bytes();

                match statement {
                    Statement::DdlStatement(ddl_statement) => {
                        Self::fill_headers(header, stream_id, &body_bytes);
                        self.handle_ddl_statement(ddl_statement);
                    }
                    Statement::DmlStatement(dml_statement) => {
                        Self::fill_headers(header, stream_id, &body_bytes);
                        self.handle_dml_statement(dml_statement);
                    }
                    Statement::UdtStatement(_udt_statement) => {
                        Self::fill_headers(header, stream_id, &body_bytes);
                        todo!();
                    }
                };
                header.extend(body_bytes);
                true
            }
            Err(err) => {
                println!("{}", err);
                false
            }
        }
    }

    /// Llena el resto de headers.
    fn fill_headers(header: &mut Vec<Byte>, stream_id: i16, body: &[u8]) {
        let version = Version::RequestV5;
        let flags = Flag::Default;
        let stream = Stream::new(stream_id);
        let opcode = Opcode::Query;
        let length = Length::new(body.len() as u32);

        header.extend(Headers::new(version, vec![flags], stream, opcode, length).as_bytes());
    }

    /// Maneja una declaración DDL.
    fn handle_ddl_statement(&self, ddl_statement: DdlStatement) {
        match ddl_statement {
            DdlStatement::UseStatement(_keyspace_name) => {
                println!("Handle USE!")
            }
            DdlStatement::CreateKeyspaceStatement(_create_keyspace) => {}
            DdlStatement::AlterKeyspaceStatement(_alter_keyspace) => {}
            DdlStatement::DropKeyspaceStatement(_drop_keyspace) => {}
            DdlStatement::CreateTableStatement(_create_table) => {}
            DdlStatement::AlterTableStatement(_alter_table) => {}
            DdlStatement::DropTableStatement(_drop_table) => {}
            DdlStatement::TruncateStatement(_truncate) => {}
        }
    }

    /// Maneja una declaración DML.
    fn handle_dml_statement(&self, dml_statement: DmlStatement) {
        match dml_statement {
            DmlStatement::SelectStatement(_select) => {}
            DmlStatement::InsertStatement(_insert) => {}
            DmlStatement::UpdateStatement(_update) => {}
            DmlStatement::DeleteStatement(_delete) => {}
            DmlStatement::BatchStatement(_batch) => {}
        }
    }
}

impl Default for Client {
    fn default() -> Self {
        Self::new(get_available_sockets(), HashSet::<i16>::new())
    }
}
