//! Módulo para manejar los datos de un aeropuerto.

use {
    crate::{
        airports::types::AirportType,
        continents::types::ContinentType,
        countries::{CountriesMap, Country},
        utils::{
            distances::{distance_euclidean, inside_area},
            paths::{get_tokens, reader_from},
            strings::{breakdown, to_option},
        },
    },
    protocol::{
        aliases::{results::Result, types::Double},
        errors::error::Error,
    },
    std::{
        collections::HashMap,
        io::{BufRead, Result as IOResult},
        sync::mpsc::Sender,
    },
    utils::get_root_path::get_root_path,
};

/// Un mapa de aeropuertos.
pub type AirportsMap = HashMap<String, Airport>;

/// La dirección por defecto del dataset de aeropuertos.
const AIRPORTS_PATH: &str = "datasets/airports/cut_airports.csv";

/// La cantidad mínima de elementos que ha de haber en una línea del dataset de aeropuertos.
const MIN_AIRPORTS_ELEMS: usize = 17;

/// Estructura que representa un aeropuerto.
///
/// Este modelo está inspirado en las definiciones de [OurAirports](https://ourairports.com/help/data-dictionary.html#airports).
#[derive(Clone, Debug, PartialEq)]
pub struct Airport {
    /// El ID del aeropuerto. Éste es constante aún si el código de aeropuerto cambia.
    pub id: usize,

    /// El identificador del aeropuerto.
    ///
    /// De ser posible, se tratará del
    /// [código ICAO](https://en.wikipedia.org/wiki/ICAO_airport_code) del mismo;
    /// un [código local](data::airports::) si no hay conflictos, o un
    /// código generado internamente por el proveedor del dataset _(en cuyo
    /// caso, se arma con el código de país [ISO2](https://en.wikipedia.org/wiki/ISO_3166-1_alpha-2),
    /// seguido de un guión y 4 dígitos)_.
    pub ident: String,

    /// El [tipo](AirportType) del aeropuerto.
    pub airport_type: AirportType,

    /// El nombre oficial de aeropuerto, incluyendo la palabra _"Airport"_ o _"Airstrip"_, etc.
    pub name: String,

    /// La las coordenadas geográficas (latitud/longitud) del aeropuerto, enpaquetadas
    /// en un [Position] para comodidad.
    pub position: (Double, Double),

    /// La elevación del aeropuerto en **pies** _(ft)_, no metros.
    pub elevation_ft: Option<isize>,

    /// El código de continente donde el aeropuerto está (primariamente) ubicado.
    pub continent: ContinentType,

    /// El país donde está el aeropuerto.
    pub country: Country,

    /// Un código alfanumérico que representa la sub-división administrativa de un país donde el
    /// aeropuerto está (primariamente) ubicado.
    ///
    /// Está prefijado por el [código](data::countries::Country::code) de país y un guión (`'-'`).
    ///
    /// [Ver más]()
    pub iso_region: String,

    /// La municipalidad a la que el aeropuerto sirve _(de estar disponible)_.
    ///
    /// **Esto NO es necesariamente** la misma municipalidad donde el aeropuerto está físicamente
    /// ubicado.
    pub municipality: String,

    /// Si el aeropuerto ofrece actualmente servicios, o no.
    pub scheduled_service: bool,

    /// El código que una base de datos de avación GPS usaría normalmente para este aeropuerto.
    /// Normalmente será un código [ICAO](https://en.wikipedia.org/wiki/ICAO_airport_code) de ser posible.
    ///
    /// <div class="warning">
    ///
    /// A diferencia de [ident](data::airports::Airport::ident), no se garantiza que este
    /// valor sea globalmente único.
    ///
    /// </div>
    pub gps_code: String,

    /// El código [IATA](https://en.wikipedia.org/wiki/International_Air_Transport_Association_code)
    /// del aeropuerto, si lo hay.
    pub iata_code: Option<String>,

    /// El código local de este aeropuerto, si el mismo difiere de [gps_code](data::airports::Airport::gps_code)
    /// o [iata_code](data::airports::Airport::iata_code).
    ///
    /// Usualmente usado para puertos de EEUU.
    pub local_code: Option<String>,

    /// El link a la página oficial del aeropuerto, si existe.
    pub home_link: Option<String>,

    /// El link a la página de wikipedia del aeropuerto, si una existe.
    pub wikipedia_link: Option<String>,

