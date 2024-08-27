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