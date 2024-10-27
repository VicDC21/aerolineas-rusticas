//! Módulo de distancias entre dos puntos.

use eframe::egui::Pos2;
use walkers::Position;

/// Calcula la distancia teniendo en cuenta una geometría euclideana.
///
/// Como normalmente observamos un mapa plano, esto es suficiente.
pub fn distance_euclidean_wpos(pos_1: &Position, pos_2: &Position) -> f64 {
    distance_euclidean(pos_1.lon(), pos_1.lat(), pos_2.lon(), pos_2.lat())
}

/// Calcula la distancia teniendo en cuenta una geometría euclideana, entre dos puntos de EGUI.
pub fn distance_euclidean_pos2(pos_1: &Pos2, pos_2: &Pos2) -> f64 {
    distance_euclidean(
        pos_1.x as f64,
        pos_1.y as f64,
        pos_2.x as f64,
        pos_2.y as f64,
    )
}

/// Calcula la distancia euclideana entre dos puntos genéricos.
pub fn distance_euclidean(x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {
    f64::sqrt((x2 - x1).powi(2) + (y2 - y1).powi(2))
}

/// Calcula si una posición está entre otras dos posiciones.
///
/// Se asume que la primera coordenada tiene valores menores que la segunda.
pub fn inside_area(pos: &Position, area: (&Position, &Position)) -> bool {
    let (area_min, area_max) = area;

    ((area_min.lat() <= pos.lat()) && (pos.lat() <= area_max.lat()))
        && ((area_min.lon() <= pos.lon()) && (pos.lon() <= area_max.lon()))
}
