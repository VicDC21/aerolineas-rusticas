//! Módulo para el header Length.

use crate::{
    aliases::{
        results::Result,
        types::{Byte, Uint},
    },
    errors::error::Error,
    traits::Byteable,
};

/// Este header indica qué tan largo es el cuerpo del frame.
///
/// _(Actualmente está limitado a 256 MB)_
pub struct Length {
    /// Largo de un cuerpo del frame
    pub len: Uint,
}

impl Length {
    /// Crea un nuevo header de Stream.
    pub fn new(len: Uint) -> Self {
        Self { len }
    }
}

impl Byteable for Length {
    fn as_bytes(&self) -> Vec<Byte> {
        self.len.to_be_bytes().to_vec()
    }
}

impl TryFrom<Vec<Byte>> for Length {
    type Error = Error;
    fn try_from(integer_in_bytes: Vec<Byte>) -> Result<Self> {
        let bytes_array: [Byte; 4] = match integer_in_bytes.try_into() {
            Ok(bytes_array) => bytes_array,
            Err(_e) => {
                return Err(Error::ConfigError(
                    "No se pudo castear el vector de bytes en un array en Lenght".to_string(),
                ))
            }
        };
        let value = Uint::from_be_bytes(bytes_array);
        let bytes_lenght_limit = 0x10000000; // limite del frame de 256 MB
        match value {
            n if n <= bytes_lenght_limit => Ok(Length::new(n)),
            _ => Err(Error::ProtocolError(
                "El body del mensaje es muy largo (supera los 256MB)".to_string(),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_1_serializar() {
        for i in 0..1000 {
            let ind = i as Uint;
            let length_bytes = Length::new(ind).as_bytes();

            assert_eq!(length_bytes.len(), 4);
            assert_eq!(length_bytes, ind.to_be_bytes());
        }
    }

    #[test]
    fn test_2_deserializar() {
        let length_res = Length::try_from(vec![0x0, 0x0, 0x10, 0x1]);

        assert!(length_res.is_ok());
        if let Ok(length) = length_res {
            assert_eq!(length.len, 0x1001);
        }
    }

    #[test]
    fn test_3_bytes_de_longitud_incorrecta() {
        let muy_corto: Vec<Byte> = vec![0x0, 0x1, 0x2];
        let muy_largo: Vec<Byte> = vec![0x0, 0x1, 0x2, 0x3, 0x5];

        let corto_res = Length::try_from(muy_corto);
        let largo_res = Length::try_from(muy_largo);

        assert!(corto_res.is_err());
        if let Err(corto_err) = corto_res {
            assert!(matches!(corto_err, Error::ConfigError(_)));
        }

        assert!(largo_res.is_err());
        if let Err(largo_err) = largo_res {
            assert!(matches!(largo_err, Error::ConfigError(_)));
        }
    }

    #[test]
    fn test_4_longitud_de_mensaje_muy_grande() {
        let muy_grande: Vec<Byte> = vec![0x10, 0x0, 0x0, 0x1];

        let grande_res = Length::try_from(muy_grande);

        assert!(grande_res.is_err());
        if let Err(grande_err) = grande_res {
            assert!(matches!(grande_err, Error::ProtocolError(_)));
        }
    }
}
