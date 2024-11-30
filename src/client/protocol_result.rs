use crate::protocol::errors::error::Error;

use super::col_data::ColData;

/// Resultado de una operación en el protocolo de Cassandra.
#[derive(Debug)]
pub enum ProtocolResult {
    /// El resultado no contiene información adicional en el cuerpo.
    Void,

    /// Resultado de SELECT, que devuelve las filas pedidas.
    Rows(Vec<Vec<ColData>>),

    /// El resultado de una _query_ `use`.
    SetKeyspace(String),

    /// El resultado de una _query_ de tipo PREPARE.
    Prepared,

    /// El resultado de una _query_ que altera un _schema_.
    SchemaChange,

    /// Indica que el cliente fue aceptado por el servidor.
    AuthSuccess,

    /// El resultado de una _query_ que indica un error en la consulta.
    QueryError(Error),
}
