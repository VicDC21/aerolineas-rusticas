//! Módulo para los tipos de vuelos a cargar de diferentes tablas.
//!
//! Esto NO es lo mismo que los [estados](crate::data::flights::states::FlightState) de vuelos.

/// Un tipo de aeropuerto cargable.
///
/// Principalmente los vuelos entrantes y los salientes.
#[derive(Clone, Debug)]
pub enum FlightType {
    /// Un vuelo entrante.
    Incoming,

    /// Un vuelo saliente.
    Departing,
}
