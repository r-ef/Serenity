use rayon::iter::IntoParallelIterator;
use serde::Serialize;
use crate::blockchain::transaction::Transaction;
use super::db::mongodb::core::MongoDB;

#[derive(Debug, Clone, Serialize)]
pub struct TransactionPool {
    pub pool: Vec<Transaction>,
    #[serde(skip)]
    pub db: MongoDB
}

impl IntoParallelIterator for TransactionPool {

    type Item = Transaction;

    type Iter = rayon::vec::IntoIter<Self::Item>;

    fn into_par_iter(self) -> Self::Iter {
        self.pool.into_par_iter()
    }
}

impl TransactionPool {
    pub fn new(db: MongoDB) -> TransactionPool {
        TransactionPool {
            pool: vec![],
            db,
        }
    }

    pub async fn add_transaction(&mut self, transaction: Transaction) {
        let _ = self.db.insert_transaction(&transaction).await;
        self.pool.push(transaction);
    }

    pub fn clear_pool(&mut self) {
        self.pool.clear();
    }
}
