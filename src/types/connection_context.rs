#[derive(Clone)]
pub struct ConnectionContext {
    pub connection_id: u64,
    pub client_address: String,
}

impl ConnectionContext {
    pub fn new(connection_id: u64, client_address: String) -> Self {
        Self {
            connection_id,
            client_address,
        }
    }
}
