use log::debug;
use sha2::{Sha256, Digest};
use crate::blockchain::block::Block;
use num_bigint::BigUint;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;
use rayon::prelude::*;


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
        let now = Instant::now();
        let target = (1u64 << (64 - difficulty)) - 1;
        info!("Starting to mine block with difficulty: {}", difficulty);

        let nonce = Arc::new(AtomicU64::new(0));
        let found = Arc::new(AtomicU64::new(0));

        let mut hasher = Sha256::new();
        hasher.update(&self.block.index.to_le_bytes());
        hasher.update(&self.block.timestamp.to_le_bytes());
        hasher.update(&self.block.prev_hash);
        hasher.update(&self.block.data);
        let constant_hash = hasher.finalize_reset();

        let result = rayon::scope(|s| {
            let result = Arc::new(std::sync::Mutex::new(None));

            for _ in 0..rayon::current_num_threads() {
                let nonce = Arc::clone(&nonce);
                let found = Arc::clone(&found);
                let constant_hash = constant_hash.clone();
                let result = Arc::clone(&result);

                s.spawn(move |_| {
                    let mut hasher = Sha256::new();
                    let mut local_nonce = nonce.fetch_add(1, Ordering::Relaxed);

                    while found.load(Ordering::Relaxed) == 0 {
                        hasher.update(&constant_hash);
                        hasher.update(&local_nonce.to_le_bytes());
                        let hash_result = hasher.finalize_reset();

                        let hash_prefix = u64::from_be_bytes(hash_result[0..8].try_into().unwrap());
                        if hash_prefix <= target {
                            let mut result_guard = result.lock().unwrap();
                            *result_guard = Some((local_nonce, hex::encode(hash_result)));
                            found.store(1, Ordering::Relaxed);
                            break;
                        }

                        local_nonce = nonce.fetch_add(1, Ordering::Relaxed);
                    }
                });
            }
            let elapsed = now.elapsed();
            info!("Mining took: {}s", elapsed.as_secs_f64());
            let result = result.lock().unwrap();
            result.clone()
        });

        debug!("Result: {:?}", result);
        if let Some((nonce, hash)) = result {
            self.block.nonce = nonce;
            self.block.hash = hash;
            info!("Block mined: nonce = {}, hash = {}", self.block.nonce, self.block.hash);
        } else {
            error!("Mining failed to produce a result");
        }
    }
}
