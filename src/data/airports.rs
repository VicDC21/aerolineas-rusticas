//! Módulo para manejar los datos de un aeropuerto.

use std::convert::TryFrom;
use walkers::Position;

use crate::data::{airport_types::AirportType, continent_types::ContinentType};
use crate::protocol::errors::error::Error;

/// La dirección por defecto del dataset de aeropuertos.
const AIRPORTS_PATH: &str = "./datasets/airports/airports.csv";

/// Estructura que representa un aeropuerto.
///
/// Este modelo está inspirado en las definiciones de [OurAirports](https://ourairports.com/help/data-dictionary.html#airports).
pub struct Airport {
    /// El ID del aeropuerto. Éste es constante aún si el código de aeropuerto cambia.
    id: usize,

    /// El identificador del aeropuerto.
    ///
    /// De ser posible, se tratará del
    /// [código ICAO](https://en.wikipedia.org/wiki/ICAO_airport_code) del mismo;
    /// un [código local](crate::data::airports::) si no hay conflictos, o un
    /// código generado internamente por el proveedor del dataset _(en cuyo
    /// caso, se arma con el código de país [ISO2](https://en.wikipedia.org/wiki/ISO_3166-1_alpha-2),
    /// seguido de un guión y 4 dígitos)_.
    ident: String,

    /// El [tipo](AirportType) del aeropuerto.
    airport_type: AirportType,

    /// El nombre oficial de aeropuerto, incluyendo la palabra _"Airport"_ o _"Airstrip"_, etc.
    name: String,

    /// La las coordenadas geográficas (latitud/longitud) del aeropuerto, enpaquetadas
    /// en un [Position] para comodidad.
    position: Position,

    /// La elevación del aeropuerto en **pies** _(ft)_, no metros.
    elevation_ft: usize,

    /// El código de continente donde el aeropuerto está (primariamente) ubicado.
    continent: ContinentType,

    /// Mismo valor que el apartado [code](crate::data::countries::Country::code) de [Country](crate::data::countries::Country).
    iso_country: String,

    /// Un código alfanumérico que representa la sub-división administrativa de un país donde el
    /// aeropuerto está (primariamente) ubicado.
    ///
    /// Está prefijado por el [código](crate::data::countries::Country::code) de país y un guión (`'-'`).
    ///
    /// [Ver más]()
    iso_region: String,

    /// La municipalidad a la que el aeropuerto sirve _(de estar disponible)_.
    ///
    /// **Esto NO es necesariamente** la misma municipalidad donde el aeropuerto está físicamente
    /// ubicado.
    municipality: String,

    /// Si el aeropuerto ofrece actualmente servicios, o no.
    scheduled_service: bool,

    /// El código que una base de datos de avación GPS usaría normalmente para este aeropuerto.
    /// Normalmente será un código [ICAO](https://en.wikipedia.org/wiki/ICAO_airport_code) de ser posible.
    ///
    /// <div class="warning">
    ///
    /// A diferencia de [ident](crate::data::airports::Airport::ident), no se garantiza que este
    /// valor sea globalmente único.
    ///
    /// </div>
    gps_code: String,

    /// El código [IATA](https://en.wikipedia.org/wiki/International_Air_Transport_Association_code)
    /// del aeropuerto, si lo hay.
    iata_code: Option<String>,

    /// El código local de este aeropuerto, si el mismo difiere de [gps_code](crate::data::airports::Airport::gps_code)
    /// o [iata_code](crate::data::airports::Airport::iata_code).
    ///
    /// Usualmente usado para puertos de EEUU.
    local_code: Option<String>,

    /// El link a la página oficial del aeropuerto, si existe.
    home_link: Option<String>,

    /// El link a la página de wikipedia del aeropuerto, si una existe.
    wikipedia_link: Option<String>,

    /// Palabras/frases extra para ayudar con búsquedas.
    keywords: Vec<String>,
}

impl TryFrom<String> for Airport {
    type Error = Error;
    /// Crea una nueva instancia de aeropuerto, a partir de un [String].
    ///
    /// Se asume que el [String] es una línea de un archivo CSV, sin parsear.
    fn try_from(line: String) -> Result<Self, Self::Error> {
        Err(Error::ServerError("TODO: Aún no implementado".to_string()))
    }
}
