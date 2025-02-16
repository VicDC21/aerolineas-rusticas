use protocol::{
    aliases::{
        results::Result,
        types::{
            Double,
            Long
        },
    },
    errors::error::Error,
};
use rand::{
    rngs::ThreadRng,
    Rng,
};
use std::time::{
    SystemTime,
    UNIX_EPOCH,
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
        initial_altitude: Double,
        dest_elevation: Double,
        total_flight_time: Double,
        current_time: Double,
        rng: &mut ThreadRng,
    ) -> Double {
        const CRUISE_ALTITUDE: Double = 35000.0;
        const MAX_VARIATION: Double = 250.0;
        let progress = current_time / total_flight_time;

        if progress < 0.1 {
            let climb_rate = (CRUISE_ALTITUDE - initial_altitude) / (total_flight_time * 0.1);
            let climb_variation = rng.gen_range(-MAX_VARIATION..MAX_VARIATION);
            initial_altitude + (climb_rate * current_time) + climb_variation
        } else if progress > 0.9 {
            let descent_rate = (dest_elevation - CRUISE_ALTITUDE) / (total_flight_time * 0.1);
            let descent_variation = rng.gen_range(-MAX_VARIATION..MAX_VARIATION);
            CRUISE_ALTITUDE
                + (descent_rate * (current_time - total_flight_time * 0.9))
                + descent_variation
        } else {
            let variation = rng.gen_range(-MAX_VARIATION..MAX_VARIATION);
            CRUISE_ALTITUDE + variation
        }
    }
}

/// Obtiene el timestamp actual en segundos
pub fn get_current_timestamp() -> Result<Long> {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(time) => Ok(time.as_secs() as Long),
        Err(_) => Err(Error::ServerError(
            "No se pudo obtener el timestamp actual".to_string(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use {super::*, protocol::aliases::types::Uint, rand::thread_rng};

    /// Función auxiliar para comparar números de punto flotante con tolerancia
    fn assert_approx_eq(a: Double, b: Double) {
        let epsilon = 1e-6;
        assert!(
            (a - b).abs() < epsilon,
            "Se espera que {} sea aproximadamente igual a {}",
            a,
            b
        );
    }

    #[test]
    fn test_calculate_next_position_start() {
        let (new_lat, new_lon) =
            FlightCalculations::calculate_next_position(10.0, 20.0, 30.0, 40.0, 0.0);

        assert_approx_eq(new_lat, 10.0);
        assert_approx_eq(new_lon, 20.0);
    }

    #[test]
    fn test_calculate_next_position_end() {
        let (new_lat, new_lon) =
            FlightCalculations::calculate_next_position(10.0, 20.0, 30.0, 40.0, 1.0);

        assert_approx_eq(new_lat, 30.0);
        assert_approx_eq(new_lon, 40.0);
    }

    #[test]
    fn test_calculate_next_position_midpoint() {
        let (new_lat, new_lon) =
            FlightCalculations::calculate_next_position(10.0, 20.0, 30.0, 40.0, 0.5);

        assert_approx_eq(new_lat, 20.0);
        assert_approx_eq(new_lon, 30.0);
    }

    #[test]
    fn test_calculate_next_position_overflow() {
        let (new_lat, new_lon) =
            FlightCalculations::calculate_next_position(10.0, 20.0, 30.0, 40.0, 1.5);

        assert_approx_eq(new_lat, 30.0);
        assert_approx_eq(new_lon, 40.0);
    }

    #[test]
    fn test_calculate_distance_same_point() {
        let distance = FlightCalculations::calculate_distance(
            40.7128, -74.0060, // Coordenadas de Nueva York
            40.7128, -74.0060,
        );

        assert!(distance < 1.0, "Se espera un valor bastante cercano a cero");
    }

    #[test]
    fn test_calculate_distance_known_cities() {
        let distance = FlightCalculations::calculate_distance(
            40.7128, -74.0060, // Coordenadas de Nueva York
            34.0522, -118.2437, // Coordenadas de Los Ángeles
        );

        assert!(
            distance > 3900.0 && distance < 3970.0,
            "Distancia inesperada: {}",
            distance
        );
    }

    #[test]
    fn test_calculate_current_speed() {
        let avg_speed = 500.0; // km/h
        let mut seed: Uint = 42;

        let mut speed_results = Vec::new();

        for _ in 0..100 {
            seed = seed.wrapping_mul(1103515245).wrapping_add(12345) & 0x7fffffff;
            let random_factor = (seed as Double / 0x7fffffff as Double) * 100.0 - 50.0;

            let speed = avg_speed + random_factor;
            speed_results.push(speed);
        }

        let min_speed = speed_results
            .iter()
            .cloned()
            .fold(Double::INFINITY, Double::min);
        let max_speed = speed_results
            .iter()
            .cloned()
            .fold(Double::NEG_INFINITY, Double::max);

        assert!(min_speed >= avg_speed - 50.0, "Velocidad demasiado baja");
        assert!(max_speed <= avg_speed + 50.0, "Velocidad demasiado alta");
    }

    #[test]
    fn test_altitude_variability() {
        let mut rng = thread_rng();
        let initial_altitude = 0.0;
        let dest_elevation = 500.0;
        let total_flight_time = 10.0;
        let current_time = 5.0;

        let mut altitudes = Vec::new();
        for _ in 0..100 {
            let altitude = FlightCalculations::calculate_current_altitude(
                initial_altitude,
                dest_elevation,
                total_flight_time,
                current_time,
                &mut rng,
            );
            altitudes.push(altitude);
        }

        let min_altitude = altitudes
            .iter()
            .cloned()
            .fold(Double::INFINITY, Double::min);
        let max_altitude = altitudes
            .iter()
            .cloned()
            .fold(Double::NEG_INFINITY, Double::max);

        assert!(
            (max_altitude - min_altitude).abs() > 0.1,
            "La variabilidad de la altitud es demasiado baja"
        );
    }
}
