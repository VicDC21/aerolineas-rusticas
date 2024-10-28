//! Módulo para los tipos de columnas en una _response_ con filas.

use crate::protocol::aliases::types::{Byte, Short};
use crate::protocol::errors::error::Error;
use crate::protocol::traits::Byteable;
use crate::protocol::user::udt_type::UdtType;
use crate::protocol::utils::parse_bytes_to_string;
use crate::server::nodes::column_data_type::ColumnDataType;

/// Tipo nativo de columna, a ser incluido en la _spec_ del cuerpo de la _response_.
#[derive(Clone)]
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
    /// Primero es precedido por un exponente ([Int](crate::protocol::aliases::types::Int)), y la base en formato [Varint](crate::protocol::messages::responses::result::col_type::ColType::Varint).
    Decimal,

    /// Un número de 8 bytes en formato IEEE 754 (Binary64) de precisión doble ([Double](crate::protocol::aliases::types::Double)).
    Double,

    /// Un número de 4 bytes en formato IEEE 754 (Binary32) de precisión simple ([Float](crate::protocol::aliases::types::Float)).
    Float,

    /// Un número de 4 bytes en complemento a dos ([Int](crate::protocol::aliases::types::Int)).
    Int,

    /// Número de 8 bytes en complemento a dos ([Long](crate::protocol::aliases::types::Long)) indicando el tiempo en milisegundos desde la
    /// _unix epoch_ (1ro de Enero de 1970, 00:00:00).
    ///
    /// Valores negativos indican una diferencia negativa a esa época.
    Timestamp,

    /// Número de 16 bytes (asumimos ([Uuid](crate::protocol::aliases::types::Uuid))) representando cualquier versión de un UUID.
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

    /// Número de 16 bytes (asumimos [Uuid](crate::protocol::aliases::types::Uuid)) representando un UUID (Versión 1) tal y como está especificado en RFC 4122.
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

    /// Numero de 8 bytes en complemento a dos ([Long](crate::protocol::aliases::types::Long)) que representa nanosegundos desde la medianoche.
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
    /// en este [Enum](crate::protocol::messages::responses::result::col_type::ColType).
    Map((Box<Self>, Box<Self>)),

    /// Un set con un tipo de columna especificado en este [Enum](crate::protocol::messages::responses::result::col_type::ColType).
    Set(Box<Self>),

    /// El valor tiene la forma `<ks><udt_name><n><name_1><type_1>...<name_n><type_n>`, donde:
    ///
    /// * `<ks>` es un [String] representado el _keyspace_ al que pertenece este UDT.
    /// * `<udt_name>` es un [String] representando el nombre del UDT.
    /// * `<n>` es un número de 2 bytes ([Short]) que representa la cantidad de campos a continuación.
    /// * `<name_i>` es un [String] representando el nombre del i-ésimo campo del UDT.
    /// * `<value_i>` es una tipo de los especificados en este [Enum](crate::protocol::messages::responses::result::col_type::ColType), tal que el i-ésimo campo del UDT tiene valor de ese tipo.
    ///
    /// Toda la info está guardada en [UdtType].
    Udt(UdtType),

    /// El valor tiene la forma `<n><type_1>...<type_n>` donde:
    ///
    /// * `<n>` es un número de 2 bytes ([Short]) representando el número de elementos.
    /// * `<type_i>` es el [tipo](crate::protocol::messages::responses::result::col_type::ColType) del i-ésimo valor de la tupla.
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
            Self::Udt(udt_type) => udt_type.as_bytes(),
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

impl TryFrom<&[Byte]> for ColType {
    type Error = Error;
    fn try_from(bytes: &[Byte]) -> Result<Self, Self::Error> {
        if bytes.len() < 2 {
            return Err(Error::ConfigError("Se esperaban 2 bytes".to_string()));
        }

        let col_type_body = &bytes[2..];

        let bytes_arr: [Byte; 2] = match bytes[0..2].try_into() {
            Ok(bytes_array) => bytes_array,
            Err(_e) => {
                return Err(Error::ConfigError(
                    "No se pudo castear el vector de bytes en un array en Lenght".to_string(),
                ))
            }
        };

        let value = Short::from_be_bytes(bytes_arr);

        let ret = match value {
            0x0000 => ColType::Custom(parse_bytes_to_string(col_type_body, &mut 0)?),
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
            0x0030 => ColType::Udt(UdtType::try_from(col_type_body)?),
            0x0031 => Self::deserialize_tuple_type(col_type_body)?,
            _ => {
                return Err(Error::ConfigError(
                    "El ID dado no corresponde a ninguna variante".to_string(),
                ))
            }
        };
        Ok(ret)
    }
}

impl ColType {
    fn deserialize_list_type(col_type_body: &[Byte]) -> Result<Self, Error> {
        if col_type_body.len() < 2 {
            return Err(Error::ConfigError(
                "Se esperaban al menos 2 bytes para el tipo de la lista".to_string(),
            ));
        }
        let inner_type = ColType::try_from(col_type_body)?;
        Ok(ColType::List(Box::new(inner_type)))
    }

    fn deserialize_map_type(col_type_body: &[Byte]) -> Result<Self, Error> {
        if col_type_body.len() < 4 {
            return Err(Error::ConfigError(
                "Se esperaban al menos 4 bytes para el map".to_string(),
            ));
        }
        let key_type = ColType::try_from(col_type_body)?;
        let key_len = key_type.as_bytes().len();
        let value_type = ColType::try_from(&col_type_body[key_len..])?;
        Ok(ColType::Map((Box::new(key_type), Box::new(value_type))))
    }

