//! Módulo de distancias entre dos puntos.

use std::time::Duration;

#[cfg(feature = "gui")]
use eframe::egui::Pos2;
#[cfg(feature = "gui")]
use walkers::Position;

use crate::protocol::aliases::types::Double;

/// Velocidad promedio de un avión (en km/h).
const AIRPLANE_AVG_SPD: f64 = 900.;
/// Distancia aproximada entre un "grado" de latitud/longitud (en km).
const DEG_DIST: f64 = 111.;

/// Calcula la distancia teniendo en cuenta una geometría euclideana.
///
/// Como normalmente observamos un mapa plano, esto es suficiente.
#[cfg(feature = "gui")]
pub fn distance_euclidean_pos(pos_1: &Position, pos_2: &Position) -> f64 {
    distance_euclidean(pos_1.lon(), pos_1.lat(), pos_2.lon(), pos_2.lat())
}

/// Calcula la distancia teniendo en cuenta una geometría euclideana, entre dos puntos de EGUI.
#[cfg(feature = "gui")]
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
#[cfg(feature = "gui")]
pub fn inside_area(pos: (Double, Double), area: (Double, Double, Double, Double)) -> bool {
    let (pos_x, pos_y) = pos;
    let (area_min_x, area_min_y, area_max_x, area_max_y) = area;

    ((area_min_y <= pos_y) && (pos_y <= area_max_y))
        && ((area_min_x <= pos_x) && (pos_x <= area_max_x))
}

/// Calcula el tiempo entre dos posiciones, asumiendo la velocidad y
/// medición de grados dada.
///
/// Si las mediciones no son proporcionadas, asumimos que cada "grado" de latitud/longitud
/// mide 111 km y que se viaja a una velocidad de 900 km/h.
#[cfg(feature = "gui")]
pub fn distance_eta(
    pos_1: &Position,
    pos_2: &Position,
    avg_spd_opt: Option<f64>,
    deg_dist_opt: Option<f64>,
) -> Duration {
    let avg_spd = match avg_spd_opt {
        Some(valid) => valid,
        None => AIRPLANE_AVG_SPD,
    };
    let deg_dist = match deg_dist_opt {
        Some(valid) => valid,
        None => DEG_DIST,
    };

    let dist_in_km = distance_euclidean_pos(pos_1, pos_2) * deg_dist;
    let duration_in_hours = dist_in_km / avg_spd;
    let hour_in_secs: f64 = 3600.;

    Duration::from_secs_f64(duration_in_hours * hour_in_secs)
}
