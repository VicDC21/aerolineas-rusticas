//! Módulo para mensajes de errores.

use std::{
    backtrace::Backtrace,
    collections::HashMap,
    fmt::{Display, Formatter, Result as FmtResult},
    net::IpAddr,
    result::Result as StdResult,
};

use crate::cassandra::aliases::types::{Byte, Int, Short};
use crate::cassandra::{notations::consistency::Consistency, traits::Byteable};

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
    /// * `<writeType>` es un [String] que representa el tipo de escritura que se estaba intentando realizar. El valor puede ser:
    ///     * "SIMPLE": La escritura no fue de tipo batch ni de tipo counter.
    ///     * "BATCH": La escritura fue de tipo batch (logged). Esto signifca que el log del batch fue escrito correctamente, caso contrario, se debería haber enviado el tipo "BATCH_LOG".
    ///     * "UNLOGGED_BATCH": La escritura fue de tipo batch (unlogged). No hubo intento de escritura en el log del batch.
    ///     * "COUNTER": La escritura fue de tipo counter (batch o no).
    ///     * "BATCH_LOG": El timeout ocurrió durante la escritura en el log del batch cuando una escritura de batch (logged) fue pedida.
    ///     * "CAS": El timeout ocurrió durante el Compare And Set write/update (escritura/actualización).
    ///     * "VIEW": El timeout ocurrió durante una escritura que involucra una actualización de VIEW (vista) y falló en adquirir el lock de vista local (MV) para la clave dentro del timeout.
    ///     * "CDC": El timeout ocurrió cuando la cantidad total de espacio en disco (en MB) que se puede utilizar para almacenar los logs de CDC (Change Data Capture) fue excedida cuando se intentaba escribir en dicho logs.
    /// * `<contentions>` es un número ([Short]) que representa la cantidad de contenciones ocurridas durante la operación CAS. Este campo solo se presenta cuando el writeType es "CAS".
    ///
    /// TODO: _Quizás meter writeType en un enum._
    WriteTimeout(String, Consistency, Int, Int, String, Option<Short>),

    /// Timeout exception durante un request de lectura.
    ///
    /// El resto del mensaje es `<cl><received><blockfor><data_present>`, donde:
    /// * `<cl>` es el nivel de [Consistency] de la query que lanzó esta excepción.
    /// * `<received>` es un número ([Int]) que representa la cantidad de nodos que han respondido a la request.
    /// * `<blockfor>` es un número ([Int]) que representa la cantidad de réplicas cuya respuesta es necesaria para cumplir `cl`. Notar que es posible tener `<received> >= <blockfor>` si <data_present> es false. También en el caso (improbable) donde cl se cumple pero el nodo coordinador sufre un timeout mientras esperaba por la confirmación de un read-repair.
    /// * `<data_present>` es un [Byte] (representa un booleano: 0 es false, distinto de 0 es true) que indica si el nodo al que se le hizo el pedido de la data respondió o no.
    ReadTimeout(String, Consistency, Int, Int, Byte),

    /// Una excepción de lectura que no fue ocasionada por un timeout.
    ///
    /// El resto del mensaje es `<cl><received><blockfor><reasonmap><data_present>`, donde:
    /// * `<cl>` es el nivel de [Consistency] de la query que lanzó esta excepción.
    /// * `<received>` es un número ([Int]) que representa la cantidad de nodos que han respondido a la request.
    /// * `<blockfor>` es un número ([Int]) que representa la cantidad de réplicas cuya respuesta es necesaria para cumplir `<cl>`.
    /// * `<reasonmap>` es un "mapa" de endpoints a códigos de razón de error. Esto mapea los endpoints de los nodos réplica que fallaron al ejecutar la request a un código representando la razón del error. La forma del mapa es empezando con un [Int] n seguido por n pares de endpoint,failurecode donde endpoint es un [IpAddr] y failurecode es un [Short].
    /// * `<data_present>` es un [Byte] (representa un booleano: 0 es false, distinto de 0 es true) que indica si el nodo al que se le hizo el pedido de la data respondió o no.
    ReadFailure(String, Consistency, Int, Int, HashMap<IpAddr, Short>, Byte),

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
    /// * `<writeType>` es un [String] que representa el tipo de escritura que se estaba intentando realizar. El valor puede ser:
    ///     * "SIMPLE": La escritura no fue de tipo batch ni de tipo counter.
    ///     * "BATCH": La escritura fue de tipo batch (logged). Esto signifca que el log del batch fue escrito correctamente, caso contrario, se debería haber enviado el tipo "BATCH_LOG".
    ///     * "UNLOGGED_BATCH": La escritura fue de tipo batch (unlogged). No hubo intento de escritura en el log del batch.
    ///     * "COUNTER": La escritura fue de tipo counter (batch o no).
    ///     * "BATCH_LOG": El timeout ocurrió durante la escritura en el log del batch cuando una escritura de batch (logged) fue pedida.
    ///     * "CAS": El timeout ocurrió durante el _Compare And Set write/update_ (escritura/actualización).
    ///     * "VIEW": El timeout ocurrió durante una escritura que involucra una actualización de VIEW (vista) y falló en adquirir el lock de vista local (MV) para la clave dentro del timeout.
    ///     * "CDC": El timeout ocurrió cuando la cantidad total de espacio en disco (en MB) que se puede utilizar para almacenar los logs de CDC (Change Data Capture) fue excedida cuando se intentaba escribir en dicho logs.
    ///
    /// TODO: _Quizás meter writeType en un enum._
    WriteFailure(
        String,
        Consistency,
        Int,
        Int,
        HashMap<IpAddr, Short>,
        String,
    ),

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

