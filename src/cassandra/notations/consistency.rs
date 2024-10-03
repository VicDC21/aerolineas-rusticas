//! Módulo para enumerar niveles de consistencia.

use crate::cassandra::traits::Byteable;

/// Nivela los modos de consistencia para los _read request_.
///
/// TODO: dejar mejores descripciones.
pub enum Consistency {
    /// Buscar cualquier nodo
    Any,

    /// Buscar un único nodo
    One,

    /// Buscar dos nodos
    Two,

    /// Buscar tres nodos
    Three,

    /// Decidir por mayoría
    Quorum,

    /// Buscar TODOS los nodos disponibles
    All,

    /// Decidir por mayoría local
    LocalQuorum,

    /// Decidir por mayoría _#NoTengoNiIdeaDeLaDiferencia_
    EachQuorum,

    /// SERIAL Variant
    Serial,

    /// LOCAL_SERIAL Variant
    LocalSerial,

    /// LOCAL_ONE Variant
    LocalOne,
}

impl Byteable for Consistency {
    fn as_bytes(&self) -> Vec<u8> {
        match self {
            Self::Any => vec![0, 0],
            Self::One => vec![0, 1],
            Self::Two => vec![0, 2],
            Self::Three => vec![0, 3],
            Self::Quorum => vec![0, 4],
            Self::All => vec![0, 5],
            Self::LocalQuorum => vec![0, 6],
            Self::EachQuorum => vec![0, 7],
            Self::Serial => vec![0, 8],
            Self::LocalSerial => vec![0, 9],
            Self::LocalOne => vec![0, 10],
        }
    }
}
