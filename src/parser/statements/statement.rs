use crate::parser::statements::{
    ddl_statement::ddl_statement_parser::DdlStatement,
    dml_statement::dml_statement_parser::DmlStatement, login_user_statement::LoginUserStatement,
};

/// Representa una declaración CQL (Cassandra Query Language).
/// Las declaraciones CQL se utilizan para definir y manipular la estructura y los datos de la base de datos.

#[derive(Debug)]
pub enum Statement {
    /// Representa una declaración DDL (Data Definition Language).
    /// Las declaraciones DDL se utilizan para definir y modificar la estructura de la base de datos,
    /// como crear, alterar o eliminar tablas y otros objetos de la base de datos.
    DdlStatement(DdlStatement),

    /// Representa una declaración DML (Data Manipulation Language).
    /// Las declaraciones DML se utilizan para manipular datos en la base de datos,
    /// como insertar, actualizar o eliminar registros de una tabla.
    DmlStatement(DmlStatement),

    /// Representa una declaración de dar inicio a la conexión.
    Startup,

    /// Representa el logueo de usuarios (es una implementacion por fuera de cassandra creada por nosotros)
    LoginUser(LoginUserStatement),
}
