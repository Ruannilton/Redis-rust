pub mod resp_serialization;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();

    loop {
        let accepted_result = listener.accept().await;

        if let Ok((stream, _)) = accepted_result {
            tokio::spawn(async move {
                handle_client_connection(stream).await.unwrap();
            });
        }
    }
}

fn process_commands(buffer: &[u8]) -> String {
    let command = resp_serialization::get_redis_command(buffer);

    let response = match command {
        resp_serialization::RedisCommand::Ping => String::from("PONG"),
        resp_serialization::RedisCommand::Echo(s) => s,
        _ => String::from("INVALID"),
    };

    let encoded = resp_serialization::encode_as_bulk_string(response);
    encoded
}

async fn handle_client_connection(mut stream: TcpStream) -> Result<(), Box<dyn std::error::Error>> {
    let mut client_request_buffer = Vec::new();
    println!("here");
    loop {
        let readed_count = stream.read_buf(&mut client_request_buffer).await.unwrap();

        if readed_count == 0 {
            break;
        }

        println!("Readed: {}", readed_count);
        println!("Payload: {:?}", client_request_buffer);

        let buffer = client_request_buffer.as_slice();

        let response = process_commands(buffer);

        stream.write(response.as_bytes()).await?;
    }
    Ok(())
}
