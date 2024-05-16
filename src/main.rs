use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    thread,
};

const PONG_RESPONSE: &[u8] = "+PONG\r\n".as_bytes();

fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(move || {
                    handle_client_connection(stream);
                });
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

fn handle_client_connection(stream: TcpStream) {
    let mut stream_copy = stream.try_clone().unwrap();
    let mut client_request_buffer = [0; 512];

    loop {
        let readed_count = stream_copy.read(&mut client_request_buffer).unwrap();
        if readed_count == 0 {
            break;
        }
        stream_copy.write(PONG_RESPONSE).unwrap();
    }
}
