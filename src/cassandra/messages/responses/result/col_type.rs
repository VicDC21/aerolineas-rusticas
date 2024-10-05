//! Módulo para los tipos de columnas en una _response_ con filas.

use std::str;

use crate::cassandra::aliases::types::{Byte, Short};
use crate::cassandra::errors::error::Error;
use crate::cassandra::traits::Byteable;

/// Tipo nativo de columna, a ser incluido en la _spec_ del cuerpo de la _response_.
pub enum ColType {
    /// Un tipo personalizado. El nombre de dicho tipo es el valor.
    Custom(String),

    /// Secuencia de bytes ([ [Byte] ]) en rango ASCII [0, 127].
    Ascii,

    /// Un número de 8 bytes en complemento a dos ([i64]).
    Bigint,

    /// Una secuencia de bytes "crudos" ([ [Byte] ]).
    Blob,

    /// Un byte único ([Byte] o [bool]) que denota un valor booleano:
    ///
    /// * Un valor de `0` indica `false`.
    /// * Cualquier otro valor indica `true`, pero igual se recomienda usar `1`.
    Boolean,

    /// Tipo Counter _(no lo decía bien en la doc)_.
    Counter,

    /// Número decimal de precisión arbitraria.
    ///
    /// Primero es precedido por un exponente ([Int](crate::cassandra::aliases::types::Int)), y la base en formato [Varint](crate::cassandra::messages::responses::result::col_type::ColType::Varint).
    Decimal,

    /// Un número de 8 bytes en formato IEEE 754 (Binary64) de precisión doble ([Double](crate::cassandra::aliases::types::Double)).
    Double,

    /// Un número de 4 bytes en formato IEEE 754 (Binary32) de precisión simple ([Float](crate::cassandra::aliases::types::Float)).
    Float,

    /// Un número de 4 bytes en complemento a dos ([Int](crate::cassandra::aliases::types::Int)).
    Int,

    /// Número de 8 bytes en complemento a dos ([Long](crate::cassandra::aliases::types::Long)) indicando el tiempo en milisegundos desde la
    /// _unix epoch_ (1ro de Enero de 1970, 00:00:00).
    ///
    /// Valores negativos indican una diferencia negativa a esa época.
    Timestamp,

    /// Número de 16 bytes (asumimos ([Uuid](crate::cassandra::aliases::types::Uuid))) representando cualquier versión de un UUID.
    Uuid,

    /// Un alias para el tipo "Text".
    ///
    /// Representa una secuencia de bytes ([ [Byte] ]) en formato UTF-8.
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

    /// Número de 16 bytes (asumimos [Uuid](crate::cassandra::aliases::types::Uuid)) representando un UUID (Versión 1) tal y como está especificado en RFC 4122.
    Timeuuid,

    /// Una secuencia de 4 o 16 bytes ([u32], [u128] o [IpAddr](std::net::IpAddr)) denotado una dirección IPv4 o IPv6 respectivamente..
    Inet,

    /// Un numero integer que representa los dias con la epoca centrada en 2^31.
    /// (unix epoch 1ro de Enero de 1970).
    /// Algunos ejemplos:
    /// 0:    -5877641-06-23
    /// 2^31: 1970-1-1
    /// 2^32: 5881580-07-11
    Date,

    /// Numero de 8 bytes en complemento a dos ([Long](crate::cassandra::aliases::types::Long)) que representa nanosegundos desde la medianoche.
    /// Los valores validos van desde 0 a 86399999999999.
    Time,

    /// Un numero de 2 bytes complemento a 2 ([Short]).
    Smallint,

    /// Un numero de 1 byte complemento a 2 ([i8]).
    Tinyint,

