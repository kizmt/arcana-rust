use std::str::FromStr;
use std::sync::{Arc, Mutex};
use solana_sdk::pubkey::Pubkey;
use tokio::runtime::Runtime;
use crate::strategies::strategy::Strategy;

pub struct OpenBookBot {
    pub strategy: Option<Box<dyn Strategy>>,
    pub strategy_executor: Option<Runtime>,
    market_id: Pubkey,
    bps_spread: f64,
    amount_bid: f64,
    amount_ask: f64,
    ooa: Pubkey,
    base_wallet: Pubkey,
    quote_wallet: Pubkey,
    price_strategy: String,
}

impl OpenBookBot {
    pub fn new() -> Self {
        Self {
            strategy: None,
            strategy_executor: None,
            market_id: Pubkey::from_str("9Lyhks5bQQxb9EyyX55NtgKQzpM4WK7JCmeaWuQ5MoXD").unwrap(),
            bps_spread: 10.0,
            amount_bid: 0.1,
            amount_ask: 0.1,
            ooa: Pubkey::from_str("7hM4pmTbyfAUoxU9p8KCqdFfdPTLXc5xFijXsbumaqAa").unwrap(),
            base_wallet: Pubkey::from_str("3UrEoG5UeE214PYQUA487oJRN89bg6fmt3ejkavmvZ81").unwrap(),
            quote_wallet: Pubkey::from_str("A6Jcj1XV6QqDpdimmL7jm1gQtSP62j8BWbyqkdhe4eLe").unwrap(),
            price_strategy: "jupiter".to_string(),
        }
    }
}


//todo idk what to do about this, this is very unsafe and most probably requires restructuring the whole codebase
// unsafe impl Send for OpenBookBot {}
// unsafe impl Sync for OpenBookBot {}

//strategy: Box<dyn Strategy>, market_id: Pubkey, bps_spread: f64, amount_bid: f64, amount_ask: f64