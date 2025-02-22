//! Módulo de distancias entre dos puntos.

use protocol::aliases::types::Double;

/// Calcula la distancia euclideana entre dos puntos genéricos.
pub fn distance_euclidean(x1: Double, y1: Double, x2: Double, y2: Double) -> Double {
    Double::sqrt((x2 - x1).powi(2) + (y2 - y1).powi(2))
}

/// Calcula si una posición está entre otras dos posiciones.
///
/// Se asume que la primera coordenada tiene valores menores que la segunda.
pub fn inside_area(pos: (Double, Double), area: (Double, Double, Double, Double)) -> bool {
    let (pos_x, pos_y) = pos;
    let (area_min_x, area_min_y, area_max_x, area_max_y) = area;

    ((area_min_y <= pos_y) && (pos_y <= area_max_y))
        && ((area_min_x <= pos_x) && (pos_x <= area_max_x))
}
