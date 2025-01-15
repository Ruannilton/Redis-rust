use std::collections::{HashMap, VecDeque};

use crate::resp_desserializer::RespTk;

pub type Transaction = VecDeque<RespTk>;
pub type ClientId = u64;

#[derive(Debug)]
pub struct TransactionMap {
    map: HashMap<ClientId, Transaction>,
}

impl TransactionMap {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn begin(&mut self, id: ClientId) {
        if let None = self.map.get(&id) {
            self.map.insert(id, VecDeque::new());
        }
    }

    pub fn push(&mut self, id: ClientId, command: &RespTk) {
        if let Some(tx) = self.map.get_mut(&id) {
            tx.push_back(command.clone());
        } else {
            let mut tx = VecDeque::<RespTk>::new();
            tx.push_back(command.clone());
            self.map.insert(id, tx);
        }
    }

    pub fn get(&self, id: ClientId) -> Option<&Transaction> {
        if let Some(tx) = self.map.get(&id) {
            return Some(tx as &Transaction);
        }
        None
    }

    pub fn discard(&mut self, id: ClientId) {
        if let Some(_) = self.map.get_mut(&id) {
            self.map.remove(&id);
        }
    }
}
