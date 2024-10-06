//! Módulo para un tipo de Dato Definido por Usuario (UDT).

use std::convert::TryFrom;

use crate::cassandra::aliases::types::{Byte, Short};
use crate::cassandra::errors::error::Error;
use crate::cassandra::messages::responses::result::col_type::ColType;
use crate::cassandra::traits::Byteable;
use crate::cassandra::utils::parse_bytes_to_string;

/// Alias para un vector de campos de UDT.
pub type UdtTypeFields = Vec<(String, Box<ColType>)>;

/// Un "User Defined Type" (UDT) es un tipo de dato personalizado que crea el usuario.
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
        let ks_bytes = self.ks.as_bytes();
        let ks_bytes_len = ks_bytes.len().to_le_bytes();

        let mut bytes_vec: Vec<Byte> = vec![
            0x0,
            0x30, // ID,
            ks_bytes_len[1],
            ks_bytes_len[0], // ks
        ];
        bytes_vec.extend_from_slice(ks_bytes);

        let udt_name_bytes = self.udt_name.as_bytes();
        let udt_name_bytes_len = udt_name_bytes.len().to_le_bytes();
        bytes_vec.extend_from_slice(&[udt_name_bytes_len[1], udt_name_bytes_len[0]]);
        bytes_vec.extend_from_slice(udt_name_bytes);

        let n_bytes = self.fields.len().to_le_bytes();
        bytes_vec.extend_from_slice(&[n_bytes[1], n_bytes[0]]);

        for (nombre_campo, tipo_campo) in &self.fields {
            let nombre_bytes = nombre_campo.as_bytes();
            let nombre_bytes_len = nombre_bytes.len().to_le_bytes();
            bytes_vec.extend_from_slice(&[nombre_bytes_len[1], nombre_bytes_len[0]]);
            bytes_vec.extend_from_slice(nombre_bytes);
            bytes_vec.extend((**tipo_campo).as_bytes());
        }

        bytes_vec
    }
}

impl TryFrom<&[Byte]> for UdtType {
    type Error = Error;
    fn try_from(bytes_vec: &[Byte]) -> Result<Self, Self::Error> {
        let mut i: usize = 0;
        let ks = parse_bytes_to_string(&bytes_vec[i..], &mut i)?;
        let udt_name = parse_bytes_to_string(&bytes_vec[i..], &mut i)?;

        let n = Short::from_be_bytes([bytes_vec[i], bytes_vec[i + 1]]);
        i += 2;

        let mut fields: Vec<(String, Box<ColType>)> = Vec::new();
        for _i in 0..n {
            let name = parse_bytes_to_string(&bytes_vec[i..], &mut i)?;

            let col_type = ColType::try_from(&mut bytes_vec[i..].to_vec())?;
            i += col_type.as_bytes().len();
            fields.push((name, Box::new(col_type)));
        }
        Ok(Self::new(ks, udt_name, fields))
    }
}
