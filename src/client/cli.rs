//! Módulo del cliente.

use std::{
    io::{stdin, BufRead, BufReader, Write},
    net::{SocketAddr, TcpStream},
};

use eframe::glow::Version;

use crate::server::actions::opcode::SvAction;
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

/// Estructura principal de un cliente.
pub struct Client {
    /// La dirección del _socket_ al que conectarse al mandar cosas.
    addr: SocketAddr,
    requests_stream: Vec<i16>,
}

impl Client {
    /// Crea una nueva instancia de cliente.
    pub fn new(addr: SocketAddr) -> Self {
        let requests_stream: Vec<i16> = Vec::new();
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
            let stream = match self.requests_stream.iter().min() {
                Some(stream_id) => *stream_id as u8,
                None => 0,
            }; // despues ver como hacer que no supere los 32768
            self.requests_stream.push(stream as i16);
            let mut header: Vec<u8> = vec![0x85, 0x00, stream];
            // version que no se cuando pediriamos (startup?)
            // flags que despues vemos como las agregamos, en principio para la entrega intermedia no afecta
            // Numero de stream, tiene que ser positivo en cliente

            match make_parse(&mut tokenize_query(&line)) {
                Ok(statement) => match statement {
                    Statement::DdlStatement(ddl_statement) => {
                        self.handle_ddl_statement(ddl_statement);
                    }
                    Statement::DmlStatement(dml_statement) => {
                        self.handle_dml_statement(dml_statement);
                    }
                    Statement::UdtStatement(_udt_statement) => {
                        todo!();
                    }
                },
                Err(err) => {
                    println!("{}", err);
                }
            };

            let _ = tcp_stream.write_all(&header);
        }
        Ok(())
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
