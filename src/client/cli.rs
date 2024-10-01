//! Módulo del cliente.

use std::io::{BufRead, BufReader, Read, Result, Write};
use std::net::{TcpStream, SocketAddrV4, Ipv4Addr};

/// Corrida de prueba de un cliente.
pub fn run(stream: &mut dyn Read) -> Result<()> {

    let reader = BufReader::new(stream);
    let address = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 8080);
    
    let mut socket = TcpStream::connect(address)?;

    for line in reader.lines().map_while(Result::ok) {
        println!("Enviando: {:?}", line);
        socket.write_all(line.as_bytes())?;
        socket.write_all("\n".as_bytes())?;
    }
    Ok(())
}