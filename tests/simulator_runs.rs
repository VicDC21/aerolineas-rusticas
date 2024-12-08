//! Módulo para _tests_ del simulador de vuelos.

mod common;

use std::{thread::sleep, time::Duration};

use aerolineas_rusticas::{
    client::{cli::Client, conn_holder::ConnectionHolder, protocol_result::ProtocolResult},
    data::{
        flights::{states::FlightState, types::FlightType},
        login_info::LoginInfo,
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
        let sim_res = FlightSimulator::new(8, true);
        assert!(sim_res.is_ok());

        if let Ok(sim) = sim_res {
            assert!(sim
                .add_flight(123456, "SABE".to_string(), "EGAA".to_string(), 800.0)
                .is_ok());

            sleep(Duration::from_secs(5));

            let flight_data = sim.get_flight_data(123456);
            assert!(flight_data.is_some());

            if let Some(data) = flight_data {
                assert!(matches!(data.state, FlightState::InCourse));
            }

            sleep(Duration::from_secs(5));

            let client_lock = conn.get_cli();
            let login_res = conn.login(&LoginInfo::new_str("juan", "1234"));
            assert!(login_res.is_ok());

            if let Ok(mut client) = client_lock.lock() {
                let select_query = "SELECT * FROM vuelos_salientes_en_vivo;";
                let select_res = client.send_query(select_query, &mut conn.tls_stream);
                if let Err(err) = &select_res {
                    println!("{}", err);
                }
                assert!(select_res.is_ok());

                if let Ok((protocol_res, _)) = select_res {
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
            };
        }
    }

    assert!(Client::default().send_shutdown().is_ok());
    assert!(graph_handle.join().is_ok());
    assert!(clean_nodes().is_ok());
}
