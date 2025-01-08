use std::{
    collections::HashMap,
    time::{SystemTime, UNIX_EPOCH},
};

use tokio::sync::{Mutex, MutexGuard};

use crate::resp::resp_serializer::to_resp_string;

use super::{
    command::command_executor,
    rdb::rdb_loader,
    redis_error::RedisError,
    types::{
        command_token::CommandToken,
        entry_value::EntryValue,
        instance_type::InstanceType,
        stream_key::StreamKey,
        transactions::{ClientId, Transaction},
        value_container::ValueContainer,
    },
};

#[derive(Debug)]
pub struct RedisSettings {
    pub(crate) dir: Option<String>,
    pub(crate) db_file_name: Option<String>,
    pub(crate) port: String,
    pub(crate) replica_of: Option<String>,
    pub(crate) instance_type: InstanceType,
    pub(crate) master_replid: Option<String>,
    pub(crate) master_repl_offset: u64,
}

impl RedisSettings {
    fn new() -> Self {
        let rand_string = Self::generate_random_string(40);

        RedisSettings {
            db_file_name: None,
            replica_of: None,
            dir: None,
            instance_type: InstanceType::Master,
            master_repl_offset: 0,
            master_replid: Some(rand_string),
            port: "6379".into(),
        }
    }

    fn generate_random_string(length: usize) -> String {
        let charset: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                               abcdefghijklmnopqrstuvwxyz\
                               0123456789";
        let charset_len = charset.len();
        let mut random_string = String::with_capacity(length);

        // Use the current system time as a source of "randomness"
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_nanos();

        let mut hash = now;
        for _ in 0..length {
            let index = (hash % charset_len as u128) as usize;
            random_string.push(charset[index] as char);

            // Update hash to get a new "random" value
            hash /= charset_len as u128;
            if hash == 0 {
                hash = now ^ charset_len as u128;
            }
        }

        random_string
    }
    pub fn to_hashmap(&self) -> HashMap<&str, String> {
        let mut hash = HashMap::new();

        if let Some(dir) = &self.dir {
            hash.insert("dir", dir.into());
        }

        if let Some(dbfilename) = &self.db_file_name {
            hash.insert("dbfilename", dbfilename.into());
        }

        if let Some(replicaof) = &self.replica_of {
            hash.insert("replicaof", replicaof.into());
        }

        if let Some(master_replid) = &self.master_replid {
            hash.insert("master_replid", master_replid.into());
        }

        hash.insert("master_repl_offset", self.master_repl_offset.to_string());

        hash
    }
}

#[derive(Debug)]
pub struct RedisApp {
    pub(crate) memory: Mutex<HashMap<String, EntryValue>>,
    pub(crate) transactions: Mutex<HashMap<ClientId, Transaction>>,
    pub(crate) settings: RedisSettings,
}

impl RedisApp {
    pub fn new(args: impl Iterator<Item = String>) -> Self {
        let mut settings = RedisSettings::new();

        Self::load_settings_from_args(args, &mut settings);

        let db = Self::init_database(&settings);

        RedisApp {
            memory: Mutex::new(db),
            transactions: Mutex::new(HashMap::new()),
            settings: settings,
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

    fn init_database(settings: &RedisSettings) -> HashMap<String, EntryValue> {
        let db = if let (Some(dir), Some(file)) = (&settings.dir, &settings.db_file_name) {
            Self::restore_from_rdb(dir, file)
        } else {
            HashMap::new()
        };
        db
    }

    fn load_settings_from_args(
        mut args: impl Iterator<Item = String>,
        settings: &mut RedisSettings,
    ) {
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--dir" => {
                    if let Some(dir_value) = args.next() {
                        settings.dir = Some(dir_value)
                    }
                }
                "--dbfilename" => {
                    if let Some(filename_value) = args.next() {
                        settings.db_file_name = Some(filename_value)
                    }
                }
                "--port" => {
                    if let Some(port_value) = args.next() {
                        settings.port = port_value
                    }
                }
                "--replicaof" => {
                    if let Some(replica_value) = args.next() {
                        settings.replica_of = Some(replica_value);
                        settings.instance_type = InstanceType::Slave
                    }
                }
                _ => {}
            }
        }
    }

    pub async fn execute_command(
        &self,
        id: ClientId,
        cmd: CommandToken,
    ) -> Result<String, RedisError> {
        match cmd {
            CommandToken::Exec => {}
            CommandToken::Discard => {}
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
