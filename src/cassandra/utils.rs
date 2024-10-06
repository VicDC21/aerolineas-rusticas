//! Módulo para objetos de utilidad y funciones auxiliares.

use std::net::IpAddr;

use crate::cassandra::aliases::{
    results::Result,
    types::{Byte, Int, ReasonMap, Short},
};
use crate::cassandra::errors::error::Error;

/// Transforma un [String] a una colección de [Byte]s tal cual como está especificado
/// en el protocolo de Cassandra.
///
/// Más específicamente, el protocolo pide que primero vaya un entero de
/// 2 bytes ([Short](crate::cassandra::aliases::types::Short)), seguido del contenido mismo del
/// [String], en donde cada _byte_ representa un carácter UTF-8.
///
/// ```rust
/// # use aerolineas::cassandra::utils::encode_string_to_bytes;
/// let bytes = encode_string_to_bytes(&"Hello");
///
/// assert_eq!(bytes, vec![0x0, 0x5, /* <- longitud | contenido -> */ 0x48, 0x65, 0x6C, 0x6C, 0x6F]);
/// ```
pub fn encode_string_to_bytes(string: &str) -> Vec<Byte> {
    let string_bytes = string.as_bytes();
    // litle endian para que los dos bytes menos significativos (los únicos que nos interesa
    // para un Short estén al principio
    let bytes_len = string_bytes.len().to_le_bytes();
    let mut bytes_vec: Vec<Byte> = vec![
        bytes_len[1],
        bytes_len[0], // Longitud del string
    ];
    bytes_vec.extend_from_slice(string_bytes);
    bytes_vec
}

/// Parsea un [HashMap] que mapea errores de IPs a una colección de [Byte]s,
/// acorde al protocolo de Cassandra.
///
/// Comienza con un [Int](crate::cassandra::aliases::types::Int), indicando la cantidades de pares
/// clave-valor ([IpAddr]-[Short]) que vienen a continuación, seguido de la serialización de
/// dichos pares en orden.
///
/// ```rust
/// # use std::collections::HashMap;
/// # use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
/// # use aerolineas::cassandra::aliases::types::Short;
/// # use aerolineas::cassandra::utils::encode_reasonmap_to_bytes;
/// let reasonmap = HashMap::from([
///     (IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0x1400),
///     (IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1)), 0x1001)
/// ]);
/// let bytes = encode_reasonmap_to_bytes(&reasonmap);
///
/// let len = vec![0x0, 0x0, 0x0, 0x2 /* longitud del mensaje */];
/// let ipv4 = vec![0x4, /* longitud */ 0x7F, 0x0, 0x0, 0x1, /* ipv4 */ 0x14, 0x0 /* código de error */];
/// let ipv6 = vec![0x10, /* longitud */ 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x1, /* ipv6 */ 0x10, 0x1, /* error */];
///
/// // el orden de un hashmap es random, así que creamos las opciones posibles
/// let mut v4_v6 = vec![];
/// # v4_v6.extend_from_slice(&len[..]);
/// # v4_v6.extend_from_slice(&ipv4[..]);
/// # v4_v6.extend_from_slice(&ipv6[..]);
/// let mut v6_v4 = vec![];
/// # v6_v4.extend_from_slice(&len[..]);
/// # v6_v4.extend_from_slice(&ipv6[..]);
/// # v6_v4.extend_from_slice(&ipv4[..]);
///
/// assert!((bytes == v4_v6) || (bytes == v6_v4));
/// ```
pub fn encode_reasonmap_to_bytes(hashmap: &ReasonMap) -> Vec<Byte> {
    let mut bytes_vec: Vec<Byte> = vec![];

    // primero la longitud
    let hashmap_len = hashmap.len().to_le_bytes();
    bytes_vec.extend_from_slice(&[
        hashmap_len[3],
        hashmap_len[2],
        hashmap_len[1],
        hashmap_len[0],
    ]);

    for (ip, code) in hashmap {
        // la clave
        let ip_bytes = match ip {
            IpAddr::V4(ipv4) => ipv4.octets().to_vec(),
            IpAddr::V6(ipv6) => ipv6.octets().to_vec(),
        };
        let ip_len = ip_bytes.len().to_le_bytes();
        bytes_vec.extend_from_slice(&[ip_len[0]]);
        bytes_vec.extend(ip_bytes);

        // y el valor
        bytes_vec.extend(code.to_be_bytes());
    }
    bytes_vec
}