impl Error {
    fn parse_string_to_bytes(string: &str) -> Vec<Byte> {
        let string_bytes = string.as_bytes();
        // litle endian para que los dos bytes menos significativos (los únicos que nos interesa
        // para un [Short]) estén al principio
        let bytes_len = string_bytes.len().to_le_bytes();
        let mut bytes_vec: Vec<Byte> = vec![
            bytes_len[1],
            bytes_len[0], // Longitud del string
        ];
        bytes_vec.extend_from_slice(string_bytes);
        bytes_vec
    }

    fn parse_hashmap_to_bytes(hashmap: &HashMap<IpAddr, Short>) -> Vec<Byte> {
        let mut bytes_vec: Vec<Byte> = vec![];
        let hashmap_len = hashmap.len().to_le_bytes();
        bytes_vec.extend_from_slice(&[
            hashmap_len[3],
            hashmap_len[2],
            hashmap_len[1],
            hashmap_len[0],
        ]);
        for (ip, code) in hashmap {
            let ip_bytes = match ip {
                IpAddr::V4(ipv4) => ipv4.octets().to_vec(),
                IpAddr::V6(ipv6) => ipv6.octets().to_vec(),
            };
            let ip_len = ip_bytes.len().to_le_bytes();
            bytes_vec.extend_from_slice(&[ip_len[1], ip_len[0]]);
            bytes_vec.extend(ip_bytes);
            bytes_vec.extend(code.to_be_bytes());
        }
        bytes_vec
    }

    fn parse_bytes_to_string(bytes_vec: &[Byte]) -> StdResult<String, Error> {
        if bytes_vec.len() < 2 {
            return Err(Error::SyntaxError(
                "Se esperaban 2 bytes que indiquen el tamaño del string a formar".to_string(),
            ));
        }
        let string_len = Short::from_le_bytes([bytes_vec[1], bytes_vec[0]]) as usize;
        match String::from_utf8(bytes_vec[2..string_len + 2].to_vec()) {
            Ok(string) => Ok(string),
            Err(_) => Err(Error::Invalid(
                "El cuerpo del string no se pudo parsear".to_string(),
            )),
        }
    }

