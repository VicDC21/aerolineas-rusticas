//! Módulo para los tipos de columnas en una _response_ con filas.

use crate::cassandra::traits::Byteable;

/// Tipo nativo de columna, a ser incluido en la _spec_ del cuerpo de la _response_.
pub enum ColType {
    /// Un tipo personalizado. El nombre de dicho tipo es el valor.
    Custom(String),

    /// Secuencia de bytes ([ [u8] ]) en rango ASCII [0, 127].
    Ascii,

    /// Un número de 8 bytes en complemento a dos ([i64]).
    Bigint,

    /// Una secuencia de bytes "crudos" ([ [u8] ]).
    Blob,

    /// Un byte único ([u8] o [bool]) que denota un valor booleano:
    ///
    /// * Un valor de `0` indica `false`.
    /// * Cualquier otro valor indica `true`, pero igual se recomienda usar `1`.
    Boolean,

    /// Tipo Counter _(no lo decía bien en la doc)_.
    Counter,

    /// Número decimal de precisión arbitraria.
    ///
    /// Primero es precedido por un exponente ([i32]), y la base en formato [Varint](crate::cassandra::messages::responses::result::col_type::ColType::Varint).
    Decimal,

    /// Un número de 8 bytes en formato IEEE 754 (Binary64) de precisión doble ([f64]).
    Double,

    /// Un número de 4 bytes en formato IEEE 754 (Binary32) de precisión simple ([f32]).
    Float,

    /// Un número de 4 bytes en complemento a dos ([i32]).
    Int,

    /// Número de 8 bytes en complemento a dos ([i64]) indicando el tiempo en milisegundos desde la
    /// _unix epoch_ (1ro de Enero de 1970, 00:00:00).
    ///
    /// Valores negativos indican una diferencia negativa a esa época.
    Timestamp,

    /// Número de 16 bytes (asumimos ([u128])) representando cualquier versión de un UUID.
    Uuid,

    /// Un alias para el tipo "Text".
    ///
    /// Representa una secuencia de bytes ([ [u8] ]) en formato UTF-8.
    Varchar,

    /// Número de complemento a dos de longitud variable de un _integer_ con signo.
    ///
    /// El protocolo de Cassandra provee el siguiente ejemplo de uso:
    ///
    /// Value | Encoding
    /// ------|---------
    ///   0 |     0x00
    ///   1 |     0x01
    /// 127 |     0x7F
    /// 128 |   0x0080
    /// 129 |   0x0081
    ///  -1 |     0xFF
    /// -128 |     0x80
    /// -129 |   0xFF7F
    Varint,

    /// Número de 16 bytes (asumimos [u128]) representando un UUID (Versión 1) tal y como está especificado en RFC 4122.
    Timeuuid,

    /// Una secuencia de 4 o 16 bytes ([u32], [u128] o [IpAddr](std::net::IpAddr)) denotado una dirección IPv4 o IPv6 respectivamente..
    Inet,

    /// Una lista de tipos de columna, que bien pueden referirse a otros tipos de éstos.
    List(Box<Self>),

    /// Un "mapa" en donde tanto las claves como los valores pueden tener cada uno un tipo especificado
    /// en este [Enum](crate::cassandra::messages::responses::result::col_type::ColType).
    Map((Box<Self>, Box<Self>)),

    /// Un set con un tipo de columna especificado en este [Enum](crate::cassandra::messages::responses::result::col_type::ColType).
    Set(Box<Self>),

    /// El valor tiene la forma `<ks><udt_name><n><name_1><type_1>...<name_n><type_n>`, donde:
    ///
    /// * `<ks>` es un [String] representado el _keyspace_ al que pertenece este UDT.
    /// * `<udt_name>` es un [String] representando el nombre del UDT.
    /// * `<n>` es un número de 2 bytes ([u16]) que representa la cantidad de campos a continuación.
    /// * `<name_i>` es un [String] representando el nombre del i-ésimo campo del UDT.
    /// * `<value_i>` es una tipo de los especificados en este [Enum](crate::cassandra::messages::responses::result::col_type::ColType), tal que el i-ésimo campo del UDT tiene valor de ese tipo.
    ///
    /// TODO: _Quizás meter eso en un struct en el futuro._
    Udt((String, String, u16, Vec<(String, Box<Self>)>)),

