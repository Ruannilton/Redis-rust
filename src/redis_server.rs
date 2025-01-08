use std::sync::Arc;

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

use crate::{
    redis::{redis_app::RedisApp, redis_parser},
    resp::resp_desserializer::{self},
};
pub struct RedisServer {
    listener: TcpListener,
    app: Arc<RedisApp>,
    connection_counter: u64,
}

impl RedisServer {
    pub async fn new(address: &str, redis_instance: Arc<RedisApp>) -> Result<Self, std::io::Error> {
        let addr = format!("{}:{}", address, redis_instance.settings.port);
        let listener = TcpListener::bind(addr).await?;

        Ok(RedisServer {
            listener,
            app: redis_instance,
            connection_counter: 0,
        })
    }

    pub async fn run(mut self) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            if let Ok((stream, _)) = self.listener.accept().await {
                let cli = Arc::clone(&self.app);
                self.connection_counter += 1;
                let conn_id = self.connection_counter;
                tokio::spawn(async move {
                    match Self::handle_request(cli, stream, conn_id).await {
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
        request_id: u64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            let mut buffer = [0; 1024]; // Um buffer de tamanho fixo para leitura

            let readed = stream.read(&mut buffer).await?;

            if readed == 0 {
                break;
            }

            let readed_buffer = &buffer[..readed];

            println!("Received: {:?}", std::str::from_utf8(readed_buffer)?);

            let tokens = resp_desserializer::desserialize(readed_buffer)?;
            let mut tokens_iter = tokens.iter().peekable();
            let commands = redis_parser::parse_token_int_command(&mut tokens_iter)?;

            for command in commands {
                let result = cli.execute_command(request_id, command).await?;
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
