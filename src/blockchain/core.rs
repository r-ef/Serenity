use std::time::{SystemTime, UNIX_EPOCH};
use crate::blockchain::block::Block;
use crate::blockchain::hashing::Hashing;
use crate::blockchain::transaction_pool::TransactionPool;
use crate::blockchain::db::core::Database;
use log::info;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Blockchain {
    pub chain: Vec<Block>,
    pub difficulty: u32,
    pub db: Database,
}

impl Blockchain {
    pub fn new() -> Blockchain {
        let mut blockchain = Blockchain {
            chain: vec![],
            difficulty: 1,
            db: Database::new("database.db"),
        };

        blockchain.load_blocks();
        blockchain
    }

    pub fn calculate_difficulty(&self, chain: &Vec<Block>) -> u32 {
        let mut difficulty = 1;
        let mut last_10_blocks = chain.iter().rev().take(10).cloned().collect::<Vec<_>>();

        while last_10_blocks.len() > 1 {
            let time_diff = last_10_blocks[0].timestamp - last_10_blocks[9].timestamp;

            if time_diff < 60 {
                difficulty += 1;
            } else {
                difficulty -= 1;
            }

            last_10_blocks = last_10_blocks.into_iter().skip(1).collect();
        }

        difficulty
    }

    pub fn mine_block(&mut self, transaction_pool: &mut TransactionPool) {
        let prev_block = self.chain.last().unwrap().clone();
        let transactions = transaction_pool.pool.clone(); // Get transactions from the pool
    
        let mut new_block = Block {
            index: prev_block.index + 1,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            data: transactions.iter().map(|tx| tx.to_string()).collect::<Vec<_>>().join("\n"),
            prev_hash: prev_block.hash.clone(),
            hash: String::new(),
            nonce: 0,
            transactions,
        };
    
        info!("Mining new block...");
    
        let mut hasher = Hashing::new(new_block.clone());
        hasher.mine_block(self.difficulty);
        new_block.hash = hasher.block.hash.clone();
    
        self.db.insert_block(new_block.clone()).unwrap();
    
        self.chain.push(new_block);
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

    pub fn create_genesis_block(&mut self) -> Block {
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let mut genesis_block = Block {
            index: 0,
            timestamp,
            data: "Genesis Block".to_string(),
            prev_hash: "0".to_string(),
            hash: String::new(),
            nonce: 0,
            transactions: vec![],
        };
        genesis_block.hash = Hashing::new(genesis_block.clone()).calculate_hash();

        self.db.insert_block(genesis_block.clone()).unwrap();
        genesis_block
    }

    pub fn add_block(&mut self, mut block: Block) -> bool {
        let prev_block = self.chain.last().unwrap();
        assert_eq!(block.prev_hash, prev_block.hash);

        block.index = prev_block.index + 1;
        block.timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        block.prev_hash = prev_block.hash.clone();

        let mut hasher = Hashing::new(block.clone());
        hasher.mine_block(self.difficulty);
        block.hash = hasher.block.hash.clone();

        self.db.insert_block(block.clone()).unwrap();

        info!("Block mined: {:?}", block);
        self.chain.push(block);
        true
    }

    fn load_blocks(&mut self) {
        let blocks = self.db.get_blocks().unwrap_or_default();
        if blocks.is_empty() {
            let genesis_block = self.create_genesis_block();
            self.chain.push(genesis_block);
        } else {
            self.chain.extend(blocks);
        }
    }
}
