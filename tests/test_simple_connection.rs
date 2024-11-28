//! Módulo para tests de conexion simple.

mod common;

use std::io::{Read, Write};

use aerolineas_rusticas::client::cli::Client;
use common::init_graph_echo;

#[test]
fn test_1_simple_connection() {
    let graph_handle = init_graph_echo();
    let client = Client::default();

    let con_res = client.connect();
    assert!(con_res.is_ok());

    if let Ok(mut tcp_stream) = con_res {
        let msg = "ping!";
        assert!(tcp_stream.write_all(msg.as_bytes()).is_ok());
        assert!(tcp_stream.flush().is_ok());

        let mut buffer = String::new();
        let read_res = tcp_stream.read_to_string(&mut buffer);
        assert!(read_res.is_ok());
        if let Ok(bytes_read) = read_res {
            assert_eq!(msg.len(), bytes_read);
        }

        assert_eq!(msg, buffer);
    }

    assert!(client.send_shutdown().is_ok());
    assert!(graph_handle.join().is_ok());
}