    fn parse_bytes_to_hashmap(
        bytes_vec: &[Byte],
        i: &mut usize,
    ) -> StdResult<HashMap<IpAddr, Short>, Error> {
        if bytes_vec.len() < 4 {
            return Err(Error::SyntaxError(
                "Se esperaban 4 bytes que indiquen el tamaño del reasonmap a formar".to_string(),
            ));
        }
        let hashmap_len = Int::from_le_bytes([
            bytes_vec[*i + 3],
            bytes_vec[*i + 2],
            bytes_vec[*i + 1],
            bytes_vec[*i],
        ]) as usize;
        *i += 4;

        let mut reasonmap: HashMap<IpAddr, Short> = HashMap::new();
        for _ in 0..hashmap_len {
            let ip_len = Short::from_le_bytes([bytes_vec[*i + 1], bytes_vec[*i]]);
            *i += 2;
            let ip = match ip_len {
                4 => IpAddr::V4(std::net::Ipv4Addr::new(
                    bytes_vec[*i],
                    bytes_vec[*i + 1],
                    bytes_vec[*i + 2],
                    bytes_vec[*i + 3],
                )),
                16 => IpAddr::V6(std::net::Ipv6Addr::new(
                    Short::from_be_bytes([bytes_vec[*i], bytes_vec[*i + 1]]),
                    Short::from_be_bytes([bytes_vec[*i + 2], bytes_vec[*i + 3]]),
                    Short::from_be_bytes([bytes_vec[*i + 4], bytes_vec[*i + 5]]),
                    Short::from_be_bytes([bytes_vec[*i + 6], bytes_vec[*i + 7]]),
                    Short::from_be_bytes([bytes_vec[*i + 8], bytes_vec[*i + 9]]),
                    Short::from_be_bytes([bytes_vec[*i + 10], bytes_vec[*i + 11]]),
                    Short::from_be_bytes([bytes_vec[*i + 12], bytes_vec[*i + 13]]),
                    Short::from_be_bytes([bytes_vec[*i + 14], bytes_vec[*i + 15]]),
                )),
                _ => {
                    return Err(Error::Invalid(
                        "La longitud de la dirección IP no es válida".to_string(),
                    ))
                }
            };
            *i += ip_len as usize;
            let code = Short::from_be_bytes([bytes_vec[*i + 1], bytes_vec[*i]]);
            *i += 2;
            reasonmap.insert(ip, code);
        }
        Ok(reasonmap)
    }
}

