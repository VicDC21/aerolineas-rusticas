//! Módulo para un tipo de Dato Definido por Usuario (UDT).

use std::convert::TryFrom;

use crate::protocol::aliases::types::{Byte, Short};
use crate::protocol::errors::error::Error;
use crate::protocol::messages::responses::result::col_type::ColType;
use crate::protocol::traits::Byteable;
use crate::protocol::utils::{encode_string_to_bytes, parse_bytes_to_string};

/// Alias para un vector de campos de UDT.
pub type UdtTypeFields = Vec<(String, Box<ColType>)>;

/// Un "User Defined Type" (UDT) es un tipo de dato personalizado que crea el usuario.
#[derive(Clone)]
pub struct UdtType {
    /// El nombre del keyspace donde pertenece el UDT.
    ks: String,

    /// El nombre del UDT mismo.
    udt_name: String,

    /// Los campos del UDT. El orden es importante, así que se guardan en secuencia.
    fields: UdtTypeFields,
}

impl UdtType {
    /// Crea una nueva instance de tipo de UDT.
    pub fn new(ks: String, udt_name: String, fields: UdtTypeFields) -> Self {
        Self {
            ks,
            udt_name,
            fields,
        }
    }
}

impl Byteable for UdtType {
    fn as_bytes(&self) -> Vec<Byte> {
        let mut bytes_vec: Vec<Byte> = vec![
            0x0, 0x30, // ID
        ];
        bytes_vec.extend(encode_string_to_bytes(&self.ks));
        bytes_vec.extend(encode_string_to_bytes(&self.udt_name));

        let n_bytes = self.fields.len().to_le_bytes();
        bytes_vec.extend_from_slice(&[n_bytes[1], n_bytes[0]]);

        for (nombre_campo, tipo_campo) in &self.fields {
            bytes_vec.extend(encode_string_to_bytes(nombre_campo));
            bytes_vec.extend((**tipo_campo).as_bytes());
        }

        bytes_vec
    }
}

impl TryFrom<&[Byte]> for UdtType {
    type Error = Error;
    fn try_from(bytes_vec: &[Byte]) -> Result<Self, Self::Error> {
        let mut i: usize = 0;
        let bytes_len = bytes_vec.len();
        if bytes_len < 2 {
            return Err(Error::ProtocolError(format!(
                "Se esperaban por lo menos 2 bytes para denotar la longitud del mensaje, no {}.",
                bytes_len
            )));
        }
        let udt_id = &[bytes_vec[i], bytes_vec[i + 1]];
        if udt_id != &[0x0, 0x30] {
            return Err(Error::ProtocolError(format!(
                "ID {:?} incorrecto para un UdtType.",
                udt_id
            )));
        }
        i += 2;

        let ks = parse_bytes_to_string(&bytes_vec[i..], &mut i)?;
        let udt_name = parse_bytes_to_string(&bytes_vec[i..], &mut i)?;

        let n = Short::from_be_bytes([bytes_vec[i], bytes_vec[i + 1]]);
        i += 2;

        let mut fields: UdtTypeFields = Vec::new();
        for _ in 0..n {
            let name = parse_bytes_to_string(&bytes_vec[i..], &mut i)?;

            let col_type = ColType::try_from(&bytes_vec[i..])?;
            i += col_type.as_bytes().len();
            fields.push((name, Box::new(col_type)));
        }
        Ok(Self::new(ks, udt_name, fields))
    }
}

#[cfg(test)]
mod tests {
    use crate::protocol::errors::error::Error;
    use crate::protocol::messages::responses::result::col_type::ColType;
    use crate::protocol::traits::Byteable;
    use crate::protocol::user::udt_type::UdtType;

    #[test]
    fn test_1_serializar() {
        let udt = UdtType::new(
            "keyspace_feo".to_string(),
            "UDT chulo".to_string(),
            vec![
                ("booleano".to_string(), Box::new(ColType::Boolean)),
                ("entero".to_string(), Box::new(ColType::Int)),
            ],
        );

        assert_eq!(
            udt.as_bytes(),
            [
                0x0, 0x30, 0x0, 0xC, 0x6B, 0x65, 0x79, 0x73, 0x70, 0x61, 0x63, 0x65, 0x5F, 0x66,
                0x65, 0x6F, 0x0, 0x9, 0x55, 0x44, 0x54, 0x20, 0x63, 0x68, 0x75, 0x6C, 0x6F, 0x0,
                0x2, 0x0, 0x8, 0x62, 0x6F, 0x6F, 0x6C, 0x65, 0x61, 0x6E, 0x6F, 0x0, 0x4, 0x0, 0x6,
                0x65, 0x6E, 0x74, 0x65, 0x72, 0x6F, 0x0, 0x9
            ]
        );
    }

    #[test]
    fn test_2_deserializar() {
        let udt_res = UdtType::try_from(
            &[
                0x0, 0x30, 0x0, 0xC, 0x6B, 0x65, 0x79, 0x73, 0x70, 0x61, 0x63, 0x65, 0x5F, 0x66,
                0x65, 0x6F, 0x0, 0x9, 0x55, 0x44, 0x54, 0x20, 0x63, 0x68, 0x75, 0x6C, 0x6F, 0x0,
                0x2, 0x0, 0x8, 0x62, 0x6F, 0x6F, 0x6C, 0x65, 0x61, 0x6E, 0x6F, 0x0, 0x4, 0x0, 0x6,
                0x65, 0x6E, 0x74, 0x65, 0x72, 0x6F, 0x0, 0x9,
            ][..],
        );

        assert!(udt_res.is_ok());
        if let Ok(udt) = udt_res {
            assert_eq!(udt.ks.as_str(), "keyspace_feo");
            assert_eq!(udt.udt_name.as_str(), "UDT chulo");

            let (bool_name, bool_type) = &udt.fields[0];
            assert_eq!(bool_name.as_str(), "booleano");
            assert!(matches!(**bool_type, ColType::Boolean));

            let (int_name, int_type) = &udt.fields[1];
            assert_eq!(int_name.as_str(), "entero");
            assert!(matches!(**int_type, ColType::Int));
        }
    }

    #[test]
    fn test_3_id_incorrecto() {
        let bad_id = UdtType::try_from(&[0x0, 0x40, 0x1, 0x2, 0x3, 0x4][..]);

        assert!(bad_id.is_err());
        if let Err(err) = bad_id {
            assert!(matches!(err, Error::ProtocolError(_)));
        }
    }

    #[test]
    fn test_4_muy_corto() {
        let corto = UdtType::try_from(&[0x1][..]);

        assert!(corto.is_err());
        if let Err(err) = corto {
            assert!(matches!(err, Error::ProtocolError(_)));
        }
    }
}
