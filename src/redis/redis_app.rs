use std::collections::HashMap;

use tokio::sync::{Mutex, MutexGuard};

use crate::resp::resp_serializer::to_resp_string;

use super::{
    command::command_executor,
    rdb::rdb_loader,
    redis_error::RedisError,
    types::{
        command_token::CommandToken,
        entry_value::EntryValue,
        stream_key::StreamKey,
        transactions::{ClientId, Transaction},
        value_container::ValueContainer,
    },
};

#[derive(Debug)]
pub struct RedisApp {
    pub(crate) memory: Mutex<HashMap<String, EntryValue>>,
    pub(crate) configurations: Mutex<HashMap<String, String>>,
    pub(crate) transactions: Mutex<HashMap<ClientId, Transaction>>,
}

impl RedisApp {
    pub fn new(args: impl Iterator<Item = String>) -> Self {
        let config_map = Self::parse_args(args);

        let db = Self::init_database(&config_map);

        RedisApp {
            memory: Mutex::new(db),
            configurations: Mutex::new(config_map),
            transactions: Mutex::new(HashMap::new()),
        }
    }

    fn restore_from_rdb(dir: &String, file: &String) -> HashMap<String, EntryValue> {
        match rdb_loader::load(dir, file) {
            Ok(database) => database,
            Err(err) => {
                println!("Failed to restore from RDB: {}", err);
                HashMap::new()
            }
        }
    }

    fn init_database(config_map: &HashMap<String, String>) -> HashMap<String, EntryValue> {
        let db = if let (Some(dir), Some(file)) =
            (config_map.get("dir"), config_map.get("dbfilename"))
        {
            Self::restore_from_rdb(dir, file)
        } else {
            HashMap::new()
        };
        db
    }

    fn parse_args(mut args: impl Iterator<Item = String>) -> HashMap<String, String> {
        let mut configs = HashMap::new();

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--dir" => {
                    if let Some(dir_value) = args.next() {
                        _ = configs.insert("dir".to_owned(), dir_value);
                    }
                }
                "--dbfilename" => {
                    if let Some(filename_value) = args.next() {
                        _ = configs.insert("dbfilename".to_owned(), filename_value);
                    }
                }
                _ => {}
            }
        }

        configs
    }

    pub async fn execute_command(
        &self,
        id: ClientId,
        cmd: CommandToken,
    ) -> Result<String, RedisError> {
        match cmd {
            CommandToken::Exec => {}
            _ => {
                let mut transacs = self.transactions.lock().await;
                if let Some(commands) = transacs.get_mut(&id) {
                    commands.push_back(cmd);
                    return Ok(to_resp_string("QUEUED".into()));
                }
            }
        }
        command_executor::execute_command(&self, id, cmd).await
    }

    pub(crate) fn get_last_stream_key(
        &self,
        stream_key: &str,
        mem: &MutexGuard<HashMap<String, EntryValue>>,
    ) -> Option<StreamKey> {
        let entry = mem.get(stream_key)?;

        if let ValueContainer::Stream(stream) = &entry.value {
            let last = stream.last()?;
            Some(last.id.clone())
        } else {
            None
        }
    }
}
