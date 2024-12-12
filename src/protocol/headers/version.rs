//! Módulo para la versión del protocolo.

use {
    crate::protocol::{aliases::types::Byte, errors::error::Error, traits::Byteable},
    std::convert::TryFrom,
};

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
    fn as_bytes(&self) -> Vec<Byte> {
        match self {
            Self::RequestV3 => vec![0x3],
            Self::ResponseV3 => vec![0x83],
            Self::RequestV4 => vec![0x4],
            Self::ResponseV4 => vec![0x84],
            Self::RequestV5 => vec![0x5],
            Self::ResponseV5 => vec![0x85],
        }
    }
}

impl TryFrom<Byte> for Version {
    type Error = Error;
    fn try_from(byte: Byte) -> Result<Self, Self::Error> {
        match byte {
            0x03 => Ok(Version::RequestV3),
            0x83 => Ok(Version::ResponseV3),
            0x04 => Ok(Version::RequestV4),
            0x84 => Ok(Version::ResponseV4),
            0x05 => Ok(Version::RequestV5),
            0x85 => Ok(Version::ResponseV5),
            _ => Err(Error::ConfigError(
                "La version del protocolo especificada no existe".to_string(),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_1_serializar_v5() {
        let req_v5 = Version::RequestV5;
        let res_v5 = Version::ResponseV5;

        let req_bytes = req_v5.as_bytes();
        let res_bytes = res_v5.as_bytes();

        // la versión debería ser sólo un byte
        assert_eq!(req_bytes.len(), 1);
        assert_eq!(res_bytes.len(), 1);

        assert_eq!(req_bytes[0], 0x5);
        assert_eq!(res_bytes[0], 0x85);
    }

    #[test]
    fn test_2_deserializar_v5() {
        let req_result = Version::try_from(0x5);
        let res_result = Version::try_from(0x85);

        assert!(req_result.is_ok());
        if let Ok(req) = req_result {
            matches!(req, Version::RequestV5);
        }

        assert!(res_result.is_ok());
        if let Ok(res) = res_result {
            matches!(res, Version::RequestV5);
        }
    }

    #[test]
    fn test_3_id_incorrecto_tira_error() {
        let inexistent = Version::try_from(0xFF);

        assert!(inexistent.is_err());
        if let Err(err) = inexistent {
            assert!(matches!(err, Error::ConfigError(_)));
        }
    }
}