    /// Palabras/frases extra para ayudar con búsquedas.
    pub keywords: Vec<String>,
}

impl Airport {
    /// Crea una entidad vacía para _matchear_.
    pub fn dummy() -> Self {
        Self {
            id: 0,
            ident: "".to_string(),
            airport_type: AirportType::Closed,
            name: "".to_string(),
            position: (0., 0.),
            elevation_ft: None,
            continent: ContinentType::Oceania, // porque no existe, es un mito
            country: Country::dummy(),
            iso_region: "".to_string(),
            municipality: "".to_string(),
            scheduled_service: false,
            gps_code: "".to_string(),
            iata_code: None,
            local_code: None,
            home_link: None,
            wikipedia_link: None,
            keywords: Vec::<String>::new(),
        }
    }

    /// Trata de parsear las coordenadas a partir de strings.
    pub fn coords(lat_str: &str, lon_str: &str) -> Result<(Double, Double)> {
        let cur_lat = match lat_str.parse::<Double>() {
            Ok(lat) => lat,
            Err(_) => {
                return Err(Error::ServerError(format!(
                    "'{}' no es un formato de latitud válido.",
                    lat_str
                )))
            }
        };
        let cur_lon = match lon_str.parse::<Double>() {
            Ok(lon) => lon,
            Err(_) => {
                return Err(Error::ServerError(format!(
                    "'{}' no es un formato de longitud válido.",
                    lon_str
                )))
            }
        };
        Ok((cur_lat, cur_lon))
    }

    /// Crea una instancia a partir de una lista de tokens.
    ///
    /// Se asume que dicha lista tiene suficientes elementos.
    fn from_tokens(tokens: Vec<String>, countries_cache: &CountriesMap) -> Result<Self> {
        let id = match tokens[0].parse::<usize>() {
            Ok(parsed) => parsed,
            Err(_) => {
                return Err(Error::ServerError(format!(
                    "'{}' no es un formato numérico válido para el ID de un aeropuerto.",
                    tokens[0]
                )))
            }
        };
        let ident = tokens[1].to_string();
        let airport_type = AirportType::try_from(tokens[2].as_str())?;
        let name = tokens[3].to_string();
        let (cur_lat, cur_lon) = Self::coords(&tokens[4], &tokens[5])?;
        let position = (cur_lat, cur_lon);
        let elevation_ft = match tokens[6].as_str() {
            "" => None,
            _ => match tokens[6].parse::<isize>() {
                Ok(parsed) => Some(parsed),
                Err(_) => {
                    return Err(Error::ServerError(format!(
                        "'{}' no es un formato numérico válido para la elevación.",
                        tokens[6]
                    )))
                }
            },
        };
        let continent = ContinentType::try_from(tokens[7].as_str())?;
        let iso_country = tokens[8].to_string();
        let country = match countries_cache.get(&iso_country) {
            Some(c) => c,
            None => &Country::try_from_code(&iso_country)?,
        };

        let iso_region = tokens[9].to_string();
        let municipality = tokens[10].to_string();
        let scheduled_service = match tokens[11].as_str() {
            "yes" => true,
            "no" => false,
            _ => false,
        };
        let gps_code = tokens[12].to_string();
        let iata_code = to_option(tokens[13].as_str());
        let local_code = to_option(tokens[14].as_str());
        let home_link = to_option(tokens[15].as_str());
        let wikipedia_link = to_option(tokens[16].as_str());
        let keywords = breakdown(&tokens[17..].join(""), ',');

        Ok(Self {
            id,
            ident,
            airport_type,
            name,
            position,
            elevation_ft,
            continent,
            country: country.clone(),
            iso_region,
            municipality,
            scheduled_service,
            gps_code,
            iata_code,
            local_code,
            home_link,
            wikipedia_link,
            keywords,
        })
    }

    /// Devuelve una lista de aeropuertos que están cerca de la posición dada.
    pub fn by_distance(
        pos: (Double, Double),
        tolerance: &Double,
        countries_cache: &CountriesMap,
    ) -> Result<Vec<Self>> {
        let reader = reader_from(get_root_path(AIRPORTS_PATH).as_str(), true)?;
        let mut airports = Vec::<Self>::new();

        for line in reader.lines().map_while(IOResult::ok) {
            let tokens = get_tokens(&line, ',', MIN_AIRPORTS_ELEMS)?;

            let (cur_lat, cur_lon) = Self::coords(&tokens[4], &tokens[5])?;
            let (lat, lon) = pos;
            if &distance_euclidean(cur_lat, cur_lon, lat, lon) <= tolerance {
                airports.push(Self::from_tokens(tokens, countries_cache)?);
            }
        }

        Ok(airports)
    }

