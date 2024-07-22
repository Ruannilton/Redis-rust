pub mod resp_decoder;
pub mod resp_encoder;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

const PONG_RESPONSE: &[u8] = "+PONG\r\n".as_bytes();

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();
    println!("Running...");
    loop {
        let accepted_result = listener.accept().await;

        match accepted_result {
            Ok((stream, _)) => {
                tokio::spawn(async move {
                    handle_client_connection(stream).await.unwrap();
                });
            }
            Err(_) => {
                print!("Failed to handle request")
            }
        }
    }
}

async fn handle_client_connection(mut stream: TcpStream) -> Result<(), Box<dyn std::error::Error>> {
    let mut client_request_buffer = Vec::new();

    loop {
        let readed_count = stream.read_to_end(&mut client_request_buffer).await?;
        if readed_count == 0 {
            break;
        }

        let buffer = client_request_buffer.as_slice();
        if let Some(command) = resp_decoder::BufferDecoder::decode(buffer) {
            match command.command.as_str() {
                "PING" => {
                    stream.write(PONG_RESPONSE).await?;
                }
                "ECHO" => {
                    if let Some(args) = command.args {
                        let arg = args[0].clone();
                        let resp = resp_encoder::resp_encode_bulk_string(arg);
                        stream.write(resp.as_bytes()).await?;
                    }
                }
                _ => {}
            }
        }
    }
    Ok(())
}
