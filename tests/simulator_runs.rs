//! Módulo para _tests_ del simulador de vuelos.

mod common;

use std::{thread::sleep, time::Duration};

use aerolineas_rusticas::{
    client::{cli::Client, conn_holder::ConnectionHolder, protocol_result::ProtocolResult},
    data::{
        flights::{states::FlightState, types::FlightType},
        tracking::live_flight_data::LiveFlightData,
    },
    simulator::flight_simulator::FlightSimulator,
};
use common::{clean_nodes, init_graph_parsing};

#[test]
fn test_1_simple_flight_adding() {
    assert!(clean_nodes().is_ok());

    let conn_res = ConnectionHolder::with_cli(Client::default());
    assert!(conn_res.is_ok());

    let graph_handle = init_graph_parsing();

    if let Ok(mut conn) = conn_res {
        let sim_res = FlightSimulator::new(8, Client::default());
        assert!(sim_res.is_ok());

        if let Ok(sim) = sim_res {
            assert!(sim
                .add_flight(123456, "SABE".to_string(), "EGAA".to_string(), 900.0)
                .is_ok());

            sleep(Duration::from_secs(5));

            let flight_data = sim.get_flight_data(123456);
            assert!(flight_data.is_some());

            if let Some(data) = flight_data {
                assert!(matches!(data.state, FlightState::InCourse));
            }

            sleep(Duration::from_secs(5));

            let client_lock = conn.get_cli();
            let tls_res = conn.get_tls_and_login(&"juan".to_string(), &"1234".to_string());

            if let Ok(mut tls_stream) = tls_res {
                if let Ok(mut client) = client_lock.lock() {
                    let select_query = "SELECT * FROM vuelos_salientes_en_vivo;";
                    let select_res = client.send_query(select_query, &mut tls_stream);
                    if let Err(err) = &select_res {
                        println!("{}", err);
                    }
                    assert!(select_res.is_ok());

                    if let Ok(protocol_res) = select_res {
                        assert!(matches!(&protocol_res, ProtocolResult::Rows(_)));
                        let live_data_res = LiveFlightData::try_from_protocol_result(
                            protocol_res.clone(),
                            &FlightType::Incoming,
                        );

                        if let ProtocolResult::Rows(rows) = protocol_res {
                            assert!(rows.len() > 1);

                            assert!(live_data_res.is_ok());
                            if let Ok(live_data) = live_data_res {
                                let latest_opt = LiveFlightData::most_recent(&live_data);
                                assert!(latest_opt.is_some());

                                if let Some(latest) = latest_opt {
                                    assert!(matches!(latest.state, FlightState::InCourse));
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    assert!(Client::default().send_shutdown().is_ok());
    assert!(graph_handle.join().is_ok());
    assert!(clean_nodes().is_ok());
}
