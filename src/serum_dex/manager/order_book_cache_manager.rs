use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use solana_account_decoder::UiAccountEncoding;
use solana_client::rpc_client::RpcClient;
use solana_client::rpc_config::RpcAccountInfoConfig;
use solana_program::pubkey::Pubkey;
use solana_sdk::commitment_config::CommitmentConfig;
use tokio::sync::Mutex as TokioMutex;
use tokio::time;
use crate::serum_dex::model::order_book::OrderBook;

type LoadingCache<K, V, F> = Arc<Mutex<HashMap<K, (Instant, V)>>>;

struct OrderBookCacheManager {
    client: Arc<RpcClient>,
    orderbook_cache: LoadingCache<String, OrderBook, Box<dyn Fn(String) -> OrderBook + Send>>,
}

impl OrderBookCacheManager {
    pub fn new(client: Arc<RpcClient>) -> Self {
        let orderbook_cache: LoadingCache<String, OrderBook, Box<dyn Fn(String) -> OrderBook + Send>> =
            Arc::new(Mutex::new(HashMap::new()));

        OrderBookCacheManager {
            client,
            orderbook_cache,
        }
    }

    pub async fn get_order_book(&self, market_id: &Pubkey) -> OrderBook {
        let client_clone = self.client.clone();

        let mut cache_guard = self.orderbook_cache.clone().lock().unwrap();

        if let Some((timestamp, value)) = cache_guard.get(&market_id.to_string()) {
            if Instant::now() - *timestamp <= Duration::from_secs(1) {
                return value.clone();
            }
        }

        let value = OrderBook::read_order_book(
            {
                let rpc_result = client_clone.get_account_with_config(market_id, RpcAccountInfoConfig{
                    encoding: Some(UiAccountEncoding::Binary),
                    data_slice: None,
                    commitment: Some(CommitmentConfig::confirmed()),
                    min_context_slot: None,
                });
                if rpc_result.is_err() {
                    eprintln!("{:?}", rpc_result.err().unwrap());
                    return {
                        let (_, order_book) = cache_guard.get(&market_id.to_string()).unwrap();
                        order_book.clone()
                    };
                }
                rpc_result.unwrap().value.unwrap().data
            }
        );

        cache_guard.insert(market_id.to_string(), (Instant::now(), value.clone()));
        value
    }
}