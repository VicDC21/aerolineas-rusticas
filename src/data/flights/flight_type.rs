//! Módulo para los tipos de vuelos a cargar de diferentes tablas.
//!
//! Esto NO es lo mismo que los [estados](crate::data::flights::states::FlightState) de vuelos.

use crate::data::flights::{departing::DepartingFlight, incoming::IncomingFlight};

/// Un tipo de aeropuerto cargable.
///
/// Principalmente los vuelos entrantes y los salientes.
pub enum FlightType {
    /// Un vuelo entrante.
    Incoming(IncomingFlight),

    /// Un vuelo saliente.
    Departing(DepartingFlight),
}

impl FlightType {
    /// Devuelve un vector sólo con los tipos entrantes.
    pub fn filter_incoming(flights: Vec<Self>) -> Vec<IncomingFlight> {
        let mut incoming = Vec::<IncomingFlight>::new();

        for flight in flights {
            if let Self::Incoming(inc) = flight {
                incoming.push(inc);
            }
        }

        incoming
    }

    /// Devuelve un vector sólo con los tipos salientes.
    pub fn filter_departing(flights: Vec<Self>) -> Vec<DepartingFlight> {
        let mut departing = Vec::<DepartingFlight>::new();

        for flight in flights {
            if let Self::Departing(dep) = flight {
                departing.push(dep);
            }
        }

        departing
    }
}
