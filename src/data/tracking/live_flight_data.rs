//! Módulo para datos de vuelos en vivo.

use chrono::{DateTime, TimeZone, Utc};

use crate::{
    client::{col_data::ColData, protocol_result::ProtocolResult},
    data::flights::{states::FlightState, types::FlightType},
    protocol::{
        aliases::{
            results::Result,
            types::{Double, Int, Long},
        },
        errors::error::Error,
    },
};

/// Datos de vuelo en vivo.
#[derive(Clone)]
pub struct LiveFlightData {
    /// El ID del vuelo.
    pub flight_id: Int,

    /// El eropuerto de origen.
    pub orig: String,

    /// El aeropuerto de destino.
    pub dest: String,

    /// El tiempo de llegada o salida.
    timestamp: Long,

    /// La velocidal actual, o rapidez instantánea, del vuelo en curso.
    spd: Double,

    /// Un registro interno de todas las lecturas de velocidad anteriores desde
    /// que se creó la instancia.
    ///
    /// Normalmente, [crate::data::tracking::live_flight_data::LiveFlightData::spd]
    /// siempre es el último elemento de la lista.
    spd_readings: Vec<Double>,

    /// El nivel de combustible.
    pub fuel: Double,

    /// La posición geográfica en latitud y longitud.
    pub pos: (Double, Double),

    /// La altitud _(en pies, porque al parecer así es en aeronáutica)_-
    pub altitude_ft: Double,

    /// El [tipo](FlightType) del vuelo.
    pub flight_type: FlightType,

    /// El [estado](FlightState) del vuelo.
    pub state: FlightState,
}

impl LiveFlightData {
    /// Crea una nueva instancia de los datos de vuelo.
    pub fn new(
        flight_id: Int,
        orig_dest: (String, String),
        timestamp: Long,
        spd_fuel: (Double, Double),
        pos: (Double, Double),
        altitude_ft: Double,
        type_state: (FlightType, FlightState),
    ) -> Self {
        let (orig, dest) = orig_dest;
        let (spd, fuel) = spd_fuel;
        let (flight_type, state) = type_state;
        Self {
            flight_id,
            orig,
            dest,
            timestamp,
            spd,
            spd_readings: vec![spd],
            fuel,
            pos,
            altitude_ft,
            flight_type,
            state,
        }
    }

    /// Consigue la velocidad actual.
    pub fn get_spd(&self) -> &Double {
        &self.spd
    }

    /// Actualiza la velocidad.
    pub fn set_spd(&mut self, new_spd: Double) {
        self.spd_readings.push(new_spd);
        self.spd = new_spd;
    }

    /// Devuelve la velocidad promedio entre todas las lecturas anteriores.
    ///
    /// _(Se implementa a mano para recorrer la lista una sola vez)_.
    pub fn avg_spd(&self) -> Double {
        let mut total: Double = 0.;
        let mut len: Double = 0.;

        for reading in &self.spd_readings {
            total += reading;
            len += 1.;
        }

        if len == 0. {
            // El denominador en 0 es un gran no-no.
            return 0.;
        }

        total / len
    }

    /// Consigue el timestamp de los datos de vuelo.
    ///
    /// Esto es un alias para los datos de vuelos entrantes.
    pub fn arrival(&self) -> Long {
        self.timestamp
    }

    /// Consigue el timestamp de los datos de vuelo.
    ///
    /// Esto es un alias para los datos de vuelos salientes.
    pub fn take_off(&self) -> Long {
        self.timestamp
    }

    /// Devuelve la posición geográfica en latitud.
    pub fn lat(&self) -> Double {
        self.pos.0
    }

    /// Devuelve posición geográfica en la longitud.
    pub fn lon(&self) -> Double {
        self.pos.1
    }

    /// Transforma el timestamp en una fecha.
    pub fn get_date(&self) -> Option<DateTime<Utc>> {
        Utc.timestamp_opt(self.timestamp, 0).single()
    }

    /// Trata de parsear el resultado de una _query_ a los datos de vuelo correspondientes.
    pub fn try_from_protocol_result(
        protocol_res: ProtocolResult,
        flight_type: &FlightType,
    ) -> Result<Vec<Self>> {
        let mut tracking_data = Vec::<Self>::new();

        if let ProtocolResult::QueryError(err) = protocol_res {
            return Err(err);
        } else if let ProtocolResult::Rows(rows) = protocol_res {
            let preferred_len = 10;
            for row in rows {
                if row.len() != preferred_len {
                    return Err(Error::ServerError(format!(
                        "Se esperaba una fila de {} elementos, pero se encontraron {}.",
                        preferred_len,
                        row.len()
                    )));
                }

                // Esto es un crimen pero no veo otra forma.
                if let ColData::Int(flight_id) = &row[0] {
                    if let ColData::String(orig) = &row[1] {
                        if let ColData::String(dest) = &row[2] {
                            if let ColData::Timestamp(timestamp) = &row[3] {
                                if let ColData::Double(lat) = &row[4] {
                                    if let ColData::Double(lon) = &row[5] {
                                        if let ColData::String(state) = &row[6] {
                                            if let ColData::Double(spd) = &row[7] {
                                                if let ColData::Double(altitude_ft) = &row[8] {
                                                    if let ColData::Double(fuel) = &row[9] {
                                                        tracking_data.push(Self::new(
                                                            *flight_id,
                                                            (orig.to_string(), dest.to_string()),
                                                            *timestamp,
                                                            (*spd, *fuel),
                                                            (*lat, *lon),
                                                            *altitude_ft,
                                                            (
                                                                flight_type.clone(),
                                                                FlightState::try_from(
                                                                    state.as_str(),
                                                                )?,
                                                            ),
                                                        ));
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(tracking_data)
    }
}
