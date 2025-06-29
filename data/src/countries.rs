//! Módulo para estructuras de países.
use {
    crate::{
        continents::types::ContinentType,
        utils::{
            paths::{get_tokens, reader_from},
            strings::breakdown,
        },
    },
    protocol::{aliases::results::Result, errors::error::Error},
    std::{
        collections::HashMap,
        io::{BufRead, Result as IOResult},
    },
    utils::get_root_path::get_root_path,
};

/// Un mapa de países.
pub type CountriesMap = HashMap<String, Country>;

/// La dirección por defecto del dataset de países.
const COUNTRIES_PATH: &str = "datasets/airports/countries.csv";

/// La cantidad mínima de elementos que ha de haber en una línea del dataset de países.
const MIN_COUNTRIES_ELEMS: usize = 6;

/// Estructura que representa un país.
///
/// Este modelo está inspirado en las definiciones de [OurAirports](https://ourairports.com/help/data-dictionary.html#countries).
#[derive(Clone, Debug, PartialEq)]
pub struct Country {
    /// El ID interno que el proveedor usa para este país.
    pub id: usize,

    /// El código de país en formato [ISO 3166:1-alpha2](https://en.wikipedia.org/wiki/List_of_ISO_3166_country_codes),
    /// así como algunas nominaciones no oficiales.
    pub code: String,

    /// El nombre del país en **inglés**. Otras variaciones podrían aparecer en [keywords](data::countries::Country::keywords)
    /// para facilitar búsquedas
    pub name: String,

    /// El tipo de continente donde el país está (primariamente) ubicado.
    pub continent: ContinentType,

    /// El link de wikipedia del país.
    pub wikipedia_link: String,

    /// Lista de palabras/frases que ayudan con búsquedas.
    pub keywords: Vec<String>,
}

impl Country {
    /// Crea una entidad vacía para _matchear_.
    pub fn dummy() -> Self {
        Self {
            id: 0,
            code: "".to_string(),
            name: "".to_string(),
            continent: ContinentType::Antarctica,
            wikipedia_link: "".to_string(),
            keywords: Vec::<String>::new(),
        }
    }

    /// Crea una instancia a partir de una lista de tokens.
    ///
    /// Se asume que dicha lista tiene suficientes elementos.
    fn from_tokens(tokens: Vec<String>) -> Result<Self> {
        // No hay forma fácil de hacer esto porque 'keywords' podría ser los últimos N elementos.
        let id = match tokens[0].parse::<usize>() {
            Ok(parsed) => parsed,
            Err(_) => {
                return Err(Error::ServerError(format!(
                    "'{}' no es un formato numérico válido para el ID de un país.",
                    tokens[0]
                )))
            }
        };
        let code = tokens[1].to_string();
        let name = tokens[2].to_string();
        let continent = ContinentType::try_from(tokens[3].as_str())?;
        let wikipedia_link = tokens[4].to_string();
        let keywords = breakdown(&tokens[5..].join(""), ',');

        Ok(Self {
            id,
            code,
            name,
            continent,
            wikipedia_link,
            keywords,
        })
    }

    /// Crea una nueva instancia a partir del código de país.
    pub fn try_from_code(country_code: &str) -> Result<Self> {
        let reader = reader_from(get_root_path(COUNTRIES_PATH).as_str(), true)?;

        for line in reader.lines().map_while(IOResult::ok) {
            let tokens = get_tokens(&line, ',', MIN_COUNTRIES_ELEMS)?;

            let code = tokens[1].to_string(); // se hace primero porque lo usamos para comparar.
            if code.as_str() == country_code {
                return Self::from_tokens(tokens);
            }
        }

        Err(Error::ServerError(format!(
            "No hay un país con código '{country_code}' entre los datos."
        )))
    }

    /// Crea un mapa gigante de todos los países disponibles.
    ///
    /// <div class="warning">
    ///
    /// _Idealmente, esta función se debería llamar lo menos posible, ya que levanta todo
    /// el dataset en memoria._
    ///
    /// </div>
    pub fn get_all() -> Result<CountriesMap> {
        let reader = reader_from(get_root_path(COUNTRIES_PATH).as_str(), true)?;
        let mut countries = CountriesMap::new();

        for line in reader.lines().map_while(IOResult::ok) {
            let tokens = get_tokens(&line, ',', MIN_COUNTRIES_ELEMS)?;

            let code = tokens[1].to_string();
            countries.insert(code, Self::from_tokens(tokens)?);
        }

        Ok(countries)
    }
}
