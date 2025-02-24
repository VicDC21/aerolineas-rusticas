//! Módulo para tests de conexion entre clientes y nodos.

mod common;

use {
    client::{cli::Client, conn_holder::ConnectionHolder},
    common::{clean_nodes, create_echo_nodes},
    std::{io::Write, thread::sleep, time::Duration},
};

#[test]
fn test_simple_connection() {
    assert!(clean_nodes().is_ok());
    let _ = create_echo_nodes(5, Duration::from_secs(1));

    sleep(Duration::from_secs(10));
    let conn_res = ConnectionHolder::with_cli(Client::default(), "ONE");
    sleep(Duration::from_secs(1));

    assert!(conn_res.is_ok());

    // le damos tiempo para procesar
    sleep(Duration::from_secs(2));

    if let Ok(mut conn) = conn_res {
        let msg = "ping!";
        assert!(conn.tls_stream.write_all(msg.as_bytes()).is_ok());
        assert!(conn.tls_stream.flush().is_ok());

        let client_lock = conn.get_cli();
        if let Ok(mut client) = client_lock.lock() {
            let read_res = client.read_n_bytes(msg.len(), &mut conn.tls_stream, true);
            assert!(read_res.is_ok());
            if let Ok(bytes_read) = read_res {
                assert_eq!(msg.len(), bytes_read.len());
                assert_eq!(msg.as_bytes(), bytes_read);
            }
        };
    }

    assert!(Client::default().send_shutdown().is_ok());
    assert!(clean_nodes().is_ok());
}