/// Parsea un conjunto de [Byte]s de vuelta a un objeto [String].
///
/// Esta es la operación recíproca a [encodearlos](encode_string_to_bytes).
///
/// ```rust
/// # use aerolineas::cassandra::utils::parse_bytes_to_string;
/// # use aerolineas::cassandra::aliases::results::Result;
/// let string = "World!".to_string();
///
/// let mut i_1: usize = 0;
/// let res_1 = parse_bytes_to_string(&[0x0, 0x6, /* <- longitud | contenido -> */ 0x57, 0x6F, 0x72, 0x6C, 0x64, 0x21], &mut i_1);
/// assert!(res_1.is_ok());
/// if let Ok(str_1) = res_1 {
///     assert_eq!(str_1, string);
///     assert_eq!(i_1, 8);
/// }
///
/// // Debería funcionar igual si el slice de bytes es más largo
/// let mut i_2: usize = 0;
/// let res_2 = parse_bytes_to_string(&[0x0, 0x6, /* <- longitud | contenido -> */ 0x57, 0x6F, 0x72, 0x6C, 0x64, 0x21, /* ruido ->*/ 0x23, 0x97, 0x24, 0x23], &mut i_2);
/// assert!(res_2.is_ok());
/// if let Ok(str_2) = res_2 {
///     assert_eq!(str_2, string);
///     assert_eq!(i_2, 8);
/// }
/// ```
pub fn parse_bytes_to_string(bytes_vec: &[Byte], i: &mut usize) -> Result<String> {
    let short_len: usize = 2; // los bytes de un Short
    if bytes_vec.len() < short_len {
        return Err(Error::SyntaxError(
            "Se esperaban 2 bytes que indiquen el tamaño del string a formar".to_string(),
        ));
    }
    let string_len = Short::from_le_bytes([bytes_vec[1], bytes_vec[0]]) as usize;
    *i += string_len + short_len;
    match String::from_utf8(bytes_vec[short_len..(string_len + short_len)].to_vec()) {
        Ok(string) => Ok(string),
        Err(_) => Err(Error::Invalid(
            "El cuerpo del string no se pudo parsear".to_string(),
        )),
    }
}

/// Parsea un conjunto de [Byte]s a un objeto [ReasonMap], conforme al protocolo de Cassandra.
///
/// Esta es la operación recíproca a [encodearlo](crate::cassandra::utils::encode_reasonmap_to_bytes),
/// y requiere de tanto el _slice_ de _bytes_ como el índice desde donde comenzar a parsear el mismo.
///
/// ```rust
/// # use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
/// # use aerolineas::cassandra::utils::parse_bytes_to_reasonmap;
/// # use aerolineas::cassandra::aliases::results::Result;
/// let bytes = vec![
///     0x0, 0x0, 0x0, 0x3, // longitud del mensaje
///     0x4, /* longitud */ 0x7F, 0x0, 0x0, 0x1, /* ipv4 */ 0x14, 0x0, /* código de error */
///     0x10, /* longitud */ 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x1, /* ipv6 */ 0x10, 0x1, /* error */
///     0x4, /* longitud */ 0x7F, 0x0, 0x0, 0x2, /* ipv4 */ 0x15, 0x0, /* error */
/// ];
/// let mut i: usize = 0;
/// let hash_res = parse_bytes_to_reasonmap(&bytes[i..], &mut i);
///
/// assert!(hash_res.is_ok());
/// if let Ok(hashmap) = hash_res {
///     assert!(hashmap.contains_key(&IpAddr::V4(Ipv4Addr::new(0x7F, 0x0, 0x0, 0x1))));
///     assert!(hashmap.contains_key(&IpAddr::V6(Ipv6Addr::new(0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x1))));
///     assert!(hashmap.contains_key(&IpAddr::V4(Ipv4Addr::new(0x7F, 0x0, 0x0, 0x2))));
///     assert_eq!(i, 4 + (1 + 4 + 2) + (1 + 16 + 2) + (1 + 4 + 2));
/// }
/// ```
pub fn parse_bytes_to_reasonmap(bytes_vec: &[Byte], i: &mut usize) -> Result<ReasonMap> {
    if bytes_vec.len() < 4 {
        return Err(Error::SyntaxError(
            "Se esperaban 4 bytes que indiquen el tamaño del reasonmap a formar".to_string(),
        ));
    }
    let mut j: usize = 0;
    let hashmap_len = Int::from_le_bytes([
        bytes_vec[j + 3],
        bytes_vec[j + 2],
        bytes_vec[j + 1],
        bytes_vec[j],
    ]) as usize;
    j += 4;

    let mut reasonmap = ReasonMap::new();
    for _ in 0..hashmap_len {
        let ip_len = Byte::from_le_bytes([bytes_vec[j]]);
        j += 1;
        let ip = match ip_len {
            4 => IpAddr::V4(std::net::Ipv4Addr::new(
                bytes_vec[j],
                bytes_vec[j + 1],
                bytes_vec[j + 2],
                bytes_vec[j + 3],
            )),
            16 => IpAddr::V6(std::net::Ipv6Addr::new(
                Short::from_be_bytes([bytes_vec[j], bytes_vec[j + 1]]),
                Short::from_be_bytes([bytes_vec[j + 2], bytes_vec[j + 3]]),
                Short::from_be_bytes([bytes_vec[j + 4], bytes_vec[j + 5]]),
                Short::from_be_bytes([bytes_vec[j + 6], bytes_vec[j + 7]]),
                Short::from_be_bytes([bytes_vec[j + 8], bytes_vec[j + 9]]),
                Short::from_be_bytes([bytes_vec[j + 10], bytes_vec[j + 11]]),
                Short::from_be_bytes([bytes_vec[j + 12], bytes_vec[j + 13]]),
                Short::from_be_bytes([bytes_vec[j + 14], bytes_vec[j + 15]]),
            )),
            _ => {
                return Err(Error::Invalid(
                    "La longitud de la dirección IP no es válida".to_string(),
                ))
            }
        };
        j += ip_len as usize;
        let code = Short::from_be_bytes([bytes_vec[j], bytes_vec[j + 1]]);
        j += 2;
        reasonmap.insert(ip, code);
    }
    *i += j; // aplicamos los cambios al índice
    Ok(reasonmap)
}
