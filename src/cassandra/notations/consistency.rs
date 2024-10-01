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
    fn as_bytes(&self) -> &[u8] {
        match self {
            Self::Any => &[0, 0],
            Self::One => &[0, 1],
            Self::Two => &[0, 2],
            Self::Three => &[0, 3],
            Self::Quorum => &[0, 4],
            Self::All => &[0, 5],
            Self::LocalQuorum => &[0, 6],
            Self::EachQuorum => &[0, 7],
            Self::Serial => &[0, 8],
            Self::LocalSerial => &[0, 9],
            Self::LocalOne => &[0, 10],
        }
    }
}
