use std::{
    collections::HashMap,
    time::{SystemTime, UNIX_EPOCH},
};

use super::types::instance_type::InstanceType;

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
    pub fn new() -> Self {
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
