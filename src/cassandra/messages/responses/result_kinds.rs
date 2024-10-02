//! Módulo para los tipos de _responses_ de tipo RESULT.

use crate::cassandra::traits::Byteable;

/// Tipos de resultados de una _query_ (mensajes QUERY, PREPARE, EXECUTE o BATCH).
pub enum ResultKind {
    /// El resultado no contiene información adicional en el cuerpo.
    Void,

    /// Resultado de SELECT, que devuelve las filas pedidas.
    Rows,

    /// El resultado de una _query_ `use`.
    SetKeyspace,

    /// El resultado de una _query_ de tipo PREPARE.
    Prepared,

    /// El resultado de una _query_ que altera un _schema_.
    SchemaChange,
}

impl Byteable for ResultKind {
    fn as_bytes(&self) -> Vec<u8> {
        match self {
            Self::Void => vec![0, 0, 0, 1],
            Self::Rows => vec![0, 0, 0, 2],
            Self::SetKeyspace => vec![0, 0, 0, 3],
            Self::Prepared => vec![0, 0, 0, 4],
            Self::SchemaChange => vec![0, 0, 0, 5],
        }
    }
}
