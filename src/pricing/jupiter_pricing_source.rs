use reqwest::blocking::Client;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use rocket::serde::json::serde_json;

#[derive(Clone)]
pub struct JupiterPricingSource {
    price_map: Arc<Mutex<HashMap<String, f64>>>,
    client: Client,
}

impl JupiterPricingSource {
    pub fn new() -> Self {
        Self {
            price_map: Arc::new(Mutex::new(HashMap::new())),
            client: Client::new(),
        }
    }

    pub fn get_usdc_price_for_symbol(&self, symbol: &str, usdc_amount: i64) -> Option<f64> {
        let url = format!("https://price.jup.ag/v4/price?ids={}&vsAmount={}", symbol, usdc_amount);

        match self.client.get(&url).send() {
            Ok(response) => {
                if let Ok(json) = response.text() {
                    if let Ok(map) = serde_json::from_str::<serde_json::Value>(&json) {
                        if let Some(data) = map.get("data") {
                            if let Some(symbol_data) = data.get(symbol) {
                                if let Some(price) = symbol_data.get("price") {
                                    if let Some(price_value) = price.as_f64() {
                                        return Some(price_value);
                                    }
                                }
                            }
                        }
                    }
                }
                None
            },
            Err(e) => {
                eprintln!("Error getting Jupiter price for {}: {}", symbol, e);
                None
            }
        }
    }

    pub fn update_price_map(&self, symbol: String, price: f64) {
        self.price_map.lock().unwrap().insert(symbol, price);
    }

    pub fn get_cached_price(&self, symbol: &str) -> Option<f64> {
        self.price_map.lock().unwrap().get(symbol).cloned()
    }
}
