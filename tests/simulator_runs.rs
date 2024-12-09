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
use common::{clean_nodes, create_parsing_nodes};

#[test]
fn test_simple_flight_adding() {
    assert!(clean_nodes().is_ok());

    let _ = create_parsing_nodes(5, Duration::from_secs(1));

    sleep(Duration::from_secs(1));
    let conn_res = ConnectionHolder::with_cli(Client::default(), "QUORUM");
    assert!(conn_res.is_ok());
    sleep(Duration::from_secs(1));

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
                let keyspace_query = "CREATE KEYSPACE IF NOT EXISTS aerolinea_rustica WITH replication = {'class': 'SimpleStrategy', 'replication_factor' : 3};";
                let keyspace_res = client.send_query(keyspace_query, &mut conn.tls_stream);
                sleep(Duration::from_secs(1));
                assert!(keyspace_res.is_ok());

                let use_query = "USE aerolinea_rustica;";
                let use_res = client.send_query(use_query, &mut conn.tls_stream);
                sleep(Duration::from_secs(1));
                assert!(use_res.is_ok());

                let create_table_query = "CREATE TABLE IF NOT EXISTS vuelos_salientes_en_vivo (id int, orig text, dest text, salida timestamp, pos_lat double, pos_lon double, estado text, velocidad double, altitud double, nivel_combustible double, duracion double, PRIMARY KEY ((orig), id));";
                let table_res = client.send_query(create_table_query, &mut conn.tls_stream);
                sleep(Duration::from_secs(1));
                assert!(table_res.is_ok());

                let select_query = "SELECT * FROM vuelos_salientes_en_vivo;";
                let select_res = client.send_query(select_query, &mut conn.tls_stream);
                assert!(select_res.is_ok());

                if let Ok((protocol_res, _)) = select_res {
                    println!("{:?}", &protocol_res);
                    assert!(matches!(&protocol_res, ProtocolResult::Rows(_)));
                    let live_data_res = LiveFlightData::try_from_protocol_result(
                        protocol_res.clone(),
                        &FlightType::Incoming,
                    );

                    if let ProtocolResult::Rows(_) = protocol_res {
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
    assert!(clean_nodes().is_ok());
}
