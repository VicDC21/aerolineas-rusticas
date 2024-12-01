use crate::parser::statements::ddl_statement::ddl_statement_parser::DdlStatement;
use crate::parser::statements::dml_statement::dml_statement_parser::DmlStatement;
//use crate::parser::statements::role_or_permission_statement::role_or_permission_statement_parser::RoleOrPermissionStatement;
use crate::parser::statements::udt_statement::udt_statement_parser::UdtStatement;

use super::login_user_statement::LoginUserStatement;

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
    //RoleOrPermissionStatement(RoleOrPermissionStatement),
    /// Representa una declaración UDT (User Defined Type).
    /// Las declaraciones UDT se utilizan para definir un nuevo tipo de datos personalizado.
    /// Este tipo de datos personalizado se puede utilizar en la definición de columnas de una tabla.
    UdtStatement(UdtStatement),

    ///Representa el logueo de usuarios (es una implementacion por fuera de cassandra creada por nosotros)
    LoginUser(LoginUserStatement),

    ///TODO
    Startup,
}
