use std::fmt::Error;
use pyth_sdk_solana::PythError;
use pyth_sdk_solana::state::{load_price_account, PriceAccount};
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;

pub struct PythPricingSource {
    client: RpcClient,
    sol_usd_price_data_account: Pubkey,
    sol_price: Option<f64>,
    sol_price_confidence: Option<f64>,
}

impl PythPricingSource {
    pub fn new(client: RpcClient) -> Self {
        Self {
            client,
            sol_usd_price_data_account: Pubkey::new(&[0; 32]), //todo Replace with actual pubkey
            sol_price: None,
            sol_price_confidence: None,
        }
    }

    //todo this function needs to be called every 190ms
    pub fn update_sol_price_cache(&mut self) {
        let price_data_account = self.get_price_data_account(&self.sol_usd_price_data_account);
        if let Some(price_data_account) = price_data_account {
            self.sol_price_confidence = Some(price_data_account.agg.conf as f64 * (10_f64).powi(price_data_account.expo)); //https://docs.rs/pyth-sdk-solana/latest/pyth_sdk_solana/state/struct.PriceInfo.html#structfield.price
            self.sol_price = Some(price_data_account.agg.price as f64 * (10_f64).powi(price_data_account.expo)); //todo test to see if directly accessing agg.price causes any issues
        }
    }

    pub fn get_sol_bid_price(&self) -> f64 {
        if let (Some(price), Some(confidence)) = (self.sol_price, self.sol_price_confidence) {
            return price - confidence;
        }
        return  0.0;
    }

    pub fn get_sol_ask_price(&self) -> f64 {
        if let (Some(price), Some(confidence)) = (self.sol_price, self.sol_price_confidence) {
            return price + confidence;
        }
        return 999999.9;
    }

    pub fn get_sol_midpoint_price(&self) -> Option<f64> {
        self.sol_price
    }

    pub fn get_sol_price_confidence(&self) -> Option<f64> {
        self.sol_price_confidence
    }

    pub fn has_sol_price(&self) -> bool {
        self.sol_price.is_some() && self.sol_price_confidence.is_some()
    }

    fn get_price_data_account(&self, public_key: &Pubkey) -> Option<PriceAccount> {
        let data = self.client.get_account_data(&public_key);
        if data.is_ok() {
            let data = data.unwrap();
            let price_account = load_price_account(&data);
            return Some(*price_account.unwrap());
        }
        else {
            eprintln!("{}", data.err().unwrap());
            return None;
        }
    }
}
