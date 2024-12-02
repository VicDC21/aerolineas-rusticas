//! Módulo para tests de conexion entre clientes y nodos.

mod common;

use std::{
    io::{Read, Write}, thread::sleep, time::Duration
};

use aerolineas_rusticas::{
    client::{cli::Client, conn_holder::ConnectionHolder, protocol_result::ProtocolResult},
    data::flights::{flight::Flight, states::FlightState, types::FlightType}
};
use common::{clean_nodes, create_echo_nodes, create_parsing_nodes};

#[test]
fn test_1_simple_connection() {
    assert!(clean_nodes().is_ok());

    let _ = create_echo_nodes(5, Duration::from_secs(1));

    sleep(Duration::from_secs(10));
    let conn_res = ConnectionHolder::with_cli(Client::default());
    sleep(Duration::from_secs(1));

    assert!(conn_res.is_ok());

    // le damos tiempo para procesar
    sleep(Duration::from_secs(2));

    if let Ok(mut conn) = conn_res {
        let msg = "ping!";
        assert!(conn.tls_stream.write_all(msg.as_bytes()).is_ok());
        assert!(conn.tls_stream.flush().is_ok());

        sleep(Duration::from_secs(5));

        let mut buffer = String::new();
        let read_res = conn.tls_stream.read_to_string(&mut buffer);
        assert!(read_res.is_ok());
        if let Ok(bytes_read) = read_res {
            assert_eq!(msg.len(), bytes_read);
        }

        assert_eq!(msg, buffer);
    }

    assert!(Client::default().send_shutdown().is_ok());
    assert!(clean_nodes().is_ok());
}

#[test]
fn test_2_simple_insert_and_select() {
    assert!(clean_nodes().is_ok());

    let _ = create_parsing_nodes(5, Duration::from_secs(1));

    sleep(Duration::from_secs(1));
    let conn_res = ConnectionHolder::with_cli(Client::default());
    sleep(Duration::from_secs(1));

    assert!(conn_res.is_ok());

    // le damos tiempo para procesar
    sleep(Duration::from_secs(2));

    if let Ok(mut conn) = conn_res {
        let client_lock = conn.get_cli();
        let login_res = conn.login("juan", "1234");
        sleep(Duration::from_secs(1));

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

            let create_table_query = "CREATE TABLE IF NOT EXISTS vuelos_entrantes (id int, orig text, dest text, llegada timestamp, estado text, PRIMARY KEY ((orig), id));";
            let table_res = client.send_query(create_table_query, &mut conn.tls_stream);
            sleep(Duration::from_secs(1));
            assert!(table_res.is_ok());

            let insert_query = "INSERT INTO vuelos_entrantes (id, orig, dest, llegada, estado) VALUES (123456, 'SABE', 'SADL', 12345678, 'in_course');";
            let insert_res = client.send_query(insert_query, &mut conn.tls_stream);
            sleep(Duration::from_secs(1));
            assert!(insert_res.is_ok());

            if let Ok(protocol_res) = insert_res {
                // el resultado de un insert es VOID
                assert!(matches!(protocol_res, ProtocolResult::Void));
            }
            let select_query = "SELECT * FROM vuelos_entrantes;";
            let select_res = client.send_query(select_query, &mut conn.tls_stream);
            sleep(Duration::from_secs(1));
            assert!(select_res.is_ok());

            if let Ok(protocol_res) = select_res {
                assert!(matches!(&protocol_res, ProtocolResult::Rows(_)));
                let flights_res =
                    Flight::try_from_protocol_result(protocol_res.clone(), &FlightType::Incoming);

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
        };
    }

    assert!(Client::default().send_shutdown().is_ok());
    assert!(clean_nodes().is_ok());
}