impl Byteable for Error {
    fn as_bytes(&self) -> Vec<Byte> {
        match self {
            Self::ServerError(msg) => {
                let mut bytes_vec: Vec<Byte> = vec![
                    0x0, 0x0, 0x0, 0x0, // ID
                ];
                bytes_vec.extend(Error::parse_string_to_bytes(msg));
                bytes_vec
            }
            Self::ProtocolError(msg) => {
                let mut bytes_vec: Vec<Byte> = vec![
                    0x0, 0x0, 0x0, 0xA, // ID
                ];
                bytes_vec.extend(Error::parse_string_to_bytes(msg));
                bytes_vec
            }
            Self::AuthenticationError(msg) => {
                let mut bytes_vec: Vec<Byte> = vec![
                    0x0, 0x0, 0x1, 0x0, // ID
                ];
                bytes_vec.extend(Error::parse_string_to_bytes(msg));
                bytes_vec
            }
            Self::UnavailableException(msg, cl, required, alive) => {
                let mut bytes_vec: Vec<Byte> = vec![
                    0x0, 0x0, 0x10, 0x0, // ID
                ];
                bytes_vec.extend(Error::parse_string_to_bytes(msg));
                bytes_vec.extend(cl.as_bytes());
                bytes_vec.extend(required.to_be_bytes());
                bytes_vec.extend(alive.to_be_bytes());
                bytes_vec
            }
            Self::Overloaded(msg) => {
                let mut bytes_vec: Vec<Byte> = vec![
                    0x0, 0x0, 0x10, 0x1, // ID
                ];
                bytes_vec.extend(Error::parse_string_to_bytes(msg));
                bytes_vec
            }
            Self::IsBootstrapping(msg) => {
                let mut bytes_vec: Vec<Byte> = vec![
                    0x0, 0x0, 0x10, 0x2, // ID
                ];
                bytes_vec.extend(Error::parse_string_to_bytes(msg));
                bytes_vec
            }
            Self::TruncateError(msg) => {
                let mut bytes_vec: Vec<Byte> = vec![
                    0x0, 0x0, 0x10, 0x3, // ID
                ];
                bytes_vec.extend(Error::parse_string_to_bytes(msg));
                bytes_vec
            }
            Self::WriteTimeout(msg, cl, received, blockfor, write_type, contentions) => {
                let mut bytes_vec: Vec<Byte> = vec![
                    0x0, 0x0, 0x11, 0x0, // ID
                ];
                bytes_vec.extend(Error::parse_string_to_bytes(msg));
                bytes_vec.extend(cl.as_bytes());
                bytes_vec.extend(received.to_be_bytes());
                bytes_vec.extend(blockfor.to_be_bytes());
                bytes_vec.extend(Error::parse_string_to_bytes(write_type));
                if write_type == "CAS" {
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
                bytes_vec.extend(Error::parse_string_to_bytes(msg));
                bytes_vec.extend(cl.as_bytes());
                bytes_vec.extend(received.to_be_bytes());
                bytes_vec.extend(blockfor.to_be_bytes());
                bytes_vec.push(*data_present);
                bytes_vec
            }
            Self::ReadFailure(msg, cl, received, blockfor, reasonmap, data_present) => {
                let mut bytes_vec: Vec<Byte> = vec![
                    0x0, 0x0, 0x13, 0x0, // ID
                ];
                bytes_vec.extend(Error::parse_string_to_bytes(msg));
                bytes_vec.extend(cl.as_bytes());
                bytes_vec.extend(received.to_be_bytes());
                bytes_vec.extend(blockfor.to_be_bytes());
                bytes_vec.extend(Error::parse_hashmap_to_bytes(reasonmap));
                bytes_vec.push(*data_present);
                bytes_vec
            }
            Self::FunctionFailure(msg, keyspace, function, arg_types) => {
                let mut bytes_vec: Vec<Byte> = vec![
                    0x0, 0x0, 0x14, 0x0, // ID
                ];
                bytes_vec.extend(Error::parse_string_to_bytes(msg));
                bytes_vec.extend(Error::parse_string_to_bytes(keyspace));
                bytes_vec.extend(Error::parse_string_to_bytes(function));

                let list_len = arg_types.len().to_le_bytes();
                bytes_vec.extend_from_slice(&[list_len[1], list_len[0]]);
                for string in arg_types {
                    bytes_vec.extend(Error::parse_string_to_bytes(string));
                }

                bytes_vec
            }
            Self::WriteFailure(msg, cl, received, blockfor, reasonmap, string) => {
                let mut bytes_vec: Vec<Byte> = vec![
                    0x0, 0x0, 0x15, 0x0, // ID
                ];
                bytes_vec.extend(Error::parse_string_to_bytes(msg));
                bytes_vec.extend(cl.as_bytes());
                bytes_vec.extend(received.to_be_bytes());
                bytes_vec.extend(blockfor.to_be_bytes());
                bytes_vec.extend(Error::parse_hashmap_to_bytes(reasonmap));
                bytes_vec.extend(Error::parse_string_to_bytes(string));
                bytes_vec
            }
            Self::CDCWriteFailure(msg) => {
                let mut bytes_vec: Vec<Byte> = vec![
                    0x0, 0x0, 0x16, 0x0, // ID
                ];
                bytes_vec.extend(Error::parse_string_to_bytes(msg));
                bytes_vec
            }
            Self::CASWriteUnknown(msg, cl, received, blockfor) => {
                let mut bytes_vec: Vec<Byte> = vec![
                    0x0, 0x0, 0x17, 0x0, // ID
                ];
                bytes_vec.extend(Error::parse_string_to_bytes(msg));
                bytes_vec.extend(cl.as_bytes());
                bytes_vec.extend(received.to_be_bytes());
                bytes_vec.extend(blockfor.to_be_bytes());
                bytes_vec
            }
            Self::SyntaxError(msg) => {
                let mut bytes_vec: Vec<Byte> = vec![
                    0x0, 0x0, 0x20, 0x0, // ID
                ];
                bytes_vec.extend(Error::parse_string_to_bytes(msg));
                bytes_vec
            }
            Self::Unauthorized(msg) => {
                let mut bytes_vec: Vec<Byte> = vec![
                    0x0, 0x0, 0x21, 0x0, // ID
                ];
                bytes_vec.extend(Error::parse_string_to_bytes(msg));
                bytes_vec
            }
            Self::Invalid(msg) => {
                let mut bytes_vec: Vec<Byte> = vec![
                    0x0, 0x0, 0x22, 0x0, // ID
                ];
                bytes_vec.extend(Error::parse_string_to_bytes(msg));
                bytes_vec
            }
            Self::ConfigError(msg) => {
                let mut bytes_vec: Vec<Byte> = vec![
                    0x0, 0x0, 0x23, 0x0, // ID
                ];
                bytes_vec.extend(Error::parse_string_to_bytes(msg));
                bytes_vec
            }
            Self::AlreadyExists(msg, ks, table) => {
                let mut bytes_vec: Vec<Byte> = vec![
                    0x0, 0x0, 0x24, 0x0, // ID
                ];
                bytes_vec.extend(Error::parse_string_to_bytes(msg));
                bytes_vec.extend(Error::parse_string_to_bytes(ks));
                bytes_vec.extend(Error::parse_string_to_bytes(table));
                bytes_vec
            }
            Self::Unprepared(msg, ids) => {
                let mut bytes_vec: Vec<Byte> = vec![
                    0x0, 0x0, 0x25, 0x0, // ID
                ];
                bytes_vec.extend(Error::parse_string_to_bytes(msg));
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
    fn try_from(bytes_vec: Vec<Byte>) -> StdResult<Self, Self> {
        let mut i = 4;
        match bytes_vec[..i] {
            [0x0, 0x0, 0x0, 0x0] => match Error::parse_bytes_to_string(&bytes_vec[i..]) {
                Ok(msg) => Ok(Error::ServerError(msg)),
                Err(err) => Err(err),
            },
            [0x0, 0x0, 0x0, 0xA] => match Error::parse_bytes_to_string(&bytes_vec[i..]) {
                Ok(msg) => Ok(Error::ProtocolError(msg)),
                Err(err) => Err(err),
            },
            [0x0, 0x0, 0x1, 0x0] => match Error::parse_bytes_to_string(&bytes_vec[i..]) {
                Ok(msg) => Ok(Error::AuthenticationError(msg)),
                Err(err) => Err(err),
            },
            [0x0, 0x0, 0x10, 0x0] => match Error::parse_bytes_to_string(&bytes_vec[i..]) {
                Ok(msg) => {
                    i += msg.len() + 2;
                    let cl = Consistency::try_from(bytes_vec[i..].to_vec())?;
                    let required = Int::from_be_bytes([
                        bytes_vec[i + 2],
                        bytes_vec[i + 3],
                        bytes_vec[i + 4],
                        bytes_vec[i + 5],
                    ]);
                    i += 5;
                    let alive = Int::from_be_bytes([
                        bytes_vec[i + 1],
                        bytes_vec[i + 2],
                        bytes_vec[i + 3],
                        bytes_vec[i + 4],
                    ]);
                    Ok(Error::UnavailableException(msg, cl, required, alive))
                }
                Err(err) => Err(err),
            },
            [0x0, 0x0, 0x10, 0x1] => match Error::parse_bytes_to_string(&bytes_vec[i..]) {
                Ok(msg) => Ok(Error::Overloaded(msg)),
                Err(err) => Err(err),
            },
            [0x0, 0x0, 0x10, 0x2] => match Error::parse_bytes_to_string(&bytes_vec[i..]) {
                Ok(msg) => Ok(Error::IsBootstrapping(msg)),
                Err(err) => Err(err),
            },
            [0x0, 0x0, 0x10, 0x3] => match Error::parse_bytes_to_string(&bytes_vec[i..]) {
                Ok(msg) => Ok(Error::TruncateError(msg)),
                Err(err) => Err(err),
            },
            [0x0, 0x0, 0x11, 0x0] => match Error::parse_bytes_to_string(&bytes_vec[i..]) {
                Ok(msg) => {
                    i += msg.len() + 2;
                    let cl = Consistency::try_from(bytes_vec[i..].to_vec())?;
                    let received = Int::from_be_bytes([
                        bytes_vec[i + 2],
                        bytes_vec[i + 3],
                        bytes_vec[i + 4],
                        bytes_vec[i + 5],
                    ]);
                    i += 5;
                    let blockfor = Int::from_be_bytes([
                        bytes_vec[i + 1],
                        bytes_vec[i + 2],
                        bytes_vec[i + 3],
                        bytes_vec[i + 4],
                    ]);
                    i += 5;
                    let write_type = Error::parse_bytes_to_string(&bytes_vec[i..])?;
                    let contentions = if write_type == "CAS" {
                        if i + write_type.len() + 3 >= bytes_vec.len() {
                            return Err(Error::SyntaxError("Se esperaban 3 bytes más para el campo <contentions> del error WriteTimeout".to_string()));
                        }
                        Some(Short::from_be_bytes([
                            bytes_vec[i + write_type.len() + 2],
                            bytes_vec[i + write_type.len() + 3],
                        ]))
                    } else {
                        None
                    };
                    Ok(Error::WriteTimeout(
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
            [0x0, 0x0, 0x12, 0x0] => match Error::parse_bytes_to_string(&bytes_vec[i..]) {
                Ok(msg) => {
                    i += msg.len() + 2;
                    let cl = Consistency::try_from(bytes_vec[i..].to_vec())?;
                    let received = Int::from_be_bytes([
                        bytes_vec[i + 2],
                        bytes_vec[i + 3],
                        bytes_vec[i + 4],
                        bytes_vec[i + 5],
                    ]);
                    i += 5;
                    let blockfor = Int::from_be_bytes([
                        bytes_vec[i + 1],
                        bytes_vec[i + 2],
                        bytes_vec[i + 3],
                        bytes_vec[i + 4],
                    ]);
                    i += 5;
                    let data_present = bytes_vec[i];
                    Ok(Error::ReadTimeout(
                        msg,
                        cl,
                        received,
                        blockfor,
                        data_present,
                    ))
                }
                Err(err) => Err(err),
            },
            [0x0, 0x0, 0x13, 0x0] => match Error::parse_bytes_to_string(&bytes_vec[i..]) {
                Ok(msg) => {
                    i += msg.len() + 2;
                    let cl = Consistency::try_from(bytes_vec[i..].to_vec())?;
                    let received = Int::from_be_bytes([
                        bytes_vec[i + 2],
                        bytes_vec[i + 3],
                        bytes_vec[i + 4],
                        bytes_vec[i + 5],
                    ]);
                    i += 5;
                    let blockfor = Int::from_be_bytes([
                        bytes_vec[i + 1],
                        bytes_vec[i + 2],
                        bytes_vec[i + 3],
                        bytes_vec[i + 4],
                    ]);
                    i += 5;
                    let reasonmap = Error::parse_bytes_to_hashmap(&bytes_vec, &mut i)?;
                    let data_present = bytes_vec[i];
                    Ok(Error::ReadFailure(
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
            [0x0, 0x0, 0x14, 0x0] => match Error::parse_bytes_to_string(&bytes_vec[i..]) {
                Ok(msg) => {
                    i += msg.len() + 2;
                    let keyspace = match Error::parse_bytes_to_string(&bytes_vec[i..]) {
                        Ok(string) => string,
                        Err(err) => return Err(err),
                    };
                    i += keyspace.len() + 2;
                    let function = match Error::parse_bytes_to_string(&bytes_vec[i..]) {
                        Ok(string) => string,
                        Err(err) => return Err(err),
                    };
                    i += function.len() + 2;
                    let arg_types_len = Short::from_le_bytes([bytes_vec[i + 1], bytes_vec[i]]);
                    i += 2;
                    let mut arg_types: Vec<String> = vec![];
                    for _ in 0..arg_types_len {
                        match Error::parse_bytes_to_string(&bytes_vec[i..]) {
                            Ok(string) => {
                                i += string.len() + 2;
                                arg_types.push(string)
                            }
                            Err(err) => return Err(err),
                        }
                    }
                    Ok(Error::FunctionFailure(msg, keyspace, function, arg_types))
                }
                Err(err) => Err(err),
            },
            [0x0, 0x0, 0x15, 0x0] => match Error::parse_bytes_to_string(&bytes_vec[i..]) {
                Ok(msg) => {
                    i += msg.len() + 2;
                    let cl = Consistency::try_from(bytes_vec[i..].to_vec())?;
                    let received = Int::from_be_bytes([
                        bytes_vec[i + 2],
                        bytes_vec[i + 3],
                        bytes_vec[i + 4],
                        bytes_vec[i + 5],
                    ]);
                    i += 5;
                    let blockfor = Int::from_be_bytes([
                        bytes_vec[i + 1],
                        bytes_vec[i + 2],
                        bytes_vec[i + 3],
                        bytes_vec[i + 4],
                    ]);
                    i += 5;
                    let reasonmap = Error::parse_bytes_to_hashmap(&bytes_vec, &mut i)?;
                    let write_type = Error::parse_bytes_to_string(&bytes_vec[i..])?;
                    Ok(Error::WriteFailure(
                        msg, cl, received, blockfor, reasonmap, write_type,
                    ))
                }
                Err(err) => Err(err),
            },
            [0x0, 0x0, 0x16, 0x0] => match Error::parse_bytes_to_string(&bytes_vec[i..]) {
                Ok(msg) => Ok(Error::CDCWriteFailure(msg)),
                Err(err) => Err(err),
            },
            [0x0, 0x0, 0x17, 0x0] => match Error::parse_bytes_to_string(&bytes_vec[i..]) {
                Ok(msg) => {
                    i += msg.len() + 2;
                    let cl = Consistency::try_from(bytes_vec[i..].to_vec())?;
                    let received = Int::from_be_bytes([
                        bytes_vec[i + 2],
                        bytes_vec[i + 3],
                        bytes_vec[i + 4],
                        bytes_vec[i + 5],
                    ]);
                    i += 5;
                    let blockfor = Int::from_be_bytes([
                        bytes_vec[i + 1],
                        bytes_vec[i + 2],
                        bytes_vec[i + 3],
                        bytes_vec[i + 4],
                    ]);
                    Ok(Error::CASWriteUnknown(msg, cl, received, blockfor))
                }
                Err(err) => Err(err),
            },
            [0x0, 0x0, 0x20, 0x0] => match Error::parse_bytes_to_string(&bytes_vec[i..]) {
                Ok(msg) => Ok(Error::SyntaxError(msg)),
                Err(err) => Err(err),
            },
            [0x0, 0x0, 0x21, 0x0] => match Error::parse_bytes_to_string(&bytes_vec[i..]) {
                Ok(msg) => Ok(Error::Unauthorized(msg)),
                Err(err) => Err(err),
            },
            [0x0, 0x0, 0x22, 0x0] => match Error::parse_bytes_to_string(&bytes_vec[i..]) {
                Ok(msg) => Ok(Error::Invalid(msg)),
                Err(err) => Err(err),
            },
            [0x0, 0x0, 0x23, 0x0] => match Error::parse_bytes_to_string(&bytes_vec[i..]) {
                Ok(msg) => Ok(Error::ConfigError(msg)),
                Err(err) => Err(err),
            },
            [0x0, 0x0, 0x24, 0x0] => match Error::parse_bytes_to_string(&bytes_vec[i..]) {
                Ok(msg) => {
                    i += msg.len() + 2;
                    let ks = match Error::parse_bytes_to_string(&bytes_vec[i..]) {
                        Ok(string) => string,
                        Err(err) => return Err(err),
                    };
                    i += ks.len() + 2;
                    let table = match Error::parse_bytes_to_string(&bytes_vec[i..]) {
                        Ok(string) => string,
                        Err(err) => return Err(err),
                    };
                    Ok(Error::AlreadyExists(msg, ks, table))
                }
                Err(err) => Err(err),
            },
            [0x0, 0x0, 0x25, 0x0] => match Error::parse_bytes_to_string(&bytes_vec[i..]) {
                Ok(msg) => {
                    i += msg.len() + 2;
                    let ids_len = Short::from_le_bytes([bytes_vec[i + 1], bytes_vec[i]]);
                    i += 2;
                    let mut ids: Vec<Byte> = vec![];
                    for _ in 0..ids_len {
                        ids.push(bytes_vec[i]);
                        i += 1;
                    }
                    Ok(Error::Unprepared(msg, ids))
                }
                Err(err) => Err(err),
            },
            _ => Err(Error::Invalid("El ID del error no es válido".to_string())),
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let backtrace = Backtrace::capture();
        match self {
            Error::ServerError(msg) => write!(f, "{:?}\nServerError: {}\n", backtrace, msg),
            Error::ProtocolError(msg) => write!(f, "{:?}\nProtocolError: {}\n", backtrace, msg),
            Error::AuthenticationError(msg) => {
                write!(f, "{:?}\nAuthenticationError: {}\n", backtrace, msg)
            }
            Error::UnavailableException(msg, _, _, _) => {
                write!(f, "{:?}\nUnavailableException: {}\n", backtrace, msg)
            }
            Error::Overloaded(msg) => write!(f, "{:?}\nOverloaded: {}\n", backtrace, msg),
            Error::IsBootstrapping(msg) => write!(f, "{:?}\nIsBootstrapping: {}\n", backtrace, msg),
            Error::TruncateError(msg) => write!(f, "{:?}\nTruncateError: {}\n", backtrace, msg),
            Error::WriteTimeout(msg, _, _, _, _, _) => {
                write!(f, "{:?}\nWriteTimeout: {}\n", backtrace, msg)
            }
            Error::ReadTimeout(msg, _, _, _, _) => {
                write!(f, "{:?}\nReadTimeout: {}\n", backtrace, msg)
            }
            Error::ReadFailure(msg, _, _, _, _, _) => {
                write!(f, "{:?}\nReadFailure: {}\n", backtrace, msg)
            }
            Error::FunctionFailure(msg, _, _, _) => {
                write!(f, "{:?}\nFunctionFailure: {}\n", backtrace, msg)
            }
            Error::WriteFailure(msg, _, _, _, _, _) => {
                write!(f, "{:?}\nWriteFailure: {}\n", backtrace, msg)
            }
            Error::CDCWriteFailure(msg) => write!(f, "{:?}\nCDCWriteFailure: {}\n", backtrace, msg),
            Error::CASWriteUnknown(msg, _, _, _) => {
                write!(f, "{:?}\nCASWriteUnknown: {}\n", backtrace, msg)
            }
            Error::SyntaxError(msg) => write!(f, "{:?}\nSyntaxError: {}\n", backtrace, msg),
            Error::Unauthorized(msg) => write!(f, "{:?}\nUnauthorized: {}\n", backtrace, msg),
            Error::Invalid(msg) => write!(f, "{:?}\nInvalid: {}\n", backtrace, msg),
            Error::ConfigError(msg) => write!(f, "{:?}\nConfigError: {}\n", backtrace, msg),
            Error::AlreadyExists(msg, _, _) => {
                write!(f, "{:?}\nAlreadyExists: {}\n", backtrace, msg)
            }
            Error::Unprepared(msg, _) => write!(f, "{:?}\nUnprepared: {}\n", backtrace, msg),
        }
    }
}
