#[cfg(test)]
mod tests {
    use {
        crate::{
            data::flights::states::FlightState,
            protocol::aliases::{results::Result, types::Double},
            simulator::flight_simulator::{FlightSimulator, FLIGHT_LIMIT_SECS},
        },
        std::{thread, time::Duration},
    };

    #[test]
    fn test_flight_simulator() -> Result<()> {
        let simulator = FlightSimulator::default();

        simulator.add_flight(123456, "EZE".to_string(), "MAD".to_string(), 900.0)?;
        assert!(simulator.get_flight_data(123456).is_some());

        if let Some(data) = simulator.get_flight_data(123456) {
            assert_eq!(data.state, FlightState::Preparing);
        }

        thread::sleep(Duration::from_secs(3));

        if let Some(data) = simulator.get_flight_data(123456) {
            assert_eq!(data.state, FlightState::InCourse);
        }

        thread::sleep(Duration::from_secs(FLIGHT_LIMIT_SECS));

        if let Some(data) = simulator.get_flight_data(123456) {
            assert_eq!(
                data.state,
                FlightState::Finished,
                "El estado del vuelo es {:?} cuando debería ser Finished",
                data.state
            );
        }

        Ok(())
    }

    #[test]
    fn test_concurrent_flights_simulation() -> Result<()> {
        let simulator = FlightSimulator::default();

        let flight_configs = vec![
            (234567, "EZE", "MAD", 900.0),
            (345678, "MAD", "EZE", 800.0),
            (456789, "EZE", "CDG", 1000.0),
            (567890, "CDG", "EZE", 950.0),
        ];

        for &(flight_id, origin, destination, avg_spd) in &flight_configs {
            simulator.add_flight(
                flight_id,
                origin.to_string(),
                destination.to_string(),
                avg_spd as Double,
            )?;
        }

        let check_intervals = 5;
        let total_wait_time = FLIGHT_LIMIT_SECS + check_intervals;
        let check_interval_duration = total_wait_time / check_intervals;

        for _ in 0..check_intervals {
            thread::sleep(Duration::from_secs(check_interval_duration));

            for &(flight_id, _, _, _) in &flight_configs {
                let flight_data = simulator.get_flight_data(flight_id);
                assert!(flight_data.is_some(), "Vuelo {} no encontrado", flight_id);
            }
        }

        for &(flight_id, _, _, _) in &flight_configs {
            let flight_data = simulator.get_flight_data(flight_id);
            assert!(flight_data.is_some(), "Vuelo {} no encontrado", flight_id);

            if let Some(data) = flight_data {
                assert_eq!(
                    data.state,
                    FlightState::Finished,
                    "El vuelo {} no ha finalizado como se esperaba. Estado actual: {:?}",
                    flight_id,
                    data.state
                );
            }
        }

        let all_flights = simulator.get_all_flights();
        assert_eq!(
            all_flights.len(),
            flight_configs.len(),
            "No se registraron todos los vuelos"
        );

        Ok(())
    }
}
