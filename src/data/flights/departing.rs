//! Módulo para vuelos salientes.

use chrono::{DateTime, TimeZone, Utc};

use crate::client::col_data::ColData;
use crate::client::protocol_result::ProtocolResult;
use crate::data::flights::{states::FlightState, traits::Flight};
use crate::protocol::aliases::{
    results::Result,
    types::{Int, Long},
};
use crate::protocol::errors::error::Error;

/// Un vuelo que ha partido.
#[derive(Debug)]
pub struct DepartingFlight {
    /// El ID único del vuelo saliente.
    pub id: Int,

    /// El [identificador](crate::data::airports::Airport::ident) del aeropuerto de origen.
    pub orig: String,

    /// El [identificador](crate::data::airports::Airport::ident) del aeropuerto de destino.
    pub dest: String,

    /// El momento en que ha comenzado el vuelo.
    pub take_off: Long,

    /// El estado del vuelo.
    pub state: FlightState,
}

impl DepartingFlight {
    /// Crea una nueva instancia de vuelo.
    pub fn new(id: Int, orig: String, dest: String, take_off: Long, state: FlightState) -> Self {
        Self {
            id,
            orig,
            dest,
            take_off,
            state,
        }
    }

    /// Trata de parsear el resultado de una _query_ a los vuelos correspondientes.
    pub fn try_from_protocol_result(protocol_res: ProtocolResult) -> Result<Vec<Self>> {
        let mut departing = Vec::<Self>::new();

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
                    // 1. Origen
                    if let ColData::String(orig) = &row[1] {
                        // 2. Destino
                        if let ColData::String(dest) = &row[2] {
                            // 3. Salida
                            if let ColData::Timestamp(take_off) = &row[3] {
                                // 4. Estado
                                if let ColData::String(state) = &row[4] {
                                    departing.push(DepartingFlight::new(
                                        *id,
                                        orig.to_string(),
                                        dest.to_string(),
                                        *take_off,
                                        FlightState::try_from(state.as_str())?,
                                    ));
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(departing)
    }
}

impl Flight for DepartingFlight {
    fn dummy() -> Self {
        Self::new(0, "".to_string(), "".to_string(), 0, FlightState::Canceled)
    }

    fn get_date(&self) -> Option<DateTime<Utc>> {
        Utc.timestamp_opt(self.take_off, 0).single()
    }
}
