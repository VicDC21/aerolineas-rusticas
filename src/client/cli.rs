//! Módulo del cliente.

use std::{
    collections::HashSet,
    io::{stdin, BufRead, BufReader, Write},
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
        traits::Byteable,
    },
    tokenizer::tokenizer::tokenize_query,
};
use crate::{
    protocol::headers::{length::Length, opcode::Opcode},
    server::actions::opcode::SvAction,
};

/// Estructura principal de un cliente.
pub struct Client {
    /// La dirección del _socket_ al que conectarse al mandar cosas.
    addr: SocketAddr,
    requests_stream: HashSet<i16>,
}

impl Client {
    /// Crea una nueva instancia de cliente.
    pub fn new(addr: SocketAddr) -> Self {
        let requests_stream = HashSet::new();
        Self {
            addr,
            requests_stream,
        }
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
    pub fn echo(&mut self) -> Result<()> {
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

            let mut stream_id: i16 = 0;
            while self.requests_stream.contains(&stream_id) {
                stream_id += 1;
            }
            self.requests_stream.insert(stream_id);
            let mut header: Vec<u8> = vec![0x05, 0x00, stream_id as u8];
            // flags que despues vemos como las agregamos, en principio para la entrega intermedia no afecta
            // Numero de stream, tiene que ser positivo en cliente
            // self.parse_request(&line, &mut header);
            if self.parse_request(&line, &mut header) {
                let _ = tcp_stream.write_all(&header);
                let _ = tcp_stream.write_all(line.as_bytes());
            }
        }
        Ok(())
    }

    fn parse_request(&mut self, line: &str, header: &mut Vec<u8>) -> bool {
        match make_parse(&mut tokenize_query(line)) {
            Ok(statement) => {
                match statement {
                    Statement::DdlStatement(ddl_statement) => {
                        header.push(Opcode::Query.as_bytes()[0]);
                        let lenght = Length::new(line.len() as u32);
                        header.append(&mut lenght.as_bytes());
                        self.handle_ddl_statement(ddl_statement);
                    }
                    Statement::DmlStatement(dml_statement) => {
                        header.push(Opcode::Query.as_bytes()[0]);
                        let lenght = Length::new(line.len() as u32);
                        header.append(&mut lenght.as_bytes());
                        self.handle_dml_statement(dml_statement);
                    }
                    Statement::UdtStatement(_udt_statement) => {
                        header.push(Opcode::Query.as_bytes()[0]);
                        let lenght = Length::new(line.len() as u32);
                        header.append(&mut lenght.as_bytes());
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

    /// Maneja una declaración DDL.
    fn handle_ddl_statement(&self, ddl_statement: DdlStatement) {
        match ddl_statement {
            DdlStatement::UseStatement(_keyspace_name) => {}
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
