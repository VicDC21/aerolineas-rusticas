//! Módulo para mensajes de errores.

use std::{
    backtrace::Backtrace,
    fmt::{Display, Formatter, Result as FmtResult},
    result::Result as StdResult,
};

use crate::protocol::aliases::types::{Byte, Int, ReasonMap, Short};
use crate::protocol::errors::write_type::WriteType;
use crate::protocol::utils::{
    encode_reasonmap_to_bytes, encode_string_to_bytes, parse_bytes_to_reasonmap,
    parse_bytes_to_string,
};
use crate::protocol::{notations::consistency::Consistency, traits::Byteable};

/// La forma del mensaje de error es `<code><message>[...]`.
/// Luego, dependiendo del código de error, tendrá más información o no luego del mensaje.
pub enum Error {
    /// Un error del lado del servidor.
    ServerError(String),

    /// Un mensaje del cliente ocasionó una violación de protocolo.
    ProtocolError(String),

    /// La autenticación era requerida y falló.
    AuthenticationError(String),

    /// Un nodo no se encontraba disponible para responder a la query.
    ///
    /// El resto del mensaje es `<cl><required><alive>`, donde:
    /// * `<cl>` es el nivel de [Consistency] de la query que lanzó esta excepción.
    /// * `<required>` es un número ([Int]) que representa la cantidad de nodos que deberían estar disponibles para respetar `<cl>`.
    /// * `<alive>` es un número ([Int]) que representa la cantidad de réplicas que se sabía que estaban disponibles cuando el request había sido procesado (como se lanzó ésta excepción, se sabe que `<alive> < <required>`).
    UnavailableException(String, Consistency, Int, Int),

    /// El request no puede ser procesado porque el nodo coordinador está sobrecargado.
    Overloaded(String),

    /// El request fue de lectura pero el nodo coordinador estaba en proceso de boostrapping (inicialización).
    IsBootstrapping(String),

    /// Un error de trucamiento.
    TruncateError(String),

    /// Timeout exception durante un request de escritura.
    ///
    /// El resto del mensaje es `<cl><received><blockfor><writeType><contentions>`, donde:
    /// * `<cl>` es el nivel de [Consistency] de la query que lanzó esta excepción.
    /// * `<received>` es un número ([Int]) que representa la cantidad de nodos que han reconocido la request.
    /// * `<blockfor>` es un número ([Int]) que representa la cantidad de réplicas cuya confirmación es necesaria para cumplir `<cl>`.
    /// * `<writeType>` es un [WriteType] que representa el tipo de escritura que se estaba intentando realizar.
    /// * `<contentions>` es un número ([Short]) que representa la cantidad de contenciones ocurridas durante la operación CAS. Este campo solo se presenta cuando el writeType es "CAS".
    WriteTimeout(String, Consistency, Int, Int, WriteType, Option<Short>),

    /// Timeout exception durante un request de lectura.
    ///
    /// El resto del mensaje es `<cl><received><blockfor><data_present>`, donde:
    /// * `<cl>` es el nivel de [Consistency] de la query que lanzó esta excepción.
    /// * `<received>` es un número ([Int]) que representa la cantidad de nodos que han respondido a la request.
    /// * `<blockfor>` es un número ([Int]) que representa la cantidad de réplicas cuya respuesta es necesaria para cumplir `cl`. Notar que es posible tener `<received> >= <blockfor>` si <data_present> es false. También en el caso (improbable) donde cl se cumple pero el nodo coordinador sufre un timeout mientras esperaba por la confirmación de un read-repair.
    /// * `<data_present>` es un [bool] que indica si el nodo al que se le hizo el pedido de la data respondió o no.
    ReadTimeout(String, Consistency, Int, Int, bool),

    /// Una excepción de lectura que no fue ocasionada por un timeout.
    ///
    /// El resto del mensaje es `<cl><received><blockfor><reasonmap><data_present>`, donde:
    /// * `<cl>` es el nivel de [Consistency] de la query que lanzó esta excepción.
    /// * `<received>` es un número ([Int]) que representa la cantidad de nodos que han respondido a la request.
    /// * `<blockfor>` es un número ([Int]) que representa la cantidad de réplicas cuya respuesta es necesaria para cumplir `<cl>`.
    /// * `<reasonmap>` es un "mapa" de endpoints a códigos de razón de error. Esto mapea los endpoints de los nodos réplica que fallaron al ejecutar la request a un código representando la razón del error. La forma del mapa es empezando con un [Int] n seguido por n pares de endpoint,failurecode donde endpoint es un [IpAddr] y failurecode es un [Short].
    /// * `<data_present>` es un [bool] que indica si el nodo al que se le hizo el pedido de la data respondió o no.
    ReadFailure(String, Consistency, Int, Int, ReasonMap, bool),

    /// Una función (definida por el usuario) falló durante su ejecución.
    ///
    /// El resto del mensaje es `<keyspace><function><arg_types>`, donde:
    /// * `<keyspace>` es un [String] representando el _keyspace_ en el que se encuentra la función.
    /// * `<function>` es un [String] representando el nombre de la función.
    /// * `<arg_types>` es una lista de [String] representando los tipos (en tipo CQL) de los argumentos de la función.
    FunctionFailure(String, String, String, Vec<String>),

