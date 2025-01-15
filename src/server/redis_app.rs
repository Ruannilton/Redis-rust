use std::{collections::HashMap, sync::Arc};

use tokio::{
    io::AsyncWriteExt,
    net::TcpStream,
    sync::{Mutex, MutexGuard},
};

use crate::{
    rdb::rdb_loader,
    resp_desserializer::RespTk,
    types::{
        entry_value::EntryValue, instance_type::InstanceType, redis_replica::RedisReplica,
        redis_settings::RedisSettings, stream_key::StreamKey, transactions::TransactionMap,
        value_container::ValueContainer,
    },
    utils,
};

#[derive(Debug)]
pub struct RedisApp {
    pub memory: Mutex<HashMap<String, EntryValue>>,
    pub transactions: Mutex<TransactionMap>,
    pub settings: RedisSettings,
    pub replicas: Mutex<Vec<RedisReplica>>,
    pub deferred_actions: Mutex<HashMap<u64, fn(app: Arc<RedisApp>) -> String>>,
}

impl RedisApp {
    pub fn new(args: impl Iterator<Item = String>) -> Self {
        let mut settings = RedisSettings::new();

        Self::load_settings_from_args(args, &mut settings);

        let db = Self::init_database(&settings);

        RedisApp {
            memory: Mutex::new(db),
            transactions: Mutex::new(TransactionMap::new()),
            settings: settings,
            replicas: Mutex::new(Vec::new()),
            deferred_actions: Mutex::new(HashMap::new()),
        }
    }

    pub fn get_istance_type(&self) -> InstanceType {
        self.settings.instance_type
    }

    pub fn get_master_conn(&self) -> Option<String> {
        if let Some(replicaof) = &self.settings.replica_of {
            let addvars: Vec<&str> = replicaof.split(' ').collect();
            let master_address = format!("{}:{}", addvars[0], addvars[1]);
            return Some(master_address);
        }
        None
    }

    pub async fn get_entry(&self, key: &String) -> Option<ValueContainer> {
        let mem = self.memory.lock().await;

        return mem.get(key).and_then(|container| container.get_value());
    }

    pub async fn put_entry(&self, key: String, value: ValueContainer, exp: Option<u128>) {
        let mut mem = self.memory.lock().await;

        let expires: Option<u128> = match exp {
            Some(ex) => Some(utils::get_current_time_ms() + ex),
            None => None,
        };

        let entry = EntryValue {
            value: value,
            expires_at: expires,
        };

        _ = mem.insert(key, entry);
    }

    pub async fn broadcast_command(&self, _cmd: &RespTk) {
        let replicas = self.replicas.lock().await;
        for replica in replicas.iter() {
            let replica_addr = replica.get_address();
            if let Ok(mut stream) = TcpStream::connect(replica_addr).await {
                let cmd_resp: String = _cmd.into();
                let bytes = cmd_resp.into_bytes();
                let _ = stream.write_all(&bytes).await;
            }
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