    /// El valor tiene la forma `<n><type_1>...<type_n>` donde:
    ///
    /// * `<n>` es un número de 2 bytes ([u16]) representando el número de elementos.
    /// * `<type_i>` es el [tipo](crate::cassandra::messages::responses::result::col_type::ColType) del i-ésimo valor de la tupla.
    Tuple(Vec<Box<Self>>),
}

impl Byteable for ColType {
    fn as_bytes(&self) -> Vec<u8> {
        // OJO que esto devuelve los bytes de los posibles valores.
        match self {
            Self::Custom(nombre) => {
                let nombre_bytes = nombre.as_bytes();
                // litle endian para que los dos bytes menos significativos (los únicos que nos interesa
                // para un [u16]) estén al principio
                let bytes_len = nombre_bytes.len().to_le_bytes();
                let mut bytes_vec: Vec<u8> = vec![
                    0,
                    0, // ID
                    bytes_len[1],
                    bytes_len[0], // Longitud del nombre
                ];
                bytes_vec.extend_from_slice(nombre_bytes);
                bytes_vec
            }
            Self::Ascii => vec![0, 1],
            Self::Bigint => vec![0, 2],
            Self::Blob => vec![0, 3],
            Self::Boolean => vec![0, 4],
            Self::Counter => vec![0, 5],
            Self::Decimal => vec![0, 6],
            Self::Double => vec![0, 7],
            Self::Float => vec![0, 8],
            Self::Int => vec![0, 9],
            Self::Timestamp => vec![0, 11], // Sí, salteamos el 10 (`0xA`) a propósito
            Self::Uuid => vec![0, 12],
            Self::Varchar => vec![0, 13],
            Self::Varint => vec![0, 14],
            Self::Timeuuid => vec![0, 15],
            Self::Inet => vec![0, 16],
            Self::List(boxed) => {
                let mut bytes_vec: Vec<u8> = vec![
                    0, 32, // ID
                ];
                bytes_vec.extend((**boxed).as_bytes());
                bytes_vec
            }
            Self::Map((box_key, box_val)) => {
                let mut bytes_vec: Vec<u8> = vec![
                    0, 33, // ID
                ];
                bytes_vec.extend((**box_key).as_bytes());
                bytes_vec.extend((**box_val).as_bytes());
                bytes_vec
            }
            Self::Set(boxed) => {
                let mut bytes_vec: Vec<u8> = vec![
                    0, 34, // ID
                ];
                bytes_vec.extend((**boxed).as_bytes());
                bytes_vec
            }
            Self::Udt((ks, udt_name, n, vec_campos)) => {
                let ks_bytes = ks.as_bytes();
                let ks_bytes_len = ks_bytes.len().to_le_bytes();

                let mut bytes_vec: Vec<u8> = vec![
                    0,
                    48, // ID,
                    ks_bytes_len[1],
                    ks_bytes_len[0], // ks
                ];
                bytes_vec.extend_from_slice(ks_bytes);

                let udt_name_bytes = udt_name.as_bytes();
                let udt_name_bytes_len = udt_name_bytes.len().to_le_bytes();
                bytes_vec.extend_from_slice(&[udt_name_bytes_len[1], udt_name_bytes_len[0]]);
                bytes_vec.extend_from_slice(udt_name_bytes);

                let n_bytes = n.to_le_bytes();
                bytes_vec.extend_from_slice(&[n_bytes[1], n_bytes[0]]);

                for (nombre_campo, tipo_campo) in vec_campos {
                    let nombre_bytes = nombre_campo.as_bytes();
                    let nombre_bytes_len = nombre_bytes.len().to_le_bytes();
                    bytes_vec.extend_from_slice(&[nombre_bytes_len[1], nombre_bytes_len[0]]);
                    bytes_vec.extend((**tipo_campo).as_bytes());
                }

                bytes_vec
            }
            Self::Tuple(types_vec) => {
                let types_vec_len = types_vec.len().to_le_bytes();
                let mut bytes_vec: Vec<u8> = vec![
                    0,
                    49, // ID
                    types_vec_len[1],
                    types_vec_len[0], // longitud de la tupla
                ];
                for boxed_col in types_vec {
                    bytes_vec.extend((**boxed_col).as_bytes());
                }
                bytes_vec
            }
        }
    }
}
