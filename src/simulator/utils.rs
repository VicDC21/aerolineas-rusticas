use {
    crate::protocol::aliases::types::Double,
    rand::{rngs::ThreadRng, Rng},
};

/// Funciones de utilidad para cálculos de vuelo
pub struct FlightCalculations;

impl FlightCalculations {
    /// Calcula la siguiente posición basada en la posición actual, destino y tamaño del paso
    pub fn calculate_next_position(
        current_lat: Double,
        current_lon: Double,
        dest_lat: Double,
        dest_lon: Double,
        progress: Double,
    ) -> (Double, Double) {
        let progress = progress.min(1.0);
        let lat1 = current_lat.to_radians();
        let lon1 = current_lon.to_radians();
        let lat2 = dest_lat.to_radians();
        let lon2 = dest_lon.to_radians();

        let lat_diff = lat2 - lat1;
        let lon_diff = lon2 - lon1;

        let new_lat = lat1 + (lat_diff * progress);
        let new_lon = lon1 + (lon_diff * progress);

        (new_lat.to_degrees(), new_lon.to_degrees())
    }

    /// Calcula la distancia entre dos puntos usando la fórmula haversine
    pub fn calculate_distance(lat1: Double, lon1: Double, lat2: Double, lon2: Double) -> Double {
        let r = 6371.0;
        let d_lat = (lat2 - lat1).to_radians();
        let d_lon = (lon2 - lon1).to_radians();
        let lat1 = lat1.to_radians();
        let lat2 = lat2.to_radians();

        let a = (d_lat / 2.0).sin().powi(2) + lat1.cos() * lat2.cos() * (d_lon / 2.0).sin().powi(2);
        let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

        r * c
    }

    /// Calcula la velocidad actual basada en la velocidad promedio y el progreso del vuelo
    pub fn calculate_current_speed(avg_speed: Double, rng: &mut ThreadRng) -> Double {
        avg_speed + rng.gen_range(-50.0..50.0)
    }

    /// Calcula la altitud actual agregando una variación aleatoria
    pub fn calculate_current_altitude(
        altitude: Double,
        rng: &mut ThreadRng,
        progress: Double,
    ) -> Double {
        const CRUISE_ALTITUDE: Double = 35000.0;
        if progress < 0.1 {
            let climb_variation = rng.gen_range(2000.0..3000.0);
            altitude + climb_variation
        } else if progress > 0.9 {
            let descent_variation = rng.gen_range(-4000.0..-3000.0);
            altitude + descent_variation
        } else {
            let variation = rng.gen_range(-500.0..500.0);
            CRUISE_ALTITUDE + variation
        }
    }
}