    /// Devuelve una lista de aeropuertos que están cerca de la posición dada, basado en un cache.
    pub fn by_distance_cache(
        pos: (Double, Double),
        tolerance: &Double,
        cache: &AirportsMap,
    ) -> Vec<Self> {
        let mut airports = Vec::<Self>::new();
        let (lat, lon) = pos;
        for airp in cache.values() {
            let (airp_lat, airp_lon) = airp.position;
            if &distance_euclidean(lat, lon, airp_lat, airp_lon) <= tolerance {
                airports.push(airp.clone());
            }
        }

        airports
    }

    /// Devuelve una lista de aeropuertos que están dentro del área indicada.
    ///
    /// La primera coordenada del área está garantizada de tener valores menores que la segunda.
    pub fn by_area(
        area: (Double, Double, Double, Double),
        countries_cache: &CountriesMap,
    ) -> Result<Vec<Self>> {
        let reader = reader_from(get_root_path(AIRPORTS_PATH).as_str(), true)?;
        let mut airports = Vec::<Self>::new();

        for line in reader.lines().map_while(IOResult::ok) {
            let tokens = get_tokens(&line, ',', MIN_AIRPORTS_ELEMS)?;

            let (cur_lat, cur_lon) = Self::coords(&tokens[4], &tokens[5])?;
            if inside_area((cur_lat, cur_lon), area) {
                airports.push(Self::from_tokens(tokens, countries_cache)?);
            }
        }

        Ok(airports)
    }

    /// Devuelve una lista de aeropuertos que están dentro del área indicada, según un cache.
    ///
    /// La primera coordenada del área está garantizada de tener valores menores que la segunda.
    pub fn by_area_cache(area: (Double, Double, Double, Double), cache: &AirportsMap) -> Vec<Self> {
        let mut airports = Vec::<Self>::new();

        for airp in cache.values() {
            if inside_area(airp.position, area) {
                airports.push(airp.clone());
            }
        }

        airports
    }

    /// Devuelve un mapa gigante de todos los aeropuertos disponibles.
    ///
    /// <div class="warning">
    ///
    /// _Idealmente, esta función se debería llamar lo menos posible, ya que levanta todo
    /// el dataset en memoria._
    ///
    /// </div>
    pub fn get_all() -> Result<AirportsMap> {
        let mut airports = AirportsMap::new();
        let countries_cache = Country::get_all()?;
        let reader = reader_from(get_root_path(AIRPORTS_PATH).as_str(), true)?;

        for line in reader.lines().map_while(IOResult::ok) {
            let tokens = get_tokens(&line, ',', MIN_AIRPORTS_ELEMS)?;
            airports.insert(
                tokens[13].to_string(),
                Self::from_tokens(tokens, &countries_cache)?,
            );
        }

        Ok(airports)
    }

    /// Carga los aeropuertos y los manda cada tanto a un canal por partes.
    ///
    /// <div class="warning">
    ///
    /// _Idealmente, esta función se debería llamar lo menos posible, ya que levanta todo
    /// el dataset en memoria._
    ///
    /// </div>
    pub fn get_all_channel(sender: Sender<AirportsMap>) -> Result<()> {
        let mut airports = AirportsMap::new();
        let countries_cache = Country::get_all()?;
        let reader = reader_from(get_root_path(AIRPORTS_PATH).as_str(), true)?;
        let sendable_step = 500; // mandar cada 100 iteraciones

        for (i, line) in reader.lines().map_while(IOResult::ok).enumerate() {
            let tokens = get_tokens(&line, ',', MIN_AIRPORTS_ELEMS)?;
            airports.insert(
                tokens[1].to_string(),
                Self::from_tokens(tokens, &countries_cache)?,
            );
            if i % sendable_step == 0 {
                let _ = sender.send(airports.clone());
                airports.clear();
            }
        }

        if !airports.is_empty() {
            let _ = sender.send(airports.clone());
        }

        Ok(())
    }
}