    /// Una excepción de escritura que no fue ocasionada por un timeout.
    ///
    /// El resto del mensaje es `<cl><received><blockfor><reasonmap><write_type>`, donde:
    /// * `<cl>` es el nivel de [Consistency] de la query que lanzó esta excepción.
    /// * `<received>` es un número ([Int]) que representa la cantidad de nodos que han respondido a la request.
    /// * `<blockfor>` es un número ([Int]) que representa la cantidad de réplicas cuya confirmación es necesaria para cumplir `<cl>`.
    /// * `<reasonmap>` es un "mapa" de endpoints a códigos de razón de error. Esto mapea los endpoints de los nodos réplica que fallaron al ejecutar la request a un código representando la razón del error. La forma del mapa es empezando con un [Int] n seguido por n pares de endpoint, failurecode donde endpoint es un [IpAddr] y failurecode es un [Short].
    /// * `<writeType>` es un [WriteType] que representa el tipo de escritura que se estaba intentando realizar.
    WriteFailure(String, Consistency, Int, Int, ReasonMap, WriteType),

    /// _En la documentación del protocolo de Cassandra figura como TODO_.
    CDCWriteFailure(String),

    /// Una excepción ocurrida debido a una operación _Compare And Set write/update_ en contención. La operación CAS fue completada solo parcialmente y la operación puede o no ser completada por la escritura CAS contenedora o la lectura SERIAL/LOCAL_SERIAL.
    ///
    /// El resto del mensaje es `<cl><received><blockfor>`, donde:
    /// * `<cl>` es el nivel de [Consistency] de la query que lanzó esta excepción.
    /// * `<received>` es un número ([Int]) que representa la cantidad de nodos que han reconocido la request.
    /// * `<blockfor>` es un número ([Int]) que representa la cantidad de réplicas cuya confirmación es necesaria para cumplir `<cl>`.
    CASWriteUnknown(String, Consistency, Int, Int),

    /// La query enviada tiene un error de sintaxis.
    SyntaxError(String),

    /// El usuario logueado no tiene los permisos necesarios para realizar la query.
    Unauthorized(String),

    /// La query es sintácticamente correcta pero inválida.
    Invalid(String),

    /// La query es inválida debido a algún problema de configuración.
    ConfigError(String),

    /// La query intentó crear un _keyspace_ o una tabla que ya existía.
    ///
    /// El resto del mensaje es `<ks><table>`, donde:
    /// * `<ks>` es un [String] representando el _keyspace_ que ya existía, o el _keyspace_ al que pertenece la tabla que ya existía.
    /// * `<table>` es un [String] representando el nombre de la tabla que ya existía. Si la query intentó crear un _keyspace_, table estará presente pero será el string vacío.
    AlreadyExists(String, String, String),

    /// Puede ser lanzado mientras una expresión preparada intenta ser ejecutada si el ID de la misma no es conocido por este host.
    ///
    /// El resto del mensaje es `<id>`, `id` siendo una lista de números ([Byte]) representando el ID desconocido.
    Unprepared(String, Vec<Byte>),
}

