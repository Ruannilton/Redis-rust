use std::sync::Arc;

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

use crate::{redis_app::RedisApp, redis_parser, resp_desserializer::RespDesserializer};
pub struct RedisServer {
    listener: TcpListener,
    app: Arc<RedisApp>,
}

impl RedisServer {
    pub async fn new(address: &str, redis_instance: Arc<RedisApp>) -> Result<Self, std::io::Error> {
        let listener = TcpListener::bind(address).await?;

        Ok(RedisServer {
            listener,
            app: redis_instance,
        })
    }

    pub async fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            if let Ok((stream, _)) = self.listener.accept().await {
                let cli = Arc::clone(&self.app);
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

            let readed = stream.read(&mut buffer).await?;

            if readed == 0 {
                break;
            }

            let readed_buffer = &buffer[..readed];

            println!("Received: {:?}", std::str::from_utf8(readed_buffer)?);

            let tokens = RespDesserializer::desserialize(readed_buffer)?;
            let mut tokens_iter = tokens.iter().peekable();
            let commands = redis_parser::parse_token_int_command(&mut tokens_iter)?;

            for command in commands {
                let result = cli.execute_command(command).await?;
                let response = result.into_bytes();

                stream.write(response.as_slice()).await?;

                println!(
                    "Returned: {:?}",
                    std::str::from_utf8(response.clone().as_slice())?
                );
            }
        }
        Ok(())
    }
}
