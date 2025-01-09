use std::sync::Arc;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

use crate::{
    redis::{redis_app::RedisApp, redis_parser, types::command_token::CommandToken},
    resp::{
        resp_desserializer::{self},
        resp_serializer,
    },
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
        if let Some(replicaof) = &self.app.settings.replica_of {
            let addvars: Vec<&str> = replicaof.split(' ').collect();
            let master_address = format!("{}:{}", addvars[0], addvars[1]);

            let mut stream = TcpStream::connect(master_address).await?;
            let mut buffer = [0; 1024];

            let ping_payload = resp_serializer::to_resp_array(vec!["PING".into()]).into_bytes();
            stream.write_all(&ping_payload).await?;
            _ = stream.read(&mut buffer).await?;

            let replconf_payload = resp_serializer::to_resp_array(vec![
                "REPLCONF".into(),
                "listening-port".into(),
                self.app.settings.port.to_string(),
            ])
            .into_bytes();
            stream.write_all(&replconf_payload).await?;
            _ = stream.read(&mut buffer).await?;

            let replconf2_payload = resp_serializer::to_resp_array(vec![
                "REPLCONF".into(),
                "capa".into(),
                "psync2".into(),
            ])
            .into_bytes();
            stream.write_all(&replconf2_payload).await?;
            _ = stream.read(&mut buffer).await?;

            let psync_payload =
                resp_serializer::to_resp_array(vec!["PSYNC".into(), "?".into(), "-1".into()])
                    .into_bytes();

            stream.write_all(&psync_payload).await?;
            _ = stream.read(&mut buffer).await?;
        }

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
                let result = cli.execute_command(request_id, command.clone()).await?;
                let response = result.into_bytes();

                stream.write(response.as_slice()).await?;

                match command {
                    CommandToken::Psync(_, _) => {
                        let base64_file = "UkVESVMwMDEx+glyZWRpcy12ZXIFNy4yLjD6CnJlZGlzLWJpdHPAQPoFY3RpbWXCbQi8ZfoIdXNlZC1tZW3CsMQQAPoIYW9mLWJhc2XAAP/wbjv+wP9aog==";
                        let bin_file = base64_to_bin(base64_file)?;
                        let response = format!("${}\r\n", bin_file.len());

                        stream.write(response.as_bytes()).await?;
                        stream.write(bin_file.as_slice()).await?;
                    }
                    _ => {}
                }

                println!(
                    "Returned: {:?}",
                    std::str::from_utf8(response.clone().as_slice())?
                );
            }
        }
        Ok(())
    }
}

fn base64_to_bin(encoded: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    const BASE64_CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    let mut decoded = Vec::new();
    let mut buffer = 0u32;
    let mut bits_collected = 0;

    for byte in encoded.bytes() {
        if byte == b'=' {
            break;
        }

        let value = BASE64_CHARS
            .iter()
            .position(|&c| c == byte)
            .ok_or("Invalid base64 character")? as u32;
        buffer = (buffer << 6) | value;
        bits_collected += 6;

        if bits_collected >= 8 {
            bits_collected -= 8;
            decoded.push((buffer >> bits_collected) as u8);
            buffer &= (1 << bits_collected) - 1;
        }
    }

    Ok(decoded)
}
