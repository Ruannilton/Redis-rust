use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

const PONG_RESPONSE: &[u8] = "+PONG\r\n".as_bytes();

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();

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
    let mut client_request_buffer = [0; 512];

    loop {
        let readed_count = stream.read(&mut client_request_buffer).await?;
        if readed_count == 0 {
            break;
        }
        stream.write(PONG_RESPONSE).await?;
    }
    Ok(())
}
