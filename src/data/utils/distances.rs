//! Módulo de distancias entre dos puntos.

use walkers::Position;

/// Calcula la distancia teniendo en cuenta una geometría euclideana.
/// 
/// Como normalmente observamos un mapa plano, esto es suficiente.
pub fn distance_euclidean(pos_1: &Position, pos_2: &Position) -> f64 {
    f64::sqrt((pos_2.lon() - pos_1.lon()).powi(2) + (pos_2.lat() - pos_1.lat()).powi(2))
}