//! Módulo de servidor.

use std::io::Result;
use std::io::{BufRead, BufReader, Read};
use std::net::{Ipv4Addr, SocketAddrV4, TcpListener};

/// Corrida de prueba para un servidor.
pub fn run() -> Result<()> {
    let socket_addr = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 8080);
    let listener = TcpListener::bind(socket_addr)?;

    // bloquea hasta que le entra un request
    let (mut client_stream, socket_addr) = listener.accept()?;

    println!("La socket addr del client: {:?}", socket_addr);
    handle_client(&mut client_stream)?;
    Ok(())
}

fn handle_client(stream: &mut dyn Read) -> std::io::Result<()> {
    let reader = BufReader::new(stream);
    let mut lines = reader.lines();
    // iteramos las lineas que recibimos de nuestro cliente
    while let Some(Ok(line)) = lines.next() {
        println!("Recibido: {:?}", line);
    }
    Ok(())
}