impl Byteable for Error {
    fn as_bytes(&self) -> Vec<Byte> {
        match self {
            Self::ServerError(msg) => {
                let mut bytes_vec: Vec<Byte> = vec![
                    0x0, 0x0, 0x0, 0x0, // ID
                ];
                bytes_vec.extend(encode_string_to_bytes(msg));
                bytes_vec
            }
            Self::ProtocolError(msg) => {
                let mut bytes_vec: Vec<Byte> = vec![
                    0x0, 0x0, 0x0, 0xA, // ID
                ];
                bytes_vec.extend(encode_string_to_bytes(msg));
                bytes_vec
            }
            Self::AuthenticationError(msg) => {
                let mut bytes_vec: Vec<Byte> = vec![
                    0x0, 0x0, 0x1, 0x0, // ID
                ];
                bytes_vec.extend(encode_string_to_bytes(msg));
                bytes_vec
            }
            Self::UnavailableException(msg, cl, required, alive) => {
                let mut bytes_vec: Vec<Byte> = vec![
                    0x0, 0x0, 0x10, 0x0, // ID
                ];
                bytes_vec.extend(encode_string_to_bytes(msg));
                bytes_vec.extend(cl.as_bytes());
                bytes_vec.extend(required.to_be_bytes());
                bytes_vec.extend(alive.to_be_bytes());
                bytes_vec
            }
            Self::Overloaded(msg) => {
                let mut bytes_vec: Vec<Byte> = vec![
                    0x0, 0x0, 0x10, 0x1, // ID
                ];
                bytes_vec.extend(encode_string_to_bytes(msg));
                bytes_vec
            }
            Self::IsBootstrapping(msg) => {
                let mut bytes_vec: Vec<Byte> = vec![
                    0x0, 0x0, 0x10, 0x2, // ID
                ];
                bytes_vec.extend(encode_string_to_bytes(msg));
                bytes_vec
            }
            Self::TruncateError(msg) => {
                let mut bytes_vec: Vec<Byte> = vec![
                    0x0, 0x0, 0x10, 0x3, // ID
                ];
                bytes_vec.extend(encode_string_to_bytes(msg));
                bytes_vec
            }
            Self::WriteTimeout(msg, cl, received, blockfor, write_type, contentions) => {
                let mut bytes_vec: Vec<Byte> = vec![
                    0x0, 0x0, 0x11, 0x0, // ID
                ];
                bytes_vec.extend(encode_string_to_bytes(msg));
                bytes_vec.extend(cl.as_bytes());
                bytes_vec.extend(received.to_be_bytes());
                bytes_vec.extend(blockfor.to_be_bytes());
                bytes_vec.extend(write_type.as_bytes());
                if matches!(write_type, WriteType::Cas) {
                    if let Some(content) = contentions {
                        bytes_vec.extend(content.to_be_bytes());
                    }
                }
                bytes_vec
            }
            Self::ReadTimeout(msg, cl, received, blockfor, data_present) => {
                let mut bytes_vec: Vec<Byte> = vec![
                    0x0, 0x0, 0x12, 0x0, // ID
                ];
                bytes_vec.extend(encode_string_to_bytes(msg));
                bytes_vec.extend(cl.as_bytes());
                bytes_vec.extend(received.to_be_bytes());
                bytes_vec.extend(blockfor.to_be_bytes());
                bytes_vec.push(if *data_present { 0x1 } else { 0x0 });
                bytes_vec
            }
            Self::ReadFailure(msg, cl, received, blockfor, reasonmap, data_present) => {
                let mut bytes_vec: Vec<Byte> = vec![
                    0x0, 0x0, 0x13, 0x0, // ID
                ];
                bytes_vec.extend(encode_string_to_bytes(msg));
                bytes_vec.extend(cl.as_bytes());
                bytes_vec.extend(received.to_be_bytes());
                bytes_vec.extend(blockfor.to_be_bytes());
                bytes_vec.extend(encode_reasonmap_to_bytes(reasonmap));
                bytes_vec.push(if *data_present { 0x1 } else { 0x0 });
                bytes_vec
            }
            Self::FunctionFailure(msg, keyspace, function, arg_types) => {
                let mut bytes_vec: Vec<Byte> = vec![
                    0x0, 0x0, 0x14, 0x0, // ID
                ];
                bytes_vec.extend(encode_string_to_bytes(msg));
                bytes_vec.extend(encode_string_to_bytes(keyspace));
                bytes_vec.extend(encode_string_to_bytes(function));

                let list_len = arg_types.len().to_le_bytes();
                bytes_vec.extend_from_slice(&[list_len[1], list_len[0]]);
                for string in arg_types {
                    bytes_vec.extend(encode_string_to_bytes(string));
                }

                bytes_vec
            }
            Self::WriteFailure(msg, cl, received, blockfor, reasonmap, write_type) => {
                let mut bytes_vec: Vec<Byte> = vec![
                    0x0, 0x0, 0x15, 0x0, // ID
                ];
                bytes_vec.extend(encode_string_to_bytes(msg));
                bytes_vec.extend(cl.as_bytes());
                bytes_vec.extend(received.to_be_bytes());
                bytes_vec.extend(blockfor.to_be_bytes());
                bytes_vec.extend(encode_reasonmap_to_bytes(reasonmap));
                bytes_vec.extend(write_type.as_bytes());
                bytes_vec
            }
            Self::CDCWriteFailure(msg) => {
                let mut bytes_vec: Vec<Byte> = vec![
                    0x0, 0x0, 0x16, 0x0, // ID
                ];
                bytes_vec.extend(encode_string_to_bytes(msg));
                bytes_vec
            }
            Self::CASWriteUnknown(msg, cl, received, blockfor) => {
                let mut bytes_vec: Vec<Byte> = vec![
                    0x0, 0x0, 0x17, 0x0, // ID
                ];
                bytes_vec.extend(encode_string_to_bytes(msg));
                bytes_vec.extend(cl.as_bytes());
                bytes_vec.extend(received.to_be_bytes());
                bytes_vec.extend(blockfor.to_be_bytes());
                bytes_vec
            }
            Self::SyntaxError(msg) => {
                let mut bytes_vec: Vec<Byte> = vec![
                    0x0, 0x0, 0x20, 0x0, // ID
                ];
                bytes_vec.extend(encode_string_to_bytes(msg));
                bytes_vec
            }
            Self::Unauthorized(msg) => {
                let mut bytes_vec: Vec<Byte> = vec![
                    0x0, 0x0, 0x21, 0x0, // ID
                ];
                bytes_vec.extend(encode_string_to_bytes(msg));
                bytes_vec
            }
            Self::Invalid(msg) => {
                let mut bytes_vec: Vec<Byte> = vec![
                    0x0, 0x0, 0x22, 0x0, // ID
                ];
                bytes_vec.extend(encode_string_to_bytes(msg));
                bytes_vec
            }
            Self::ConfigError(msg) => {
                let mut bytes_vec: Vec<Byte> = vec![
                    0x0, 0x0, 0x23, 0x0, // ID
                ];
                bytes_vec.extend(encode_string_to_bytes(msg));
                bytes_vec
            }
            Self::AlreadyExists(msg, ks, table) => {
                let mut bytes_vec: Vec<Byte> = vec![
                    0x0, 0x0, 0x24, 0x0, // ID
                ];
                bytes_vec.extend(encode_string_to_bytes(msg));
                bytes_vec.extend(encode_string_to_bytes(ks));
                bytes_vec.extend(encode_string_to_bytes(table));
                bytes_vec
            }
            Self::Unprepared(msg, ids) => {
                let mut bytes_vec: Vec<Byte> = vec![
                    0x0, 0x0, 0x25, 0x0, // ID
                ];
                bytes_vec.extend(encode_string_to_bytes(msg));
                let ids_len = ids.len().to_le_bytes();
                bytes_vec.extend(&[ids_len[1], ids_len[0]]);
                bytes_vec.extend(ids);
                bytes_vec
            }
        }
    }
}

