use sha2::{Sha256, Digest};
use crate::blockchain::block::Block;
use log::{info, debug};

pub struct Hashing {
    pub block: Block,
}

impl Hashing {
    pub fn new(block: Block) -> Hashing {
        Hashing {
            block,
        }
    }

    pub fn calculate_hash(&self) -> String {
        let data = format!(
            "{}{}{}{}{}",
            self.block.index, self.block.timestamp, self.block.data, self.block.prev_hash, self.block.nonce
        );
        let mut hasher = Sha256::new();
        hasher.update(data.as_bytes());
        let result = hasher.finalize();
        hex::encode(result)
    }

    pub fn mine_block(&mut self, difficulty: u32) {
        let prefix = "0".repeat(difficulty as usize);
        info!("Starting to mine block with difficulty: {}", difficulty);

        while !self.block.hash.starts_with(&prefix) {
            self.block.nonce += 1;
            self.block.hash = self.calculate_hash();
            debug!("Nonce: {}, Hash: {}", self.block.nonce, self.block.hash);
        }

        info!("Block mined: {:?}", self.block);
    }
}
