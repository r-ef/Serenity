use std::time::{SystemTime, UNIX_EPOCH};
use crate::blockchain::block::Block;
use crate::blockchain::hashing::Hashing;
use crate::blockchain::transaction::Transaction;
use crate::blockchain::transaction_pool::TransactionPool;
use crate::utils::calculations;
use log::{debug, info};
use serde::Serialize;

use super::db::mongodb::core::MongoDB;

#[derive(Debug, Clone, Serialize)]
pub struct Blockchain {
    pub chain: Vec<Block>,
    pub difficulty: u32,
    pub db: MongoDB,
}

impl Blockchain {
    pub async fn new(db: MongoDB) -> Blockchain {
        let chain = db.get_blocks().await.unwrap_or_default();
        let mut blockchain = Blockchain {
            chain: chain.clone(),
            difficulty: calculations::calculate_difficulty(&chain),
            db: db
        };

        blockchain.load_blocks().await;
        blockchain
    }


    pub async fn mine_block(&mut self, transaction_pool: &mut TransactionPool, miner_address: &str) {
        let prev_block = self.chain.last().unwrap().clone();
        let mut transactions = transaction_pool.pool.clone();

        let amount = calculations::calculate_mining_reward(self.chain.len() as u64, &transaction_pool);
    
        let reward_transaction = Transaction {
            sender: "block_reward".to_string(),
            receiver: miner_address.to_string(),
            amount: amount,
            fee: calculations::calculate_fee(amount),
            timestamp: chrono::Utc::now().timestamp() as u64,
        };
        println!("Reward transaction: {:?}", reward_transaction);
    
        transactions.push(reward_transaction.clone());
    
        let mut new_block = Block {
            index: prev_block.index + 1,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            data: transactions.iter().map(|tx| tx.to_string()).collect::<Vec<_>>().join("\n"),
            prev_hash: prev_block.hash.clone(),
            hash: String::new(),
            nonce: 0,
            transactions,
            difficulty: self.difficulty,
        };
    
        info!("Mining new block...");
    
        let mut hasher = Hashing::new(new_block.clone());
        hasher.mine_block(self.difficulty);
        new_block.hash = hasher.block.hash.clone();
        self.db.insert_block(new_block.clone()).await.expect("Failed to insert block into database");
        self.chain.push(new_block);
        let reward_amount = reward_transaction.clone().amount;
        let amount = self.db.get_balance(miner_address).await.unwrap_or(0.0);
        self.db.update_balance(miner_address, amount + reward_amount).await.expect("Failed to update balance");
        self.db.insert_transaction(&reward_transaction).await.expect("Failed to insert transaction into database");
        debug!("Reward transaction: {:?}", reward_transaction);
        transaction_pool.clear_pool();
        info!("Block mined and added to the chain");
    
        if self.chain.len() % 10 == 0 { 
            self.adjust_difficulty();
        }
    }
    

    pub fn adjust_difficulty(&mut self) {
        let last_block = self.chain.last().unwrap();
        let prev_block = self.chain.get(self.chain.len() - 10).unwrap();
        let time_diff = last_block.timestamp - prev_block.timestamp;

        if time_diff < 60 {
            self.difficulty += 1;
        } else {
            self.difficulty -= 1;
        }

        info!("Adjusted difficulty to: {}", self.difficulty);
    }

    pub async fn create_genesis_block(&mut self) -> Block {
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let mut genesis_block = Block {
            index: 0,
            timestamp,
            data: "Genesis Block".to_string(),
            prev_hash: "0".to_string(),
            hash: String::new(),
            nonce: 0,
            transactions: vec![],
            difficulty: self.difficulty,
        };
        genesis_block.hash = Hashing::new(genesis_block.clone()).calculate_hash();

        self.db.insert_block(genesis_block.clone()).await.unwrap();
        genesis_block
    }

    pub async fn add_block(&mut self, mut block: Block) -> bool {
        let prev_block = self.chain.last().unwrap();
        assert_eq!(block.prev_hash, prev_block.hash);

        block.index = prev_block.index + 1;
        block.timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        block.prev_hash = prev_block.hash.clone();

        let mut hasher = Hashing::new(block.clone());
        hasher.mine_block(self.difficulty);
        block.hash = hasher.block.hash.clone();

        self.db.insert_block(block.clone()).await.unwrap();

        info!("Block mined: {:?}", block);
        self.chain.push(block);
        true
    }

    pub async fn load_blocks(&mut self) {
        let blocks = self.db.get_blocks().await.unwrap();
        if blocks.is_empty() {
            let genesis_block = self.create_genesis_block().await;
            self.chain.push(genesis_block);
        } else {
            self.chain.extend(blocks);
        }
    }
}
