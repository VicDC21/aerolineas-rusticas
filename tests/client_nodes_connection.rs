//! Módulo para tests de conexion entre clientes y nodos.

mod common;

use std::{
    io::{Read, Write},
    thread::sleep,
    time::Duration,
};

use aerolineas_rusticas::{
    client::{cli::Client, conn_holder::ConnectionHolder, protocol_result::ProtocolResult},
    data::flights::{flight::Flight, states::FlightState, types::FlightType},
};
use common::{clean_nodes, init_graph_echo, init_graph_parsing};

#[test]
fn test_1_simple_connection() {
    assert!(clean_nodes().is_ok());

    let graph_handle = init_graph_echo();
    let conn_res = ConnectionHolder::with_cli(Client::default());
    assert!(conn_res.is_ok());

    // le damos tiempo para procesar
    sleep(Duration::from_secs(5));

    if let Ok(mut conn) = conn_res {
        println!("----------------------- ANTES");
        let tls_res = conn.get_tls_and_login(&"juan".to_string(), &"1234".to_string());
        println!("----------------------- DESPUÉS");
        if let Ok(mut tls_stream) = tls_res {
            let msg = "ping!";
            assert!(tls_stream.write_all(msg.as_bytes()).is_ok());
            assert!(tls_stream.flush().is_ok());

            sleep(Duration::from_secs(5));

            let mut buffer = String::new();
            let read_res = tls_stream.read_to_string(&mut buffer);
            assert!(read_res.is_ok());
            if let Ok(bytes_read) = read_res {
                assert_eq!(msg.len(), bytes_read);
            }

            assert_eq!(msg, buffer);
        }
    }

    assert!(Client::default().send_shutdown().is_ok());
    assert!(graph_handle.join().is_ok());
    assert!(clean_nodes().is_ok());
}

#[test]
fn test_2_simple_insert_and_select() {
    assert!(clean_nodes().is_ok());

    let graph_handle = init_graph_parsing();
    let conn_res = ConnectionHolder::with_cli(Client::default());
    assert!(conn_res.is_ok());

    // le damos tiempo para procesar
    sleep(Duration::from_secs(5));

    if let Ok(mut conn) = conn_res {
        let client_lock = conn.get_cli();
        let tls_res = conn.get_tls_and_login(&"juan".to_string(), &"1234".to_string());
        if let Ok(mut tls_stream) = tls_res {
            if let Ok(mut client) = client_lock.lock() {
                let insert_query = "INSERT INTO vuelos_entrantes (id, orig, dest, llegada, estado) VALUES (123456, 'SABE', 'SADL', 12345678, 'in_course');";

                let insert_res = client.send_query(insert_query, &mut tls_stream);
                assert!(insert_res.is_ok());
                if let Ok(protocol_res) = insert_res {
                    // el resultado de un insert es VOID
                    assert!(matches!(protocol_res, ProtocolResult::Void));
                }

                sleep(Duration::from_secs(5));

                let select_query = "SELECT * FROM vuelos_entrantes;";
                let select_res = client.send_query(select_query, &mut tls_stream);
                assert!(select_res.is_ok());
                if let Ok(protocol_res) = select_res {
                    assert!(matches!(&protocol_res, ProtocolResult::Rows(_)));
                    let flights_res = Flight::try_from_protocol_result(
                        protocol_res.clone(),
                        &FlightType::Incoming,
                    );

                    if let ProtocolResult::Rows(rows) = protocol_res {
                        assert_eq!(rows.len(), 1);

                        assert!(flights_res.is_ok());
                        if let Ok(flights) = flights_res {
                            assert_eq!(flights.len(), 1);
                            let flight = &flights[0];

                            assert_eq!(flight.id, 123456);
                            assert_eq!(flight.orig, "SABE".to_string());
                            assert_eq!(flight.dest, "SADL".to_string());
                            assert_eq!(flight.arrival(), 12345678);
                            assert!(matches!(flight.state, FlightState::InCourse));
                            assert!(matches!(flight.flight_type, FlightType::Incoming));
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
