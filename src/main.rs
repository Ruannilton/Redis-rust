use std::{
    io::{Read, Write},
    net::TcpListener,
};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();

    for mut stream in listener.incoming() {
        match stream {
            Ok(ref mut stream) => {
                let response = "+PONG\r\n".as_bytes();
                let mut buf = [0; 512];
                loop {
                    let readed_count = stream.read(&mut buf).unwrap();
                    if readed_count == 0 {
                        break;
                    }
                    stream.write(response).unwrap();
                }
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