    /// Una duración está compuesta de 3 enteros de longitud variable con signo (vint)
    /// El primer (vint) representa una cantidad de meses
    /// El segundo (vint) representa una cantidad de días
    /// Y el tercer (vint) representa una cantidad de nanosegundos
    /// Tanto la cantidad de meses como de días deben ser un entero de 32 bits válido
    /// Mientras que la cantidad de nanosegundos debe ser un entero de 64 bits válido
    /// Una duración puede ser tanto positiva como negativa
    /// Si una duración es positiva todos los enteros deben ser positivos o cero
    /// Si es negativa todos los números deben ser negativos o cero.
    Duration,

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
    /// * `<n>` es un número de 2 bytes ([Short]) que representa la cantidad de campos a continuación.
    /// * `<name_i>` es un [String] representando el nombre del i-ésimo campo del UDT.
    /// * `<value_i>` es una tipo de los especificados en este [Enum](crate::cassandra::messages::responses::result::col_type::ColType), tal que el i-ésimo campo del UDT tiene valor de ese tipo.
    ///
    /// TODO: _Quizás meter eso en un struct en el futuro._
    Udt((String, String, Short, Vec<(String, Box<Self>)>)),

    /// El valor tiene la forma `<n><type_1>...<type_n>` donde:
    ///
    /// * `<n>` es un número de 2 bytes ([Short]) representando el número de elementos.
    /// * `<type_i>` es el [tipo](crate::cassandra::messages::responses::result::col_type::ColType) del i-ésimo valor de la tupla.
    Tuple(Vec<Box<Self>>),
}

