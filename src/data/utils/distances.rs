//! Módulo de distancias entre dos puntos.

use walkers::Position;

/// Calcula la distancia teniendo en cuenta una geometría euclideana.
///
/// Como normalmente observamos un mapa plano, esto es suficiente.
pub fn distance_euclidean(pos_1: &Position, pos_2: &Position) -> f64 {
    f64::sqrt((pos_2.lon() - pos_1.lon()).powi(2) + (pos_2.lat() - pos_1.lat()).powi(2))
}

/// Calcula si una posición está entre otras dos posiciones.
///
/// Se asume que la primera coordenada tiene valores menores que la segunda.
pub fn inside_area(pos: &Position, area: (&Position, &Position)) -> bool {
    let (area_min, area_max) = area;

    ((area_min.lat() <= pos.lat()) && (pos.lat() <= area_max.lat()))
        && ((area_min.lon() <= pos.lon()) && (pos.lon() <= area_max.lon()))
}
