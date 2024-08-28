use crate::blockchain::transaction_pool::TransactionPool;

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