impl Byteable for ColType {
    fn as_bytes(&self) -> Vec<Byte> {
        // OJO que esto devuelve los bytes de los posibles valores.
        match self {
            Self::Custom(nombre) => {
                let nombre_bytes = nombre.as_bytes();
                // litle endian para que los dos bytes menos significativos (los únicos que nos interesa
                // para un [Short]) estén al principio
                let bytes_len = nombre_bytes.len().to_le_bytes();
                let mut bytes_vec: Vec<Byte> = vec![
                    0x0,
                    0x0, // ID
                    bytes_len[1],
                    bytes_len[0], // Longitud del nombre
                ];
                bytes_vec.extend_from_slice(nombre_bytes);
                bytes_vec
            }
            Self::Ascii => vec![0x0, 0x1],
            Self::Bigint => vec![0x0, 0x2],
            Self::Blob => vec![0x0, 0x3],
            Self::Boolean => vec![0x0, 0x4],
            Self::Counter => vec![0x0, 0x5],
            Self::Decimal => vec![0x0, 0x6],
            Self::Double => vec![0x0, 0x7],
            Self::Float => vec![0x0, 0x8],
            Self::Int => vec![0x0, 0x9],
            Self::Timestamp => vec![0x0, 0xB], // Sí, salteamos el `0xA` a propósito
            Self::Uuid => vec![0x0, 0xC],
            Self::Varchar => vec![0x0, 0xD],
            Self::Varint => vec![0x0, 0xE],
            Self::Timeuuid => vec![0x0, 0xF],
            Self::Inet => vec![0x0, 0x10],
            Self::Date => vec![0x0, 0x11],
            Self::Time => vec![0x0, 0x12],
            Self::Smallint => vec![0x0, 0x13],
            Self::Tinyint => vec![0x0, 0x14],
            Self::Duration => vec![0x0, 0x15],
            Self::List(boxed) => {
                let mut bytes_vec: Vec<Byte> = vec![
                    0x0, 0x20, // ID
                ];
                bytes_vec.extend((**boxed).as_bytes());
                bytes_vec
            }
            Self::Map((box_key, box_val)) => {
                let mut bytes_vec: Vec<Byte> = vec![
                    0x0, 0x21, // ID
                ];
                bytes_vec.extend((**box_key).as_bytes());
                bytes_vec.extend((**box_val).as_bytes());
                bytes_vec
            }
            Self::Set(boxed) => {
                let mut bytes_vec: Vec<Byte> = vec![
                    0, 0x22, // ID
                ];
                bytes_vec.extend((**boxed).as_bytes());
                bytes_vec
            }
            Self::Udt((ks, udt_name, n, vec_campos)) => {
                let ks_bytes = ks.as_bytes();
                let ks_bytes_len = ks_bytes.len().to_le_bytes();

                let mut bytes_vec: Vec<Byte> = vec![
                    0x0,
                    0x30, // ID,
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
                    bytes_vec.extend_from_slice(nombre_bytes);
                    bytes_vec.extend((**tipo_campo).as_bytes());
                }

                bytes_vec
            }
            Self::Tuple(types_vec) => {
                let types_vec_len = types_vec.len().to_le_bytes();
                let mut bytes_vec: Vec<Byte> = vec![
                    0x0,
                    0x31, // ID
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

impl TryFrom<&mut Vec<Byte>> for ColType {
    type Error = Error;
    fn try_from(short_in_bytes: &mut Vec<Byte>) -> Result<Self, Self::Error> {
        if short_in_bytes.len() < 2 {
            return Err(Error::ConfigError("Se esperaban 2 bytes".to_string()));
        }

        let col_type_body: Vec<Byte> = short_in_bytes.split_off(2);

        let bytes_array: [Byte; 2] = match (**short_in_bytes).try_into() {
            Ok(bytes_array) => bytes_array,
            Err(_e) => {
                return Err(Error::ConfigError(
                    "No se pudo castear el vector de bytes en un array en Lenght".to_string(),
                ))
            }
        };

        let value = Short::from_be_bytes(bytes_array);

        let ret = match value {
            0x0000 => Self::deserialize_custom_type(col_type_body)?,
            0x0001 => ColType::Ascii,
            0x0002 => ColType::Bigint,
            0x0003 => ColType::Blob,
            0x0004 => ColType::Boolean,
            0x0005 => ColType::Counter,
            0x0006 => ColType::Decimal,
            0x0007 => ColType::Double,
            0x0008 => ColType::Float,
            0x0009 => ColType::Int,
            0x000B => ColType::Timestamp,
            0x000C => ColType::Uuid,
            0x000D => ColType::Varchar,
            0x000E => ColType::Varint,
            0x000F => ColType::Timeuuid,
            0x0010 => ColType::Inet,
            0x0011 => ColType::Date,
            0x0012 => ColType::Time,
            0x0013 => ColType::Smallint,
            0x0014 => ColType::Tinyint,
            0x0015 => ColType::Duration,
            0x0020 => Self::deserialize_list_type(col_type_body)?,
            0x0021 => Self::deserialize_map_type(col_type_body)?,
            0x0022 => Self::deserialize_set_type(col_type_body)?,
            0x0030 => Self::deserialize_udt_type(col_type_body)?,
            0x0031 => Self::deserialize_tuple_type(col_type_body)?,
            _ => return Err(Error::ConfigError("".to_string())),
        };
        Ok(ret)
    }
}

impl ColType {
    fn deserialize_custom_type(col_type_body: Vec<Byte>) -> Result<Self, Error> {
        if col_type_body.len() < 2 {
            return Err(Error::ConfigError(
                "Se esperaban 2 bytes Que indiquen el tamaño del string a formar".to_string(),
            ));
        }
        let custom_body = match str::from_utf8(&col_type_body) {
            Ok(str) => str,
            Err(_e) => {
                return Err(Error::ConfigError(
                    "El cuerpo del string no se pudo parsear".to_string(),
                ))
            }
        };

        Ok(ColType::Custom(custom_body.to_string()))
    }

    fn deserialize_list_type(col_type_body: Vec<Byte>) -> Result<Self, Error> {
        if col_type_body.len() < 2 {
            return Err(Error::ConfigError(
                "Se esperaban al menos 2 bytes para la lista".to_string(),
            ));
        }
        let inner_type = ColType::try_from(&mut col_type_body[2..].to_vec())?;
        Ok(ColType::List(Box::new(inner_type)))
    }

    fn deserialize_map_type(col_type_body: Vec<Byte>) -> Result<Self, Error> {
        if col_type_body.len() < 4 {
            return Err(Error::ConfigError(
                "Se esperaban al menos 4 bytes para el map".to_string(),
            ));
        }
        let key_type = ColType::try_from(&mut col_type_body[2..4].to_vec())?;
        let value_type = ColType::try_from(&mut col_type_body[4..].to_vec())?;
        Ok(ColType::Map((Box::new(key_type), Box::new(value_type))))
    }

    fn deserialize_set_type(col_type_body: Vec<Byte>) -> Result<Self, Error> {
        if col_type_body.len() < 2 {
            return Err(Error::ConfigError(
                "Se esperaban al menos 2 bytes para la lista".to_string(),
            ));
        }
        let inner_type = ColType::try_from(&mut col_type_body[2..].to_vec())?;
        Ok(ColType::Set(Box::new(inner_type)))
    }

    fn deserialize_udt_type(mut col_type_body: Vec<Byte>) -> Result<Self, Error> {
        let ks_lenght: usize = get_size_short(&mut col_type_body)?;
        let ks: String = get_string_from_bytes_with_lenght(&mut col_type_body, ks_lenght)?;

        let udt_name_lenght: usize = get_size_short(&mut col_type_body)?;
        let udt_name: String =
            get_string_from_bytes_with_lenght(&mut col_type_body, udt_name_lenght)?;

        let n: Short = get_size_short(&mut col_type_body)? as Short;

        let mut fields: Vec<(String, Box<Self>)> = Vec::new();
        for _i in 0..n {
            let lenght: usize = get_size_short(&mut col_type_body)?;

            let name_i: String = get_string_from_bytes_with_lenght(&mut col_type_body, lenght)?;
            let col_type = ColType::try_from(&mut col_type_body)?;
            let type_i = Box::new(col_type);
            fields.push((name_i, type_i));
        }
        Ok(ColType::Udt((ks, udt_name, n, fields)))
    }

    fn deserialize_tuple_type(mut col_type_body: Vec<Byte>) -> Result<Self, Error> {
        if col_type_body.len() < 2 {
            return Err(Error::ConfigError(
                "Se esperaban al menos 2 bytes para la tupla".to_string(),
            ));
        }
        let n: usize = get_size_short(&mut col_type_body)?;
        let mut types: Vec<Box<Self>> = Vec::new();
        for _i in 0..n {
            let col_type: ColType = ColType::try_from(&mut col_type_body)?;
            types.push(Box::new(col_type));
        }
        Ok(ColType::Tuple(types))
    }
}

fn get_string_from_bytes_with_lenght(
    col_type_body: &mut Vec<Byte>,
    ks_lenght: usize,
) -> Result<String, Error> {
    if col_type_body.len() < ks_lenght {
        return Err(Error::ConfigError(
            "Se esperaban mas bytes en ColType".to_string(),
        ));
    }
    let ks: Vec<Byte> = col_type_body.drain(0..ks_lenght).collect();
    let ks = match str::from_utf8(&ks) {
        Ok(str) => str,
        Err(_e) => {
            return Err(Error::ConfigError(
                "El cuerpo del string no se pudo parsear".to_string(),
            ))
        }
    };
    Ok(ks.to_string())
}

fn get_size_short(col_type_body: &mut Vec<Byte>) -> Result<usize, Error> {
    if col_type_body.len() < 2 {
        return Err(Error::ConfigError(
            "Se esperaban 2 bytes Que indiquen el tamaño del string a formar".to_string(),
        ));
    }
    let lenght: Vec<Byte> = (col_type_body.drain(0..2)).collect();
    let bytes_array: [Byte; 2] = match lenght.try_into() {
        Ok(bytes_array) => bytes_array,
        Err(_e) => {
            return Err(Error::ConfigError(
                "No se pudo castear el vector de bytes en un array en udt".to_string(),
            ))
        }
    };
    let lenght: usize = Short::from_be_bytes(bytes_array) as usize;
    Ok(lenght)
}
