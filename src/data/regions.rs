//! Módulo de regiones geográficas.

use std::io::{BufRead, Result as IOResult};

use crate::data::continents::types::ContinentType;
use crate::data::utils::{
    paths::{get_tokens, reader_from},
    strings::{breakdown, to_option},
};
use crate::protocol::aliases::results::Result;
use crate::protocol::errors::error::Error;

/// La dirección por defecto del dataset de regiones.
const REGIONS_PATH: &str = "./datasets/airports/regions.csv";

/// La cantidad mínima de elementos que ha de haber en una línea del dataset de regiones.
const MIN_REGIONS_ELEMS: usize = 8;

/// Estructura que representa un país.
///
/// Este modelo está inspirado en las definiciones de [OurAirports](https://ourairports.com/help/data-dictionary.html#regions).
pub struct Region {
    /// El ID interno que el proveedor usa.
    pub id: usize,

    /// El [código local](crate::data::regions::Region::local_code) prefijado del
    /// [código de país](crate::data::regions::Region::iso_country), para crear un
    /// identificador único.
    pub code: String,

    /// Código local para la sub-división administrativa.
    ///
    /// De ser posible, se trataría de un código [ISO 3166:2](https://en.wikipedia.org/wiki/ISO_3166-2),
    /// o un identificador no oficial.
    pub local_code: String,

    /// El nombre de la región en **inglés**. Nombres en lenguas locales pueden aparecer en
    /// [keywords](crate::data::regions::Region::keywords) para ayudar con búsquedas.
    pub name: String,

    /// El tipo de continente donde la región está (primariamente) ubicada.
    pub continent: ContinentType,

    /// Mismo valor que [Country::code](crate::data::countries::Country::code).
    pub iso_country: String,

    /// El link de wikipedia describiendo la sub-división, si existe.
    pub wikipedia_link: Option<String>,

    /// Lista de palabras/frases que asisten con búsquedas.
    pub keywords: Vec<String>,
}

impl Region {
    /// Crea una instancia a partir de una lista de tokens.
    ///
    /// Se asume que dicha lista tiene suficientes elementos.
    fn from_tokens(tokens: Vec<String>) -> Result<Self> {
        let id = match tokens[0].parse::<usize>() {
            Ok(parsed) => parsed,
            Err(_) => {
                return Err(Error::ServerError(format!(
                    "'{}' no es un formato numérico válido para el ID de una región.",
                    tokens[0]
                )))
            }
        };
        let code = tokens[1].to_string();
        let local_code = tokens[2].to_string();
        let name = tokens[3].to_string();
        let continent = ContinentType::try_from(tokens[4].as_str())?;
        let iso_country = tokens[5].to_string();
        let wikipedia_link = to_option(tokens[6].as_str());
        let keywords = breakdown(&tokens[7..].join(""), ',');

        Ok(Self {
            id,
            code,
            local_code,
            name,
            continent,
            iso_country,
            wikipedia_link,
            keywords,
        })
    }

    /// Crea una nueva instancia a partir del código de región.
    pub fn from_region_code(region_code: &str) -> Result<Self> {
        let reader = reader_from(REGIONS_PATH, true)?;

        for line in reader.lines().map_while(IOResult::ok) {
            let tokens = get_tokens(&line, ',', MIN_REGIONS_ELEMS)?;

            let reg_code = tokens[1].to_string();
            if reg_code.as_str() == region_code {
                return Self::from_tokens(tokens);
            }
        }

        Err(Error::ServerError(format!(
            "No hay un país con código '{}' entre los datos.",
            region_code
        )))
    }
}
