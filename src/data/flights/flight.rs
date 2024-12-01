//! Módulo para la estructura de vuelo.

use chrono::{DateTime, Local, TimeZone};

use crate::client::col_data::ColData;
use crate::client::protocol_result::ProtocolResult;
use crate::data::flights::{states::FlightState, types::FlightType};
use crate::protocol::aliases::{
    results::Result,
    types::{Int, Long},
};
use crate::protocol::errors::error::Error;

/// Un vuelo propiamente dicho.
#[derive(Clone, Debug)]
pub struct Flight {
    /// El ID único del vuelo saliente.
    pub id: Int,

    /// El [identificador](crate::data::airports::airp::Airport::ident) del aeropuerto de origen.
    pub orig: String,

    /// El [identificador](crate::data::airports::airp::Airport::ident) del aeropuerto de destino.
    pub dest: String,

    /// El momento en que comienzao finaliza el vuelo.
    timestamp: Long,

    /// El estado del vuelo.
    pub state: FlightState,

    /// El tipo de vuelo.
    pub flight_type: FlightType,
}

impl Flight {
    /// Crea una nueva instancia de vuelo.
    pub fn new(
        id: Int,
        orig: String,
        dest: String,
        timestamp: Long,
        state: FlightState,
        flight_type: FlightType,
    ) -> Self {
        Self {
            id,
            orig,
            dest,
            timestamp,
            state,
            flight_type,
        }
    }

    /// Consigue el timestamp del vuelo.
    ///
    /// Esto es un alias para los vuelos entrantes.
    pub fn arrival(&self) -> Long {
        self.timestamp
    }

    /// Consigue el timestamp del vuelo.
    ///
    /// Esto es un alias para los vuelos salientes.
    pub fn take_off(&self) -> Long {
        self.timestamp
    }

    /// Trata de parsear el resultado de una _query_ a los vuelos correspondientes.
    pub fn try_from_protocol_result(
        protocol_res: ProtocolResult,
        flight_type: &FlightType,
    ) -> Result<Vec<Self>> {
        let mut flights = Vec::<Self>::new();

        if let ProtocolResult::QueryError(err) = protocol_res {
            return Err(err);
        } else if let ProtocolResult::Rows(rows) = protocol_res {
            let preferred_len = 5;
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
                            if let ColData::Timestamp(timestamp) = &row[3] {
                                // 4. Estado
                                if let ColData::String(state) = &row[4] {
                                    flights.push(Self::new(
                                        *id,
                                        orig.to_string(),
                                        dest.to_string(),
                                        *timestamp,
                                        FlightState::try_from(state.as_str())?,
                                        flight_type.clone(),
                                    ));
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(flights)
    }

    /// Transforma el timestamp en una fecha.
    pub fn get_date(&self) -> Option<DateTime<Local>> {
        Local.timestamp_opt(self.timestamp, 0).single()
    }
}
