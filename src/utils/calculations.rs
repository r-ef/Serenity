use log::debug;

use crate::blockchain::{block::Block, transaction_pool::TransactionPool};

const TARGET_BLOCK_TIME: u64 = 60; // Target block time in seconds
const DIFFICULTY_ADJUSTMENT_INTERVAL: usize = 10; // Number of blocks to consider for difficulty adjustment

pub fn calculate_fee(amount: f64) -> f64 {
    let fee_percentage = 0.01; // 1% transaction fee
    amount * fee_percentage
}

pub fn calculate_block_subsidy(height: u64) -> f64 {
    let halving_interval = 210_000;
    let subsidy = 50.0;
    let mut total_subsidy = subsidy;

    for _ in 0..(height / halving_interval) {
        total_subsidy /= 2.0;
    }

    total_subsidy
}

const REWARD_SCALING_FACTOR: f64 = 0.01;

pub fn calculate_mining_reward(height: u64, pool: &TransactionPool) -> f64 {
    let subsidy = calculate_block_subsidy(height);
    let total_fee = pool.pool.iter().map(|tx| tx.fee).sum::<f64>();

    (subsidy + total_fee) * REWARD_SCALING_FACTOR
}

pub fn calculate_difficulty(chain: &Vec<Block>) -> u32 {
    if chain.len() < DIFFICULTY_ADJUSTMENT_INTERVAL {
        return 1;
    }

    let last_block = chain.last().unwrap();
    let prev_adjustment_block = chain.get(chain.len() - DIFFICULTY_ADJUSTMENT_INTERVAL).unwrap();
    let time_diff = last_block.timestamp - prev_adjustment_block.timestamp;
    let expected_time = TARGET_BLOCK_TIME * DIFFICULTY_ADJUSTMENT_INTERVAL as u64;

    let mut difficulty = last_block.difficulty;

    if time_diff < expected_time {
        difficulty += 1;
    } else if difficulty > 1 {
        difficulty -= 1;
    }

    difficulty
}