    fn deserialize_set_type(col_type_body: &[Byte]) -> Result<Self, Error> {
        if col_type_body.len() < 2 {
            return Err(Error::ConfigError(
                "Se esperaban al menos 2 bytes para la lista".to_string(),
            ));
        }
        let inner_type = ColType::try_from(col_type_body)?;
        Ok(ColType::Set(Box::new(inner_type)))
    }

    fn deserialize_tuple_type(col_type_body: &[Byte]) -> Result<Self, Error> {
        if col_type_body.len() < 2 {
            return Err(Error::ConfigError(
                "Se esperaban al menos 2 bytes para la tupla".to_string(),
            ));
        }
        let n = Short::from_be_bytes([col_type_body[0], col_type_body[1]]);
        let mut types: Vec<Box<Self>> = Vec::new();
        let mut cur_type_body = col_type_body;
        for _ in 0..n {
            let col_type: ColType = ColType::try_from(col_type_body)?;
            cur_type_body = &cur_type_body[col_type.as_bytes().len()..];
            types.push(Box::new(col_type));
        }
        Ok(ColType::Tuple(types))
    }
}

impl From<ColumnDataType> for ColType {
    fn from(col: ColumnDataType) -> Self {
        match col {
            ColumnDataType::String => ColType::Varchar,
            ColumnDataType::Timestamp => ColType::Timestamp,
            ColumnDataType::Double => ColType::Double,
            ColumnDataType::Int => ColType::Int,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::protocol::aliases::types::Byte;
    use crate::protocol::errors::error::Error;
    use crate::protocol::messages::responses::result::col_type::ColType;
    use crate::protocol::traits::Byteable;

    #[test]
    fn test_1_serializar() {
        let simple_types: Vec<ColType> = vec![
            ColType::Ascii,
            ColType::Bigint,
            ColType::Blob,
            ColType::Boolean,
            ColType::Counter,
            ColType::Decimal,
            ColType::Double,
            ColType::Float,
            ColType::Int,
            ColType::Timestamp,
            ColType::Uuid,
            ColType::Varchar,
            ColType::Varint,
            ColType::Timeuuid,
            ColType::Inet,
            ColType::Date,
            ColType::Time,
            ColType::Smallint,
            ColType::Tinyint,
            ColType::Duration,
        ];
        let simple_ids: Vec<[Byte; 2]> = vec![
            [0x0, 0x1],
            [0x0, 0x2],
            [0x0, 0x3],
            [0x0, 0x4],
            [0x0, 0x5],
            [0x0, 0x6],
            [0x0, 0x7],
            [0x0, 0x8],
            [0x0, 0x9],
            [0x0, 0xB],
            [0x0, 0xC],
            [0x0, 0xD],
            [0x0, 0xE],
            [0x0, 0xF],
            [0x0, 0x10],
            [0x0, 0x11],
            [0x0, 0x12],
            [0x0, 0x13],
            [0x0, 0x14],
            [0x0, 0x15],
        ];

        for i in 0..simple_types.len() {
            assert_eq!(simple_types[i].as_bytes(), simple_ids[i]);
        }

        let custom = ColType::Custom("Tipo Custom".to_string());
        assert_eq!(
            custom.as_bytes(),
            [
                0x0, 0x0, 0x0, 0xB, 0x54, 0x69, 0x70, 0x6F, 0x20, 0x43, 0x75, 0x73, 0x74, 0x6F,
                0x6D
            ]
        );
    }

    #[test]
    fn test_2_deserializar() {
        let bool_set_res = ColType::try_from(&[0x0, 0x22, 0x0, 0x4][..]);

        assert!(bool_set_res.is_ok());
        if let Ok(bool_set) = bool_set_res {
            assert!(matches!(bool_set, ColType::Set(_)));
            if let ColType::Set(set_type) = bool_set {
                assert!(matches!(*set_type, ColType::Boolean));
            }
        }
    }

    #[test]
    fn test_3_id_incorrecto() {
        let inexistente = ColType::try_from(&[0x0, 0xF4][..]);

        assert!(inexistente.is_err());
        if let Err(err) = inexistente {
            assert!(matches!(err, Error::ConfigError(_)));
        }
    }

    #[test]
    fn test_4_muy_corto() {
        let corto = ColType::try_from(&[0x1][..]);

        assert!(corto.is_err());
        if let Err(err) = corto {
            assert!(matches!(err, Error::ConfigError(_)));
        }
    }

    #[test]
    fn test_5_lista_muy_corta() {
        let lista_res = ColType::try_from(&[0x0, 0x20][..]);

        assert!(lista_res.is_err());
        if let Err(lista_err) = lista_res {
            assert!(matches!(lista_err, Error::ConfigError(_)));
        }
    }

    #[test]
    fn test_6_mapa_muy_corto() {
        let mapa_res = ColType::try_from(&[0x0, 0x21, 0x2][..]);

        assert!(mapa_res.is_err());
        if let Err(mapa_err) = mapa_res {
            assert!(matches!(mapa_err, Error::ConfigError(_)));
        }
    }

    #[test]
    fn test_7_set_muy_corto() {
        let set_res = ColType::try_from(&[0x0, 0x22, 0x6][..]);

        assert!(set_res.is_err());
        if let Err(set_err) = set_res {
            assert!(matches!(set_err, Error::ConfigError(_)));
        }
    }

    #[test]
    fn test_8_tupla_muy_corta() {
        let tupla_res = ColType::try_from(&[0x0, 0x31, 0xF][..]);

        assert!(tupla_res.is_err());
        if let Err(tupla_err) = tupla_res {
            assert!(matches!(tupla_err, Error::ConfigError(_)));
        }
    }
}
