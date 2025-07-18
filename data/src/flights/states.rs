//! Módulo para el estado de un vuelo.
use {
    crate::traits::PrettyShow,
    protocol::errors::error::Error,
    std::fmt::{Display, Formatter, Result as FmtResult},
};

/// Un mismo vuelo puede cancelarse, atrasarse, u otras cosas que es necesario detectar.

#[derive(Debug, Clone, PartialEq)]
pub enum FlightState {
    /// Un vuelo en preparación.
    Preparing,

    /// Un vuelo actualmente ne curso.
    InCourse,

    /// Un vuelo finalizado correctamente (no [cancelado](data::flight_states::FlightState::Canceled)).
    Finished,

    /// Un vuelo atrasado.
    Delayed,

    /// Un vuelo cancelado.
    Canceled,
}

impl PrettyShow for FlightState {
    fn pretty_name(&self) -> &str {
        match self {
            Self::InCourse => "In Course",
            Self::Delayed => "Delayed",
            Self::Canceled => "Canceled",
            Self::Finished => "Finished",
            Self::Preparing => "Preparing",
        }
    }
}

impl Display for FlightState {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::InCourse => write!(f, "in_course"),
            Self::Delayed => write!(f, "delayed"),
            Self::Canceled => write!(f, "canceled"),
            Self::Finished => write!(f, "finished"),
            Self::Preparing => write!(f, "preparing"),
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
            "preparing" => Ok(Self::Preparing),
            _ => Err(Error::ServerError(format!(
                "'{state}' no es un nombre válido de estado de vuelo."
            ))),
        }
    }
}
