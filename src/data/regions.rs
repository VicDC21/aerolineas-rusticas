//! Módulo de regiones geográficas.

use std::convert::TryFrom;

use crate::protocol::errors::error::Error;

/// La dirección por defecto del dataset de regiones.
const REGIONS_PATH: &str = "./datasets/airports/regions.csv";

/// Estructura que representa un país.
///
/// Este modelo está inspirado en las definiciones de [OurAirports](https://ourairports.com/help/data-dictionary.html#regions).
pub struct Region {
    /// El ID interno que el proveedor usa.
    id: usize,

    /// El [código local](crate::data::regions::Region::local_code) prefijado del
    /// [código de país](crate::data::regions::Region::iso_country), para crear un
    /// identificador único.
    code: String,

    /// Código local para la sub-división administrativa.
    ///
    /// De ser posible, se trataría de un código [ISO 3166:2](https://en.wikipedia.org/wiki/ISO_3166-2),
    /// o un identificador no oficial.
    local_code: String,

    /// El nombre de la región en **inglés**. Nombres en lenguas locales pueden aparecer en
    /// [keywords](crate::data::regions::Region::keywords) para ayudar con búsquedas.
    name: String,

    /// Mismo valor que [Country::code](crate::data::countries::Country::code).
    iso_country: String,

    /// El link de wikipedia describiendo la sub-división, si existe.
    wikipedia_link: Option<String>,

    /// Lista de palabras/frases que asisten con búsquedas.
    keywords: Vec<String>,
}

impl TryFrom<String> for Region {
    type Error = Error;
    /// Crea una nueva instancia de región, a partir de un [String].
    ///
    /// Se asume que el [String] es una línea de un archivo CSV, sin parsear.
    fn try_from(line: String) -> Result<Self, Self::Error> {
        Err(Error::ServerError("TODO: Aún no implementado".to_string()))
    }
}
