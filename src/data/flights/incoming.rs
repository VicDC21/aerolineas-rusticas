//! Módulo para vuelos entrantes.

use chrono::{DateTime, TimeZone, Utc};
use walkers::Position;

use crate::client::col_data::ColData;
use crate::client::protocol_result::ProtocolResult;
use crate::data::flights::{states::FlightState, traits::Flight};
use crate::protocol::aliases::{
    results::Result,
    types::{Int, Long},
};
use crate::protocol::errors::error::Error;

/// Un vuelo esperando a concluir.
#[derive(Debug)]
pub struct IncomingFlight {
    /// El ID único del vuelo entrante.
    pub id: Int,

    /// El [identificador](crate::data::airports::Airport::ident) del aeropuerto de destino.
    pub dest: String,

    /// El momento en que se calcula que el vuelo concluya.
    pub arrival: Long,

    /// La posición actual del vuelo.
    pub pos: Position,

    /// El estado del vuelo.
    pub state: FlightState,
}

impl IncomingFlight {
    /// Crea una nueva instancia de vuelo.
    pub fn new(id: Int, dest: String, arrival: Long, pos: Position, state: FlightState) -> Self {
        Self {
            id,
            dest,
            arrival,
            pos,
            state,
        }
    }

    /// Trata de parsear el resultado de una _query_ a los vuelos correspondientes.
    pub fn try_from_protocol_result(protocol_res: ProtocolResult) -> Result<Vec<Self>> {
        let mut incoming = Vec::<Self>::new();

        if let ProtocolResult::QueryError(err) = protocol_res {
            return Err(err);
        } else if let ProtocolResult::Rows(rows) = protocol_res {
            let preferred_len = 6;
            for row in rows {
                if row.len() != preferred_len {
                    return Err(Error::ServerError(format!(
                        "Se esperaba una fila de {} elementos, pero se encontraron {}.",
                        preferred_len,
                        row.len()
                    )));
                }

                // 0. ID
                if let ColData::Int(id) = &row[0] {
                    // 1. Destino
                    if let ColData::String(dest) = &row[1] {
                        // 2. Llegada
                        if let ColData::Timestamp(arrival) = &row[2] {
                            // 3. Latitud
                            if let ColData::Double(lat) = &row[3] {
                                // 4. Longitud
                                if let ColData::Double(lon) = &row[4] {
                                    // 5. Estado
                                    if let ColData::String(state) = &row[5] {
                                        incoming.push(IncomingFlight::new(
                                            *id,
                                            dest.to_string(),
                                            *arrival,
                                            Position::from_lat_lon(*lat, *lon),
                                            FlightState::try_from(state.as_str())?,
                                        ));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(incoming)
    }
}

impl Flight for IncomingFlight {
    fn dummy() -> Self {
        Self::new(
            0,
            "".to_string(),
            0,
            Position::from_lat_lon(0., 0.),
            FlightState::Canceled,
        )
    }

    fn get_date(&self) -> Option<DateTime<Utc>> {
        Utc.timestamp_opt(self.arrival, 0).single()
    }
}
