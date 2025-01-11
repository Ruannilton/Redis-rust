#[derive(Debug)]
pub struct RedisReplica {
    _address: String,
    _port: String,
}

impl RedisReplica {
    pub fn new(address: String, port: String) -> Self {
        Self {
            _address: address,
            _port: port,
        }
    }
}
