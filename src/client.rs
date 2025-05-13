use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;

pub fn send_command(command: &str) {
    if let Ok(mut stream) = TcpStream::connect("127.0.0.1:7878") {
        stream.write_all(format!("{}\n", command).as_bytes()).unwrap();

        let mut reader = BufReader::new(stream);
        let mut response = String::new();
        reader.read_line(&mut response).unwrap();
        println!("{response}");
    } else {
        eprintln!("❌ No se pudo conectar al servidor Pomodoro");
    }
}
