//! Módulo para funciones auxiliares del servidor.

use {
    protocol::{
        aliases::{results::Result, types::Byte},
        errors::error::Error,
    },
    rustls::pki_types::CertificateDer,
    serde::{Deserialize, Serialize},
    std::{
        fs::{self, File},
        io::{Read, Write},
    },
};

/// Toma un elemento serializable y lo convierte a JSON, escribiendo el contenido en un archivo en la ruta recibida.
pub fn store_json<T: Serialize>(serializable: &T, path: &str) -> Result<()> {
    let file = File::create(path)
        .map_err(|e| Error::ServerError(format!("Error creando el archivo JSON: {}", e)))?;
    serde_json::to_writer_pretty(file, serializable)
        .map_err(|e| Error::ServerError(format!("Error escribiendo datos JSON: {}", e)))
}

/// Toma la ruta al nombre de un archivo JSON, cuyo contenido es serializable, lo deserealiza y devuelve el contenido.
pub fn load_json<T: for<'de> Deserialize<'de>>(path: &str) -> Result<T> {
    let content = fs::read_to_string(path)
        .map_err(|e| Error::ServerError(format!("Error leyendo datos JSON: {}", e)))?;
    serde_json::from_str(&content)
        .map_err(|e| Error::ServerError(format!("Error deserializando datos JSON: {}", e)))
}

/// Copia el contenido de un stream a otro.
pub fn move_contents<R: Read, W: Write>(src: &mut R, dest: &mut W) -> Result<()> {
    let mut buf = Vec::<Byte>::new();
    match src.read_to_end(&mut buf) {
        Err(io_err) => Err(Error::ServerError(format!(
            "Error al leer datos:\n\n{}",
            io_err
        ))),
        Ok(_) => match dest.write_all(&buf[..]) {
            Err(io_err) => Err(Error::ServerError(format!(
                "Error al escribir datos:\n\n{}",
                io_err
            ))),
            Ok(_) => Ok(()),
        },
    }
}

/// Muestra los bytes en un formato imprimible.
pub fn printable_bytes<'a>(bytes: impl IntoIterator<Item = &'a Byte>) -> String {
    let pretty_bytes = bytes
        .into_iter()
        .map(|b| format!("{:02X}", b))
        .collect::<Vec<String>>();

    format!("[ {} ]", pretty_bytes.join(" "))
}

/// Handelea los results que se devuelven al cargar el certificado
pub fn handle_pem_file_iter() -> Result<Vec<CertificateDer<'static>>> {
    let cert_file = "cert.pem";
    let certs: Vec<CertificateDer<'_>> = match CertificateDer::pem_file_iter(cert_file) {
        Ok(certs_iter) => certs_iter
            .map(|cert_res| {
                cert_res.map_err(|_| Error::Invalid("No se pudo leer un certificado".to_string()))
            })
            .collect(),
        Err(_) => Err(Error::Invalid(
            "No se pudo leer el archivo de certificados".to_string(),
        )),
    }?;
    Ok(certs)
}