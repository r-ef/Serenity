use std::time::{SystemTime, UNIX_EPOCH};
use rand::Rng;
use serde::{Serialize, Deserialize};

use super::transaction::Transaction;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub index: u32,
    pub timestamp: u64,
    pub data: String,
    pub prev_hash: String,
    pub hash: String,
    pub nonce: u64,
    pub transactions: Vec<Transaction>,
}

impl Block {
    pub fn new(index: u32, data: String, prev_hash: String) -> Block {
        let mut rng = rand::thread_rng();
        let start_nonce = rng.gen_range(0..u64::MAX);
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        Block {
            index,
            timestamp,
            data,
            prev_hash,
            hash: String::new(),
            nonce: start_nonce,
            transactions: vec![],
        }
    }
}