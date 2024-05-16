use std::{io::Write, net::TcpListener};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();

    for mut stream in listener.incoming() {
        match stream {
            Ok(ref mut stream) => {
                let response = "+PONG\r\n".as_bytes();
                stream.write(response).unwrap();
                stream.write(response).unwrap();
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
