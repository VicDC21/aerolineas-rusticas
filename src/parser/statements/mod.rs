/// Módulo para sentencias de Lenguaje de Definición de Datos (DDL).
/// Las sentencias DDL se utilizan para definir y modificar la estructura de la base de datos,
pub mod ddl_statement;

/// Módulo para sentencias de Lenguaje de Manipulación de Datos (DML).
/// Las sentencias DML se utilizan para manipular datos en la base de datos,
pub mod dml_statement;

/// Módulo global para sentencias y declaraciones.
/// Las sentencias y declaraciones se utilizan para definir y manipular la estructura y los datos de la base de datos.
pub mod statement;

/// Módulo para sentencias de tipos de datos definidos por el usuario (UDT).
/// Las sentencias UDT se utilizan para definir un nuevo tipo de datos personalizado.
pub mod udt_statement;

/// Módulo para sentencias de login de usuarios.
pub mod login_user_statement;

/// Módulo para sentencias de STARTUP.
pub mod startup_statement;
