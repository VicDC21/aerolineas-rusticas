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
        traits::Byteable,
        utils::encode_string_to_bytes,
    },
    server::{actions::opcode::SvAction, utils::get_available_sockets},
    tokenizer::tokenizer::tokenize_query,
};

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
                match statement {
                    Statement::DdlStatement(ddl_statement) => {
                        Self::fill_headers(line, header, stream_id);
                        self.handle_ddl_statement(ddl_statement);
                    }
                    Statement::DmlStatement(dml_statement) => {
                        Self::fill_headers(line, header, stream_id);
                        self.handle_dml_statement(dml_statement);
                    }
                    Statement::UdtStatement(_udt_statement) => {
                        Self::fill_headers(line, header, stream_id);
                        todo!();
                    }
                };
                true
            }
            Err(err) => {
                println!("{}", err);
                false
            }
        }
    }

    /// Llena el resto de headers.
    fn fill_headers(line: &str, header: &mut Vec<Byte>, stream_id: i16) {
        let version = Version::RequestV5;
        let flags = Flag::Default;
        let stream = Stream::new(stream_id);
        let opcode = Opcode::Query;
        // Esto está mal, el body de una query tiene un montón de metadatos,
        // y el len se le hace al vector de bytes serializados.
        let line_bytes = encode_string_to_bytes(line);
        let length = Length::new(line_bytes.len() as u32);

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
