//! Módulo de distancias entre dos puntos.

use data::utils::distances::distance_euclidean;
use protocol::aliases::types::Double;
use {eframe::egui::Pos2, std::time::Duration, walkers::Position};

/// Velocidad promedio de un avión (en km/h).
const AIRPLANE_AVG_SPD: Double = 900.;
/// Distancia aproximada entre un "grado" de latitud/longitud (en km).
const DEG_DIST: Double = 111.;

/// Calcula la distancia teniendo en cuenta una geometría euclideana.
///
/// Como normalmente observamos un mapa plano, esto es suficiente.
pub fn distance_euclidean_pos(pos_1: &Position, pos_2: &Position) -> Double {
    distance_euclidean(pos_1.lon(), pos_1.lat(), pos_2.lon(), pos_2.lat())
}

/// Calcula la distancia teniendo en cuenta una geometría euclideana, entre dos puntos de EGUI.
pub fn distance_euclidean_pos2(pos_1: &Pos2, pos_2: &Pos2) -> Double {
    distance_euclidean(
        pos_1.x as Double,
        pos_1.y as Double,
        pos_2.x as Double,
        pos_2.y as Double,
    )
}

/// Calcula el tiempo entre dos posiciones, asumiendo la velocidad y
/// medición de grados dada.
///
/// Si las mediciones no son proporcionadas, asumimos que cada "grado" de latitud/longitud
/// mide 111 km y que se viaja a una velocidad de 900 km/h.
pub fn distance_eta(
    pos_1: &Position,
    pos_2: &Position,
    avg_spd_opt: Option<Double>,
    deg_dist_opt: Option<Double>,
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
    let hour_in_secs: Double = 3600.;

    Duration::from_secs_f64(duration_in_hours * hour_in_secs)
}
