use std::collections::HashMap;

use tokio::sync::{Mutex, MutexGuard};

use super::{
    rdb::rdb_loader,
    redis_replica::RedisReplica,
    redis_settings::RedisSettings,
    types::{
        entry_value::EntryValue,
        instance_type::InstanceType,
        stream_key::StreamKey,
        transactions::{ClientId, Transaction},
        value_container::ValueContainer,
    },
};

#[derive(Debug)]
pub struct RedisApp {
    pub(crate) memory: Mutex<HashMap<String, EntryValue>>,
    pub(crate) transactions: Mutex<HashMap<ClientId, Transaction>>,
    pub(crate) settings: RedisSettings,
    pub(crate) _replicas: Mutex<Vec<RedisReplica>>,
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
            _replicas: Mutex::new(Vec::new()),
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
