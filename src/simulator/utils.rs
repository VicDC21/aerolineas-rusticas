use {crate::protocol::aliases::types::Double, rand::Rng};

/// Funciones de utilidad para cálculos de vuelo
pub struct FlightCalculations;

impl FlightCalculations {
    /// Calcula la siguiente posición basada en la posición actual, destino y tamaño del paso
    pub fn calculate_next_position(
        current_lat: Double,
        current_lon: Double,
        dest_lat: Double,
        dest_lon: Double,
        step_size: Double,
    ) -> (Double, Double) {
        let r = 6371.0;

        let lat1 = current_lat.to_radians();
        let lon1 = current_lon.to_radians();
        let lat2 = dest_lat.to_radians();
        let lon2 = dest_lon.to_radians();

        let d_lon = lon2 - lon1;
        let d_lat = lat2 - lat1;
        let a = (d_lat / 2.0).sin().powi(2) + lat1.cos() * lat2.cos() * (d_lon / 2.0).sin().powi(2);
        let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
        let current_distance = r * c;

        if current_distance < step_size {
            return (dest_lat, dest_lon);
        }

        let y = d_lon.sin() * lat2.cos();
        let x = lat1.cos() * lat2.sin() - lat1.sin() * lat2.cos() * d_lon.cos();
        let bearing = y.atan2(x);

        let angular_distance = step_size / r;

        let new_lat = (lat1.sin() * angular_distance.cos()
            + lat1.cos() * angular_distance.sin() * bearing.cos())
        .asin();

        let new_lon = lon1
            + (bearing.sin() * angular_distance.sin() * lat1.cos())
                .atan2(angular_distance.cos() - lat1.sin() * new_lat.sin());

        (
            (new_lat.to_degrees() * 10000.0).round() / 10000.0,
            (new_lon.to_degrees() * 10000.0).round() / 10000.0,
        )
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

    /// Calcula la altitud de crucero basada en la elevación de origen y destino
    pub fn calculate_cruise_altitude(
        origin_elevation: Double,
        dest_elevation: Double,
        progress: Double,
    ) -> Double {
        const CRUISE_ALTITUDE: Double = 35000.0;
        const ASCENT_PHASE: Double = 0.15;
        const DESCENT_PHASE: Double = 0.85;

        if progress < ASCENT_PHASE {
            origin_elevation
                + (CRUISE_ALTITUDE - origin_elevation) * (progress / ASCENT_PHASE).powi(2)
        } else if progress > DESCENT_PHASE {
            let descent_progress = (progress - DESCENT_PHASE) / (1.0 - DESCENT_PHASE);
            CRUISE_ALTITUDE - (CRUISE_ALTITUDE - dest_elevation) * descent_progress.powi(2)
        } else {
            CRUISE_ALTITUDE
        }
    }

    /// Calcula la velocidad actual basada en la velocidad promedio y el progreso del vuelo
    pub fn calculate_current_speed(
        avg_speed: Double,
        progress: Double,
        rng: &mut rand::rngs::ThreadRng,
    ) -> Double {
        const ASCENT_PHASE: Double = 0.15;
        const DESCENT_PHASE: Double = 0.85;

        let base_speed = if progress < ASCENT_PHASE {
            avg_speed * (progress / ASCENT_PHASE).powi(2)
        } else if progress > DESCENT_PHASE {
            let descent_progress = (progress - DESCENT_PHASE) / (1.0 - DESCENT_PHASE);
            avg_speed * (1.0 - descent_progress.powi(2))
        } else {
            avg_speed
        };

        base_speed * (1.0 + rng.gen_range(-0.02..0.02))
    }

    /// Calcula la altitud actual agregando una variación aleatoria
    pub fn calculate_current_altitude(
        base_altitude: Double,
        rng: &mut rand::rngs::ThreadRng,
    ) -> Double {
        base_altitude + rng.gen_range(-50.0..50.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_distance() {
        let distance = FlightCalculations::calculate_distance(-34.6037, -58.3816, 40.4168, -3.7038);

        assert!((distance - 10000.0).abs() < 500.0);
    }

    #[test]
    fn test_cruise_altitude() {
        let origin_elevation = 0.0;
        let dest_elevation = 2000.0;

        let altitude_takeoff =
            FlightCalculations::calculate_cruise_altitude(origin_elevation, dest_elevation, 0.05);
        assert!(altitude_takeoff > origin_elevation);
        assert!(altitude_takeoff < 35000.0);

        let altitude_cruise =
            FlightCalculations::calculate_cruise_altitude(origin_elevation, dest_elevation, 0.5);
        assert_eq!(altitude_cruise, 35000.0);

        let altitude_landing =
            FlightCalculations::calculate_cruise_altitude(origin_elevation, dest_elevation, 0.95);
        assert!(altitude_landing < 35000.0);
        assert!(altitude_landing > dest_elevation);
    }
}
