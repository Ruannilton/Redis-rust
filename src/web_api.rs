use std::sync::Arc;

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

use crate::{input_parser, redis_cli::RedisApp};
pub struct RedisServer {
    listener: TcpListener,
    cli: Arc<RedisApp>,
}

impl RedisServer {
    pub async fn new(address: &str, redis_instance: Arc<RedisApp>) -> Result<Self, std::io::Error> {
        let listener = TcpListener::bind(address).await?;

        Ok(RedisServer {
            listener,
            cli: redis_instance,
        })
    }

    pub async fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            if let Ok((stream, _)) = self.listener.accept().await {
                let cli = Arc::clone(&self.cli);
                tokio::spawn(async move {
                    match Self::handle_request(cli, stream).await {
                        Ok(_) => {}
                        Err(err) => println!("{:?}", err),
                    }
                });
            }
        }
    }

    async fn handle_request(
        cli: Arc<RedisApp>,
        mut stream: TcpStream,
    ) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            let mut buffer = [0; 1024]; // Um buffer de tamanho fixo para leitura
            let mut payload_buffer = Vec::new();
            let readed = stream.read(&mut buffer).await?;

            if readed == 0 {
                break;
            }

            payload_buffer.extend_from_slice(&buffer[..readed]);

            println!(
                "Received: {:?}",
                std::str::from_utf8(payload_buffer.clone().as_slice())?
            );

            let command = input_parser::desserialize(payload_buffer)?;
            let result = cli.execute_command(command)?;
            let response = result.into_bytes();

            stream.write(response.as_slice()).await?;

            println!(
                "Returned: {:?}",
                std::str::from_utf8(response.clone().as_slice())?
            );
        }
        Ok(())
    }
}
