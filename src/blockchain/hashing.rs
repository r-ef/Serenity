use log::debug;
use sha2::{Sha256, Digest};
use crate::blockchain::block::Block;
use num_bigint::BigUint;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;
use rayon::prelude::*;
use ocl::{ProQue, Buffer};


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

    pub fn mine_block_parallel(&mut self, difficulty: u32, start_nonce: u64, step: u64) {
        let max_target = BigUint::from(1u64) << 256;
        let target = max_target >> difficulty;
        let mut nonce = start_nonce;

        loop {
            self.block.nonce = nonce;
            let hash = self.calculate_hash();
            if let Some(hash_value) = BigUint::parse_bytes(&hex::decode(&hash).unwrap(), 16) {
                if hash_value <= target {
                    self.block.hash = hash;
                    return;
                }
            }
            nonce += step;
        }
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

    pub fn mine_block2(&mut self, difficulty: u32) {
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
    
        let kernel_src = r#"
            __kernel void hash_kernel(__global const uchar* constant_hash, __global ulong* nonce, __global ulong* found, __global ulong* result_nonce, __global uchar* result_hash, ulong target) {
                int gid = get_global_id(0);
                ulong local_nonce = nonce[0] + gid;
                uchar hash[32];
                sha256(constant_hash, local_nonce, hash);
    
                ulong hash_prefix = 0;
                for (int i = 0; i < 8; i++) {
                    hash_prefix = (hash_prefix << 8) | hash[i];
                }
    
                if (hash_prefix <= target) {
                    found[0] = 1;
                    result_nonce[0] = local_nonce;
                    for (int i = 0; i < 32; i++) {
                        result_hash[i] = hash[i];
                    }
                }
            }
    
            void sha256(__global const uchar* constant_hash, ulong nonce, __global uchar* hash) {
                    // Pre-compute constants for efficiency
                    uint32_t h0 = 0x6a09e667;
                    uint32_t h1 = 0xbb67ae85;
                    uint32_t h2 = 0x3c6ef372;
                    uint32_t h3 = 0xa54ff53a;
                    uint32_t h4 = 0x510e527f;
                    uint32_t h5 = 0x9b04a709;   

                    uint32_t h6 = 0x5570cbbb;
                    uint32_t h7 = 0x63238b4c;
                    uint32_t k[64];
                    for (int i = 0; i < 64; i++) {
                        k[i] = sha256_k[i];
                    }

                    // Prepare message
                    uint32_t message[16];
                    for (int i = 0; i < 16; i++) {
                        message[i] = (constant_hash[4 * i] << 24) | (constant_hash[4 * i + 1] << 16) | (constant_hash[4 * i + 2] << 8) | constant_hash[4 * i + 3];
                    }
                    message[15] = nonce;

                    // Pad message
                    message[16] = 0x80;
                    for (int i = 17; i < 64; i++) {
                        message[i] = 0;
                    }
                    message[63] = (1 << 29) | (nonce >> 32);

                    // Perform compression rounds
                    uint32_t w[64];
                    for (int i = 0; i < 64; i++) {
                        w[i] = message[i];
                    }
                    for (int i = 16; i < 64; i++) {
                        w[i] = (w[i - 15] ^ w[i - 2] ^ w[i - 3] ^ w[i - 7]) << 1 | (w[i - 15] ^ w[i - 2] ^ w[i - 3] ^ w[i - 7]) >> 31;
                        w[i] ^= w[i - 16];
                    }

                    uint32_t a = h0;
                    uint32_t b = h1;
                    uint32_t c = h2;
                    uint32_t d = h3;
                    uint32_t e = h4;
                    uint32_t f = h5;
                    uint32_t g = h6;
                    uint32_t h = h7;

                    for (int i = 0; i < 64; i++) {
                        uint32_t   
                t1 = h + (e << 6 | e >> 26) + (f ^ g ^ h) + k[i] + w[i];
                        uint32_t t2 = (a << 6 | a >> 26) + (b ^ c ^ d);
                        h = g;
                        g = f;
                        f = e;
                        e = d + t1;
                        d = c;
                        c = b;
                        b = a;
                        a = t1 + t2;
                    }

                    h0 += a;
                    h1 += b;
                    h2 += c;
                    h3 += d;
                    h4 += e;
                    h5 += f;
                    h6 += g;
                    h7 += h;

                    // Store hash result
                    hash[0] = (h0 >> 24) & 0xff;
                    hash[1] = (h0 >> 16) & 0xff;
                    hash[2] = (h0 >> 8) & 0xff;
                    hash[3] = h0 & 0xff;
            }
        "#;
    
        // Set up OpenCL environment
        let pro_que = ProQue::builder()
            .src(kernel_src)
            .dims(1 << 20)
            .build().unwrap();
    
        let constant_hash_buffer = Buffer::<u8>::builder()
            .queue(pro_que.queue().clone())
            .flags(ocl::flags::MEM_READ_ONLY)
            .len(constant_hash.len())
            .copy_host_slice(&constant_hash)
            .build().unwrap();
    
        let nonce_buffer = Buffer::<u64>::builder()
            .queue(pro_que.queue().clone())
            .flags(ocl::flags::MEM_READ_WRITE)
            .len(1)
            .copy_host_slice(&[nonce.load(Ordering::Relaxed)])
            .build().unwrap();
    
        let found_buffer = Buffer::<u64>::builder()
            .queue(pro_que.queue().clone())
            .flags(ocl::flags::MEM_READ_WRITE)
            .len(1)
            .copy_host_slice(&[found.load(Ordering::Relaxed)])
            .build().unwrap();
    
        let result_nonce_buffer = Buffer::<u64>::builder()
            .queue(pro_que.queue().clone())
            .flags(ocl::flags::MEM_WRITE_ONLY)
            .len(1)
            .build().unwrap();
    
        let result_hash_buffer = Buffer::<u8>::builder()
            .queue(pro_que.queue().clone())
            .flags(ocl::flags::MEM_WRITE_ONLY)
            .len(32)
            .build().unwrap();
    
        let kernel = pro_que.kernel_builder("hash_kernel")
            .arg(&constant_hash_buffer)
            .arg(&nonce_buffer)
            .arg(&found_buffer)
            .arg(&result_nonce_buffer)
            .arg(&result_hash_buffer)
            .arg(target)
            .build().unwrap();
    
        unsafe {
            kernel.enq().unwrap();
        }
    
        let mut found_value = vec![0u64; 1];
        found_buffer.read(&mut found_value).enq().unwrap();
    
        if found_value[0] == 1 {
            let mut result_nonce = vec![0u64; 1];
            result_nonce_buffer.read(&mut result_nonce).enq().unwrap();
    
            let mut result_hash = vec![0u8; 32];
            result_hash_buffer.read(&mut result_hash).enq().unwrap();
    
            self.block.nonce = result_nonce[0];
            self.block.hash = hex::encode(result_hash);
            info!("Block mined: nonce = {}, hash = {}", self.block.nonce, self.block.hash);
        } else {
            error!("Mining failed to produce a result");
        }
    
        let elapsed = now.elapsed();
        info!("Mining took: {}s", elapsed.as_secs_f64());
    }
}