impl TryFrom<Vec<Byte>> for Error {
    type Error = Error;
    fn try_from(bytes_vec: Vec<Byte>) -> StdResult<Self, Self::Error> {
        let mut i = 4;
        match bytes_vec[..i] {
            [0x0, 0x0, 0x0, 0x0] => match parse_bytes_to_string(&bytes_vec[i..], &mut i) {
                Ok(msg) => Ok(Self::ServerError(msg)),
                Err(err) => Err(err),
            },
            [0x0, 0x0, 0x0, 0xA] => match parse_bytes_to_string(&bytes_vec[i..], &mut i) {
                Ok(msg) => Ok(Self::ProtocolError(msg)),
                Err(err) => Err(err),
            },
            [0x0, 0x0, 0x1, 0x0] => match parse_bytes_to_string(&bytes_vec[i..], &mut i) {
                Ok(msg) => Ok(Self::AuthenticationError(msg)),
                Err(err) => Err(err),
            },
            [0x0, 0x0, 0x10, 0x0] => match parse_bytes_to_string(&bytes_vec[i..], &mut i) {
                Ok(msg) => {
                    let cl = Consistency::try_from(&bytes_vec[i..])?;
                    i += 2;
                    let required = Int::from_be_bytes([
                        bytes_vec[i],
                        bytes_vec[i + 1],
                        bytes_vec[i + 2],
                        bytes_vec[i + 3],
                    ]);
                    i += 4;
                    let alive = Int::from_be_bytes([
                        bytes_vec[i],
                        bytes_vec[i + 1],
                        bytes_vec[i + 2],
                        bytes_vec[i + 3],
                    ]);
                    Ok(Self::UnavailableException(msg, cl, required, alive))
                }
                Err(err) => Err(err),
            },
            [0x0, 0x0, 0x10, 0x1] => match parse_bytes_to_string(&bytes_vec[i..], &mut i) {
                Ok(msg) => Ok(Self::Overloaded(msg)),
                Err(err) => Err(err),
            },
            [0x0, 0x0, 0x10, 0x2] => match parse_bytes_to_string(&bytes_vec[i..], &mut i) {
                Ok(msg) => Ok(Self::IsBootstrapping(msg)),
                Err(err) => Err(err),
            },
            [0x0, 0x0, 0x10, 0x3] => match parse_bytes_to_string(&bytes_vec[i..], &mut i) {
                Ok(msg) => Ok(Self::TruncateError(msg)),
                Err(err) => Err(err),
            },
            [0x0, 0x0, 0x11, 0x0] => match parse_bytes_to_string(&bytes_vec[i..], &mut i) {
                Ok(msg) => {
                    let cl = Consistency::try_from(&bytes_vec[i..])?;
                    i += 2;
                    let received = Int::from_be_bytes([
                        bytes_vec[i],
                        bytes_vec[i + 1],
                        bytes_vec[i + 2],
                        bytes_vec[i + 3],
                    ]);
                    i += 4;
                    let blockfor = Int::from_be_bytes([
                        bytes_vec[i],
                        bytes_vec[i + 1],
                        bytes_vec[i + 2],
                        bytes_vec[i + 3],
                    ]);
                    i += 4;
                    let write_type = WriteType::try_from(&bytes_vec[i..])?;
                    i += write_type.as_bytes().len();
                    let contentions = if matches!(write_type, WriteType::Cas) {
                        if i + 1 >= bytes_vec.len() {
                            return Err(Self::SyntaxError("Se esperaban 2 bytes más para el campo <contentions> del error WriteTimeout".to_string()));
                        }
                        let cont = Short::from_be_bytes([bytes_vec[i], bytes_vec[i + 1]]);
                        Some(cont)
                    } else {
                        None
                    };
                    Ok(Self::WriteTimeout(
                        msg,
                        cl,
                        received,
                        blockfor,
                        write_type,
                        contentions,
                    ))
                }
                Err(err) => Err(err),
            },
            [0x0, 0x0, 0x12, 0x0] => match parse_bytes_to_string(&bytes_vec[i..], &mut i) {
                Ok(msg) => {
                    let cl = Consistency::try_from(&bytes_vec[i..])?;
                    i += 2;
                    let received = Int::from_be_bytes([
                        bytes_vec[i],
                        bytes_vec[i + 1],
                        bytes_vec[i + 2],
                        bytes_vec[i + 3],
                    ]);
                    i += 4;
                    let blockfor = Int::from_be_bytes([
                        bytes_vec[i],
                        bytes_vec[i + 1],
                        bytes_vec[i + 2],
                        bytes_vec[i + 3],
                    ]);
                    i += 4;
                    let data_present = bytes_vec[i] != 0x0;
                    Ok(Self::ReadTimeout(msg, cl, received, blockfor, data_present))
                }
                Err(err) => Err(err),
            },
            [0x0, 0x0, 0x13, 0x0] => match parse_bytes_to_string(&bytes_vec[i..], &mut i) {
                Ok(msg) => {
                    let cl = Consistency::try_from(&bytes_vec[i..])?;
                    i += 2;
                    let received = Int::from_be_bytes([
                        bytes_vec[i],
                        bytes_vec[i + 1],
                        bytes_vec[i + 2],
                        bytes_vec[i + 3],
                    ]);
                    i += 4;
                    let blockfor = Int::from_be_bytes([
                        bytes_vec[i],
                        bytes_vec[i + 1],
                        bytes_vec[i + 2],
                        bytes_vec[i + 3],
                    ]);
                    i += 4;
                    let reasonmap = parse_bytes_to_reasonmap(&bytes_vec[i..], &mut i)?;
                    let data_present = bytes_vec[i] != 0x0;
                    Ok(Self::ReadFailure(
                        msg,
                        cl,
                        received,
                        blockfor,
                        reasonmap,
                        data_present,
                    ))
                }
                Err(err) => Err(err),
            },
            [0x0, 0x0, 0x14, 0x0] => match parse_bytes_to_string(&bytes_vec[i..], &mut i) {
                Ok(msg) => {
                    let keyspace = parse_bytes_to_string(&bytes_vec[i..], &mut i)?;
                    let function = parse_bytes_to_string(&bytes_vec[i..], &mut i)?;
                    let arg_types_len = Short::from_le_bytes([bytes_vec[i + 1], bytes_vec[i]]);
                    i += 2;
                    let mut arg_types: Vec<String> = vec![];
                    for _ in 0..arg_types_len {
                        match parse_bytes_to_string(&bytes_vec[i..], &mut i) {
                            Ok(string) => arg_types.push(string),
                            Err(err) => return Err(err),
                        }
                    }
                    Ok(Self::FunctionFailure(msg, keyspace, function, arg_types))
                }
                Err(err) => Err(err),
            },
            [0x0, 0x0, 0x15, 0x0] => match parse_bytes_to_string(&bytes_vec[i..], &mut i) {
                Ok(msg) => {
                    let cl = Consistency::try_from(&bytes_vec[i..])?;
                    i += 2;
                    let received = Int::from_be_bytes([
                        bytes_vec[i],
                        bytes_vec[i + 1],
                        bytes_vec[i + 2],
                        bytes_vec[i + 3],
                    ]);
                    i += 4;
                    let blockfor = Int::from_be_bytes([
                        bytes_vec[i],
                        bytes_vec[i + 1],
                        bytes_vec[i + 2],
                        bytes_vec[i + 3],
                    ]);
                    i += 4;
                    let reasonmap = parse_bytes_to_reasonmap(&bytes_vec[i..], &mut i)?;
                    let write_type = WriteType::try_from(&bytes_vec[i..])?;
                    Ok(Self::WriteFailure(
                        msg, cl, received, blockfor, reasonmap, write_type,
                    ))
                }
                Err(err) => Err(err),
            },
            [0x0, 0x0, 0x16, 0x0] => match parse_bytes_to_string(&bytes_vec[i..], &mut i) {
                Ok(msg) => Ok(Self::CDCWriteFailure(msg)),
                Err(err) => Err(err),
            },
            [0x0, 0x0, 0x17, 0x0] => match parse_bytes_to_string(&bytes_vec[i..], &mut i) {
                Ok(msg) => {
                    let cl = Consistency::try_from(&bytes_vec[i..])?;
                    i += 2;
                    let received = Int::from_be_bytes([
                        bytes_vec[i],
                        bytes_vec[i + 1],
                        bytes_vec[i + 2],
                        bytes_vec[i + 3],
                    ]);
                    i += 4;
                    let blockfor = Int::from_be_bytes([
                        bytes_vec[i],
                        bytes_vec[i + 1],
                        bytes_vec[i + 2],
                        bytes_vec[i + 3],
                    ]);
                    Ok(Self::CASWriteUnknown(msg, cl, received, blockfor))
                }
                Err(err) => Err(err),
            },
            [0x0, 0x0, 0x20, 0x0] => match parse_bytes_to_string(&bytes_vec[i..], &mut i) {
                Ok(msg) => Ok(Self::SyntaxError(msg)),
                Err(err) => Err(err),
            },
            [0x0, 0x0, 0x21, 0x0] => match parse_bytes_to_string(&bytes_vec[i..], &mut i) {
                Ok(msg) => Ok(Self::Unauthorized(msg)),
                Err(err) => Err(err),
            },
            [0x0, 0x0, 0x22, 0x0] => match parse_bytes_to_string(&bytes_vec[i..], &mut i) {
                Ok(msg) => Ok(Self::Invalid(msg)),
                Err(err) => Err(err),
            },
            [0x0, 0x0, 0x23, 0x0] => match parse_bytes_to_string(&bytes_vec[i..], &mut i) {
                Ok(msg) => Ok(Self::ConfigError(msg)),
                Err(err) => Err(err),
            },
            [0x0, 0x0, 0x24, 0x0] => match parse_bytes_to_string(&bytes_vec[i..], &mut i) {
                Ok(msg) => {
                    let ks = parse_bytes_to_string(&bytes_vec[i..], &mut i)?;
                    let table = parse_bytes_to_string(&bytes_vec[i..], &mut i)?;
                    Ok(Self::AlreadyExists(msg, ks, table))
                }
                Err(err) => Err(err),
            },
            [0x0, 0x0, 0x25, 0x0] => match parse_bytes_to_string(&bytes_vec[i..], &mut i) {
                Ok(msg) => {
                    let ids_len = Short::from_le_bytes([bytes_vec[i + 1], bytes_vec[i]]);
                    i += 2;
                    let mut ids: Vec<Byte> = vec![];
                    for _ in 0..ids_len {
                        ids.push(bytes_vec[i]);
                        i += 1;
                    }
                    Ok(Self::Unprepared(msg, ids))
                }
                Err(err) => Err(err),
            },
            _ => Err(Self::Invalid("El ID del error no es válido".to_string())),
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let backtrace = Backtrace::force_capture();
        match self {
            Self::ServerError(msg) => write!(f, "{}\nServerError: {}\n", backtrace, msg),
            Self::ProtocolError(msg) => write!(f, "{}\nProtocolError: {}\n", backtrace, msg),
            Self::AuthenticationError(msg) => {
                write!(f, "{}\nAuthenticationError: {}\n", backtrace, msg)
            }
            Self::UnavailableException(msg, _, _, _) => {
                write!(f, "{}\nUnavailableException: {}\n", backtrace, msg)
            }
            Self::Overloaded(msg) => write!(f, "{}\nOverloaded: {}\n", backtrace, msg),
            Self::IsBootstrapping(msg) => write!(f, "{}\nIsBootstrapping: {}\n", backtrace, msg),
            Self::TruncateError(msg) => write!(f, "{}\nTruncateError: {}\n", backtrace, msg),
            Self::WriteTimeout(msg, _, _, _, _, _) => {
                write!(f, "{}\nWriteTimeout: {}\n", backtrace, msg)
            }
            Self::ReadTimeout(msg, _, _, _, _) => {
                write!(f, "{}\nReadTimeout: {}\n", backtrace, msg)
            }
            Self::ReadFailure(msg, _, _, _, _, _) => {
                write!(f, "{}\nReadFailure: {}\n", backtrace, msg)
            }
            Self::FunctionFailure(msg, _, _, _) => {
                write!(f, "{}\nFunctionFailure: {}\n", backtrace, msg)
            }
            Self::WriteFailure(msg, _, _, _, _, _) => {
                write!(f, "{}\nWriteFailure: {}\n", backtrace, msg)
            }
            Self::CDCWriteFailure(msg) => write!(f, "{}\nCDCWriteFailure: {}\n", backtrace, msg),
            Self::CASWriteUnknown(msg, _, _, _) => {
                write!(f, "{}\nCASWriteUnknown: {}\n", backtrace, msg)
            }
            Self::SyntaxError(msg) => write!(f, "{}\nSyntaxError: {}\n", backtrace, msg),
            Self::Unauthorized(msg) => write!(f, "{}\nUnauthorized: {}\n", backtrace, msg),
            Self::Invalid(msg) => write!(f, "{}\nInvalid: {}\n", backtrace, msg),
            Self::ConfigError(msg) => write!(f, "{}\nConfigError: {}\n", backtrace, msg),
            Self::AlreadyExists(msg, _, _) => {
                write!(f, "{}\nAlreadyExists: {}\n", backtrace, msg)
            }
            Self::Unprepared(msg, _) => write!(f, "{}\nUnprepared: {}\n", backtrace, msg),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::net::{IpAddr, Ipv4Addr};

    use super::Error;
    use crate::protocol::{
        aliases::types::ReasonMap, errors::write_type::WriteType,
        notations::consistency::Consistency, traits::Byteable,
    };

    #[test]
    fn test_1_serializar() {
        let error = Error::ServerError("Error".to_string());
        let expected = vec![
            0x0, 0x0, 0x0, 0x0, // ID
            0x0, 0x5, // len msg
            0x45, 0x72, 0x72, 0x6F, 0x72, // msg
        ];
        assert_eq!(error.as_bytes(), expected);

        let error = Error::UnavailableException("Error".to_string(), Consistency::Three, 3, 2);
        let expected = vec![
            0x0, 0x0, 0x10, 0x0, // ID
            0x0, 0x5, // len msg
            0x45, 0x72, 0x72, 0x6F, 0x72, // msg
            0x0, 0x3, // cl
            0x0, 0x0, 0x0, 0x3, // required
            0x0, 0x0, 0x0, 0x2, // alive
        ];
        assert_eq!(error.as_bytes(), expected);

        let error = Error::WriteTimeout(
            "Error".to_string(),
            Consistency::Three,
            3,
            2,
            WriteType::Simple,
            None,
        );
        let expected = vec![
            0x0, 0x0, 0x11, 0x0, // ID
            0x0, 0x5, // len msg
            0x45, 0x72, 0x72, 0x6F, 0x72, // msg
            0x0, 0x3, // cl
            0x0, 0x0, 0x0, 0x3, // received
            0x0, 0x0, 0x0, 0x2, // blockfor
            0x0, 0x6, 0x53, 0x49, 0x4D, 0x50, 0x4C, 0x45, // writeType
        ];
        assert_eq!(error.as_bytes(), expected);

        let error = Error::WriteTimeout(
            "Error".to_string(),
            Consistency::Three,
            3,
            2,
            WriteType::Cas,
            Some(2),
        );
        let expected = vec![
            0x0, 0x0, 0x11, 0x0, // ID
            0x0, 0x5, // len msg
            0x45, 0x72, 0x72, 0x6F, 0x72, // msg
            0x0, 0x3, // cl
            0x0, 0x0, 0x0, 0x3, // received
            0x0, 0x0, 0x0, 0x2, // blockfor
            0x0, 0x3, 0x43, 0x41, 0x53, // writeType
            0x0, 0x2, // contentions
        ];
        assert_eq!(error.as_bytes(), expected);

        let error = Error::ReadTimeout("Error".to_string(), Consistency::Three, 3, 2, true);
        let expected = vec![
            0x0, 0x0, 0x12, 0x0, // ID
            0x0, 0x5, // len msg
            0x45, 0x72, 0x72, 0x6F, 0x72, // msg
            0x0, 0x3, // cl
            0x0, 0x0, 0x0, 0x3, // received
            0x0, 0x0, 0x0, 0x2, // blockfor
            0x1, // data_present
        ];
        assert_eq!(error.as_bytes(), expected);

        let error = Error::ReadFailure(
            "Error".to_string(),
            Consistency::Three,
            3,
            2,
            ReasonMap::from([(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0x00)]),
            true,
        );
        let expected = vec![
            0x0, 0x0, 0x13, 0x0, // ID
            0x0, 0x5, // len msg
            0x45, 0x72, 0x72, 0x6F, 0x72, // msg
            0x0, 0x3, // cl
            0x0, 0x0, 0x0, 0x3, // received
            0x0, 0x0, 0x0, 0x2, // blockfor
            0x0, 0x0, 0x0, 0x1, // reasonmap len
            0x4, 0x7F, 0x0, 0x0, 0x1, 0x0, 0x0, // endpoint, failurecode
            0x1, // data_present
        ];
        assert_eq!(error.as_bytes(), expected);

        let error = Error::FunctionFailure(
            "Error".to_string(),
            "keyspace".to_string(),
            "function".to_string(),
            vec!["arg1".to_string(), "arg2".to_string()],
        );
        let expected = vec![
            0x0, 0x0, 0x14, 0x0, // ID
            0x0, 0x5, // len msg
            0x45, 0x72, 0x72, 0x6F, 0x72, // msg
            0x0, 0x8, // len keyspace
            0x6B, 0x65, 0x79, 0x73, 0x70, 0x61, 0x63, 0x65, // keyspace
            0x0, 0x8, // len function
            0x66, 0x75, 0x6E, 0x63, 0x74, 0x69, 0x6F, 0x6E, // function
            0x0, 0x2, // arg_types len
            0x0, 0x4, // len arg1
            0x61, 0x72, 0x67, 0x31, // arg1
            0x0, 0x4, // len arg2
            0x61, 0x72, 0x67, 0x32, // arg2
        ];
        assert_eq!(error.as_bytes(), expected);

        let error = Error::WriteFailure(
            "Error".to_string(),
            Consistency::Three,
            3,
            2,
            ReasonMap::from([(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0x00)]),
            WriteType::Simple,
        );
        let expected = vec![
            0x0, 0x0, 0x15, 0x0, // ID
            0x0, 0x5, // len msg
            0x45, 0x72, 0x72, 0x6F, 0x72, // msg
            0x0, 0x3, // cl
            0x0, 0x0, 0x0, 0x3, // received
            0x0, 0x0, 0x0, 0x2, // blockfor
            0x0, 0x0, 0x0, 0x1, // reasonmap len
            0x4, 0x7F, 0x0, 0x0, 0x1, 0x0, 0x0, // endpoint, failurecode
            0x0, 0x6, 0x53, 0x49, 0x4D, 0x50, 0x4C, 0x45, // writeType
        ];
        assert_eq!(error.as_bytes(), expected);

        let error = Error::CASWriteUnknown("Error".to_string(), Consistency::Three, 3, 2);
        let expected = vec![
            0x0, 0x0, 0x17, 0x0, // ID
            0x0, 0x5, // len msg
            0x45, 0x72, 0x72, 0x6F, 0x72, // msg
            0x0, 0x3, // cl
            0x0, 0x0, 0x0, 0x3, // received
            0x0, 0x0, 0x0, 0x2, // blockfor
        ];
        assert_eq!(error.as_bytes(), expected);

        let error =
            Error::AlreadyExists("Error".to_string(), "ks".to_string(), "table".to_string());
        let expected = vec![
            0x0, 0x0, 0x24, 0x0, // ID
            0x0, 0x5, // len msg
            0x45, 0x72, 0x72, 0x6F, 0x72, // msg
            0x0, 0x2, // len ks
            0x6B, 0x73, // ks
            0x0, 0x5, // len table
            0x74, 0x61, 0x62, 0x6C, 0x65, // table
        ];
        assert_eq!(error.as_bytes(), expected);

        let error = Error::Unprepared("Error".to_string(), vec![0x1, 0x2]);
        let expected = vec![
            0x0, 0x0, 0x25, 0x0, // ID
            0x0, 0x5, // len msg
            0x45, 0x72, 0x72, 0x6F, 0x72, // msg
            0x0, 0x2, // len ids
            0x1, 0x2, // ids
        ];
        assert_eq!(error.as_bytes(), expected);
    }

    #[test]
    fn test_2_deserializar() {
        let bytes = vec![
            0x0, 0x0, 0x0, 0x0, // ID
            0x0, 0x5, // len msg
            0x45, 0x72, 0x72, 0x6F, 0x72, // msg
        ];
        let error = Error::try_from(bytes);
        assert!(error.is_ok());
        assert!(matches!(error, Ok(Error::ServerError(msg)) if msg == "Error"));

        let bytes = vec![
            0x0, 0x0, 0x10, 0x0, // ID
            0x0, 0x5, // len msg
            0x45, 0x72, 0x72, 0x6F, 0x72, // msg
            0x0, 0x3, // cl
            0x0, 0x0, 0x0, 0x3, // required
            0x0, 0x0, 0x0, 0x2, // alive
        ];
        let error = Error::try_from(bytes);
        assert!(error.is_ok());
        assert!(
            matches!(error, Ok(Error::UnavailableException(msg, cl, required, alive)) if msg == "Error" && matches!(cl, Consistency::Three) && required == 3 && alive == 2)
        );

        let bytes = vec![
            0x0, 0x0, 0x11, 0x0, // ID
            0x0, 0x5, // len msg
            0x45, 0x72, 0x72, 0x6F, 0x72, // msg
            0x0, 0x3, // cl
            0x0, 0x0, 0x0, 0x3, // received
            0x0, 0x0, 0x0, 0x2, // blockfor
            0x0, 0x6, 0x53, 0x49, 0x4D, 0x50, 0x4C, 0x45, // writeType
        ];
        let error = Error::try_from(bytes);
        assert!(error.is_ok());
        assert!(
            matches!(error, Ok(Error::WriteTimeout(msg, cl, received, blockfor, write_type, None)) if msg == "Error" && matches!(cl, Consistency::Three) && received == 3 && blockfor == 2 && matches!(write_type, WriteType::Simple))
        );

        let bytes = vec![
            0x0, 0x0, 0x11, 0x0, // ID
            0x0, 0x5, // len msg
            0x45, 0x72, 0x72, 0x6F, 0x72, // msg
            0x0, 0x3, // cl
            0x0, 0x0, 0x0, 0x3, // received
            0x0, 0x0, 0x0, 0x2, // blockfor
            0x0, 0x3, 0x43, 0x41, 0x53, // writeType
            0x0, 0x2, // contentions
        ];
        let error = Error::try_from(bytes);
        assert!(error.is_ok());
        assert!(
            matches!(error, Ok(Error::WriteTimeout(msg, cl, received, blockfor, write_type, contentions)) if msg == "Error" && matches!(cl, Consistency::Three) && received == 3 && blockfor == 2 && matches!(write_type, WriteType::Cas) && contentions == Some(2))
        );

        let bytes = vec![
            0x0, 0x0, 0x12, 0x0, // ID
            0x0, 0x5, // len msg
            0x45, 0x72, 0x72, 0x6F, 0x72, // msg
            0x0, 0x3, // cl
            0x0, 0x0, 0x0, 0x3, // received
            0x0, 0x0, 0x0, 0x2, // blockfor
            0x1, // data_present
        ];
        let error = Error::try_from(bytes);
        assert!(error.is_ok());
        assert!(
            matches!(error, Ok(Error::ReadTimeout(msg, cl, received, blockfor, data_present)) if msg == "Error" && matches!(cl, Consistency::Three) && received == 3 && blockfor == 2 && data_present)
        );

        let bytes = vec![
            0x0, 0x0, 0x13, 0x0, // ID
            0x0, 0x5, // len msg
            0x45, 0x72, 0x72, 0x6F, 0x72, // msg
            0x0, 0x3, // cl
            0x0, 0x0, 0x0, 0x3, // received
            0x0, 0x0, 0x0, 0x2, // blockfor
            0x0, 0x0, 0x0, 0x1, // reasonmap len
            0x4, 0x7F, 0x0, 0x0, 0x1, 0x0, 0x0, // endpoint, failurecode
            0x1, // data_present
        ];
        let error = Error::try_from(bytes);
        assert!(error.is_ok());
        assert!(
            matches!(error, Ok(Error::ReadFailure(msg, cl, received, blockfor, reasonmap, data_present)) if msg == "Error" && matches!(cl, Consistency::Three) && received == 3 && blockfor == 2 && reasonmap == ReasonMap::from([(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0x00)]) && data_present)
        );

        let bytes = vec![
            0x0, 0x0, 0x14, 0x0, // ID
            0x0, 0x5, // len msg
            0x45, 0x72, 0x72, 0x6F, 0x72, // msg
            0x0, 0x8, // len keyspace
            0x6B, 0x65, 0x79, 0x73, 0x70, 0x61, 0x63, 0x65, // keyspace
            0x0, 0x8, // len function
            0x66, 0x75, 0x6E, 0x63, 0x74, 0x69, 0x6F, 0x6E, // function
            0x0, 0x2, // arg_types len
            0x0, 0x4, // len arg1
            0x61, 0x72, 0x67, 0x31, // arg1
            0x0, 0x4, // len arg2
            0x61, 0x72, 0x67, 0x32, // arg2
        ];
        let error = Error::try_from(bytes);
        assert!(error.is_ok());
        assert!(
            matches!(error, Ok(Error::FunctionFailure(msg, keyspace, function, arg_types)) if msg == "Error" && keyspace == "keyspace" && function == "function" && arg_types == vec!["arg1".to_string(), "arg2".to_string()])
        );

        let bytes = vec![
            0x0, 0x0, 0x15, 0x0, // ID
            0x0, 0x5, // len msg
            0x45, 0x72, 0x72, 0x6F, 0x72, // msg
            0x0, 0x3, // cl
            0x0, 0x0, 0x0, 0x3, // received
            0x0, 0x0, 0x0, 0x2, // blockfor
            0x0, 0x0, 0x0, 0x1, // reasonmap len
            0x4, 0x7F, 0x0, 0x0, 0x1, 0x0, 0x0, // endpoint, failurecode
            0x0, 0x6, 0x53, 0x49, 0x4D, 0x50, 0x4C, 0x45, // writeType
        ];
        let error = Error::try_from(bytes);
        assert!(error.is_ok());
        assert!(
            matches!(error, Ok(Error::WriteFailure(msg, cl, received, blockfor, reasonmap, write_type)) if msg == "Error" && matches!(cl, Consistency::Three) && received == 3 && blockfor == 2 && reasonmap == ReasonMap::from([(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0x00)]) && matches!(write_type, WriteType::Simple))
        );

        let bytes = vec![
            0x0, 0x0, 0x17, 0x0, // ID
            0x0, 0x5, // len msg
            0x45, 0x72, 0x72, 0x6F, 0x72, // msg
            0x0, 0x3, // cl
            0x0, 0x0, 0x0, 0x3, // received
            0x0, 0x0, 0x0, 0x2, // blockfor
        ];
        let error = Error::try_from(bytes);
        assert!(error.is_ok());
        assert!(
            matches!(error, Ok(Error::CASWriteUnknown(msg, cl, received, blockfor)) if msg == "Error" && matches!(cl, Consistency::Three) && received == 3 && blockfor == 2)
        );

        let bytes = vec![
            0x0, 0x0, 0x24, 0x0, // ID
            0x0, 0x5, // len msg
            0x45, 0x72, 0x72, 0x6F, 0x72, // msg
            0x0, 0x2, // len ks
            0x6B, 0x73, // ks
            0x0, 0x5, // len table
            0x74, 0x61, 0x62, 0x6C, 0x65, // table
        ];
        let error = Error::try_from(bytes);
        assert!(error.is_ok());
        assert!(
            matches!(error, Ok(Error::AlreadyExists(msg, ks, table)) if msg == "Error" && ks == "ks" && table == "table")
        );

        let bytes = vec![
            0x0, 0x0, 0x25, 0x0, // ID
            0x0, 0x5, // len msg
            0x45, 0x72, 0x72, 0x6F, 0x72, // msg
            0x0, 0x2, // len ids
            0x1, 0x2, // ids
        ];
        let error = Error::try_from(bytes);
        assert!(error.is_ok());
        assert!(
            matches!(error, Ok(Error::Unprepared(msg, ids)) if msg == "Error" && ids == vec![0x1, 0x2])
        );
    }

    #[test]
    fn test_3_deserializar_error() {
        let bytes = vec![0x0, 0xFF, 0x0, 0xFF];
        let error = Error::try_from(bytes);
        assert!(error.is_err());
        assert!(matches!(error, Err(Error::Invalid(_))));
    }
}
