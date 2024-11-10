//! Módulo para el estado de un vuelo.

use std::fmt::{Display, Formatter, Result as FmtResult};

use crate::protocol::errors::error::Error;

/// Un mismo vuelo puede cancelarse, atrasarse, u otras cosas que es necesario detectar.
pub enum FlightState {
    /// Un vuelo actualmente ne curso.
    InCourse,

    /// Un vuelo atrasado.
    Delayed,

    /// Un vuelo cancelado.
    Canceled,

    /// Un vuelo finalizado correctamente (no [cancelado](crate::data::flight_states::FlightState::Canceled)).
    Finished,
}

impl Display for FlightState {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::InCourse => write!(f, "in_course"),
            Self::Delayed => write!(f, "delayed"),
            Self::Canceled => write!(f, "canceled"),
            Self::Finished => write!(f, "finished"),
        }
    }
}

impl TryFrom<&str> for FlightState {
    type Error = Error;
    fn try_from(state: &str) -> Result<Self, Self::Error> {
        match state {
            "in_course" => Ok(Self::InCourse),
            "delayed" => Ok(Self::Delayed),
            "canceled" => Ok(Self::Canceled),
            "finished" => Ok(Self::Finished),
            _ => Err(Error::ServerError(format!(
                "'{}' no es un nombre válido de estado de vuelo.",
                state
            ))),
        }
    }
}
