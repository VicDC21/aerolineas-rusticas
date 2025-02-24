/// Este módulo contiene submódulos relacionados con la cláusula `WHERE` en las declaraciones DML.
/// - `and`: Documentación para el módulo `and`.
pub mod and;
/// - `expression`: Maneja las expresiones dentro de la cláusula `WHERE`.
pub mod expression;
/// - `operator`: Define los operadores utilizados en las condiciones `WHERE`.
pub mod operator;
/// - `relation`: Gestiona las relaciones entre las condiciones `WHERE`.
pub mod relation;
/// - `r#where`: Implementa la lógica principal de la cláusula `WHERE`.
pub mod r#where_parser;
