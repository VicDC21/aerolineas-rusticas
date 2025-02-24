//! Módulo para los tipos de vuelos a cargar de diferentes tablas.
//!
//! Esto NO es lo mismo que los [estados](data::flights::states::FlightState) de vuelos.

use {
    crate::traits::PrettyShow,
    std::fmt::{Display, Formatter, Result as FmtResult},
};

/// Un tipo de aeropuerto cargable.
///
/// Principalmente los vuelos entrantes y los salientes.
#[derive(Clone, Debug, PartialEq)]
pub enum FlightType {
    /// Un vuelo entrante.
    Incoming,

    /// Un vuelo saliente.
    Departing,
}

impl PrettyShow for FlightType {
    fn pretty_name(&self) -> &str {
        match self {
            Self::Incoming => "Incoming",
            Self::Departing => "Departing",
        }
    }
}

impl Display for FlightType {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::Incoming => write!(f, "incoming"),
            Self::Departing => write!(f, "departing"),
        }
    }
}
