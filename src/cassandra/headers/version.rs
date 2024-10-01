//! Módulo para la versión del protocolo.

use crate::cassandra::traits::Byteable;

/// La 'versión' indica tanto la versión del protocolo a usar,
/// así como si se trata con un _request_ o un _response_.
pub enum Version {
    /// _Request_ del protocolo nativo de Cassandra (Versión 3).
    RequestV3,

    /// _Response_ del protocolo nativo de Cassandra (Versión 3).
    ResponseV3,

    /// _Request_ del protocolo nativo de Cassandra (Versión 4).
    RequestV4,

    /// _Response_ del protocolo nativo de Cassandra (Versión 4).
    ResponseV4,

    /// _Request_ del protocolo nativo de Cassandra (Versión 5).
    RequestV5,

    /// _Response_ del protocolo nativo de Cassandra (Versión 5).
    ResponseV5,
}

impl Byteable for Version {
    fn as_bytes(&self) -> &[u8] {
        match self {
            Self::RequestV3 => &[3],
            Self::ResponseV3 => &[131],
            Self::RequestV4 => &[4],
            Self::ResponseV4 => &[132],
            Self::RequestV5 => &[5],
            Self::ResponseV5 => &[133],
        }
    }
}
