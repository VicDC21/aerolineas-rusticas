//! Módulo para estructuras de países.

use std::convert::TryFrom;

use crate::data::continent_types::ContinentType;
use crate::protocol::errors::error::Error;

/// La dirección por defecto del dataset de países.
const COUNTRIES_PATH: &str = "./datasets/airports/countries.csv";

/// Estructura que representa un país.
///
/// Este modelo está inspirado en las definiciones de [OurAirports](https://ourairports.com/help/data-dictionary.html#countries).
pub struct Country {
    /// El ID interno que el proveedor usa para este país.
    id: usize,

    /// El código de país en formato [ISO 3166:1-alpha2](https://en.wikipedia.org/wiki/List_of_ISO_3166_country_codes),
    /// así como algunas nominaciones no oficiales.
    code: String,

    /// El nombre del país en **inglés**. Otras variaciones podrían aparecer en [keywords](crate::data::countries::Country::keywords)
    /// para facilitar búsquedas
    name: String,

    /// El tipo de continente donde el país está (primariamente) ubicado.
    continent: ContinentType,

    /// El link de wikipedia del país.
    wikipedia_link: String,

    /// Lista de palabras/frases que ayudan con búsquedas.
    keywords: Vec<String>,
}

impl TryFrom<String> for Country {
    type Error = Error;
    /// Crea una nueva instancia de país, a partir de un [String].
    ///
    /// Se asume que el [String] es una línea de un archivo CSV, sin parsear.
    fn try_from(line: String) -> Result<Self, Self::Error> {
        Err(Error::ServerError("TODO: Aún no implementado".to_string()))
    }
}
