use std::sync::{Arc, Mutex};
use serde::Serialize;
use crate::blockchain::transaction::Transaction;
use super::db::core::Database;

#[derive(Debug, Clone, Serialize)]
pub struct TransactionPool {
    pub pool: Vec<Transaction>,
    #[serde(skip)]
    pub db: Arc<Mutex<Database>>,
}

impl TransactionPool {
    pub fn new(db: Arc<Mutex<Database>>) -> TransactionPool {
        TransactionPool {
            pool: vec![],
            db,
        }
    }

    pub fn add_transaction(&mut self, transaction: Transaction) {
        let conn = self.db.lock().unwrap();
        if let Err(e) = conn.insert_transaction(&transaction) {
            debug!("Error inserting transaction: {:?}", e);
        }

        self.pool.push(transaction);
    }

    pub fn remove_transaction(&mut self, transaction: &Transaction) {
        self.pool.retain(|tx| tx != transaction);
    }

    pub fn clear_pool(&mut self) {
        self.pool.clear();
    }

    pub fn get_transactions(&self) -> Vec<Transaction> {
        self.pool.clone()
    }
}
