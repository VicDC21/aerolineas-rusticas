//! Módulo para manejar los datos de un aeropuerto.

use crate::data::airport_types::AirportType;

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
}