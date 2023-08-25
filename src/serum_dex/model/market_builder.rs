use std::collections::HashMap;
use std::sync::Arc;
use std::{task, thread, time};
use std::error::Error;
use std::thread::sleep;
use std::time::Duration;
use rocket::serde::json::serde_json;
use serde_json::value::Value;
use solana_client::rpc_client::RpcClient;
use solana_client::rpc_request::{RpcError, RpcRequest};

use serde_json::json;
use solana_account_decoder::UiAccountEncoding::Base64;
use solana_account_decoder::{UiAccountEncoding, UiDataSliceConfig};
use solana_client::rpc_config::{RpcAccountInfoConfig, RpcSendTransactionConfig};
use solana_client::rpc_response::RpcResult;
use solana_program::pubkey::Pubkey;
use solana_sdk::account::Account;
use solana_sdk::commitment_config::{CommitmentConfig, CommitmentLevel};
use solana_sdk::transaction::Transaction;
use crate::serum_dex::model::market::Market;
use crate::serum_dex::model::order_book::OrderBook;
use crate::serum_dex::model::serum_utils;
use crate::serum_dex::model::serum_utils::SerumUtils;

pub struct MarketBuilder {
    client: RpcClient,
    public_key: PublicKey,
    retrieve_orderbooks: bool,
    retrieve_event_queue: bool,
    retrieve_decimals_only: bool,
    order_book_cache_enabled: bool,
    min_context_slot: u64,
    built: bool,
    base64_account_info: Option<Vec<u8>>,
    order_book_cache_manager: Option<Arc<RwLock<OrderBookCacheManager>>>,
    decimals_cache: HashMap<PublicKey, u8>,
}

impl MarketBuilder {
    pub fn new(client: RpcClient, public_key: PublicKey) -> Self {
        MarketBuilder {
            client,
            public_key,
            retrieve_orderbooks: false,
            retrieve_event_queue: false,
            retrieve_decimals_only: false,
            order_book_cache_enabled: false,
            min_context_slot: 0,
            built: false,
            base64_account_info: None,
            order_book_cache_manager: None,
            decimals_cache: HashMap::new(),
        }
    }

    pub fn set_retrieve_orderbooks(&mut self, retrieve_orderbooks: bool) -> &mut Self {
        self.retrieve_orderbooks = retrieve_orderbooks;
        self
    }

    pub fn build(&mut self) -> Result<Market, dyn Error> {
        if !self.built {
            self.base64_account_info = self.retrieve_account_data();
        }

        let base64_account_info = self.base64_account_info.clone().ok_or_else(|| {
            RpcException::Custom("Unable to read account data".to_string())
        })?;

        let mut market = Market::read_market(&base64_account_info);//todo implement this function

        let base_mint = market.base_mint;
        let quote_mint = market.quote_mint;

        if self.retrieve_orderbooks {
            let mut base_decimals:u8 = 0;
            let mut quote_decimals:u8 = 0;

            if self.decimals_cache.contains_key(&base_mint) {
                base_decimals = *self.decimals_cache.get(&base_mint).unwrap();
            } else {
                base_decimals = self.get_mint_decimals(base_mint);
                self.decimals_cache.insert(base_mint, base_decimals)
            }

            if self.decimals_cache.contains_key(&quote_mint) {
                quote_decimals = *self.decimals_cache.get(&quote_mint).unwrap();
            } else {
                quote_decimals = self.get_mint_decimals(quote_mint);
                self.decimals_cache.insert(quote_mint, quote_decimals);
            }

            market.base_decimals = base_decimals;
            market.quote_decimals = quote_decimals;


            let bid_thread = thread::spawn(async || {
                self.retrieve_order_book(&market.bids).await
            });
            let ask_thread = thread::spawn(async || {
                self.retrieve_order_book(&market.asks).await
            });

            let (bid_order_book_result, ask_order_book_result) = (bid_thread.join(), ask_thread.join());

            if bid_order_book_result.is_err() { return Err(bid_order_book_result.err().unwrap()); }
            if ask_order_book_result.is_err() { return Err(ask_order_book_result.err().unwrap()); }

            let (bid_order_book_result, ask_order_book_result) = (bid_order_book_result.unwrap(), ask_order_book_result.unwrap());

            if bid_order_book_result.is_err() { return Err(bid_order_book_result.err().unwrap()); }
            if ask_order_book_result.is_err() { return Err(ask_order_book_result.err().unwrap()); }

            let (mut bid_order_book, mut ask_order_book) = (bid_order_book_result.unwrap(), ask_order_book_result.unwrap());

            bid_order_book.base_decimals = base_decimals;
            bid_order_book.quote_decimals = quote_decimals;
            ask_order_book.base_decimals = base_decimals;
            ask_order_book.quote_decimals = quote_decimals;

            bid_order_book.base_lot_size = market.base_lot_size;
            bid_order_book.quote_lot_size = market.quote_lot_size;
            ask_order_book.base_lot_size = market.base_lot_size;
            ask_order_book.quote_lot_size = market.quote_lot_size;

            market.bid_order_book = bid_order_book;
            market.ask_order_book = ask_order_book;
        }

        if self.retrieve_event_queue {
            let base64_event_queue = self.retrieve_account_data_for_key(market.event_queue_key);

            let (mut base_decimals, mut quote_decimals) = (0, 0);

            if let Some(val) = self.decimals_cache.get(&base_mint) {
                base_decimals = val.clone();
            }
            else {
                base_decimals = self.get_mint_decimals(&base_mint);
                self.decimals_cache.insert(base_mint, base_decimals);
            }

            if let Some(val) = self.decimals_cache.get(&quote_mint) {
                quote_decimals = val.clone();
            }
            else {
                quote_decimals = self.get_mint_decimals(&quote_mint);
                self.decimals_cache.insert(quote_mint, quote_decimals);
            }

            market.base_decimals = base_decimals;
            market.quote_decimals = quote_decimals;

            let base_lot_size = market.base_lot_size;
            let quote_lot_size = market.quote_lot_size;

            EventQueue

            // let (base_decimals, quote_decimals) =
            //     self.get_base_quote_decimals(&market).await?;
            // let (base_lot_size, quote_lot_size) = (
            //     market.get_base_lot_size(),
            //     market.get_quote_lot_size(),
            // );
            // let event_queue = EventQueue::read_event_queue(
            //     &base64_event_queue,
            //     base_decimals,
            //     quote_decimals,
            //     base_lot_size,
            //     quote_lot_size,
            // )?;
            // market.set_event_queue(event_queue);
        }

        if self.retrieve_decimals_only {
            let (base_decimals, quote_decimals) = self.get_base_quote_decimals(&market).await?;
            market.set_base_decimals(base_decimals);
            market.set_quote_decimals(quote_decimals);
        }

        self.built = true;

        Ok(market)
    }

    fn retrieve_account_data(&mut self) -> Option<Vec<u8>> {
        self.retrieve_account_data_for_key(self.public_key)
    }

    fn retrieve_account_data_for_key(&mut self, public_key: PublicKey) -> Option<Vec<u8>> {
        let config = RpcAccountInfoConfig {
            encoding: Some(UiAccountEncoding::Binary),
            data_slice: None,
            commitment: Some(CommitmentConfig::processed()),
            min_context_slot: Some(self.min_context_slot),
        };

        let order_book_result: RpcResult<Option<Account>> = self.client.get_account_with_config(public_key, config);

        if order_book_result.is_err() {
            eprintln!("{:?}", order_book_result.err().unwrap());
            return None;
        }

        let order_book = order_book_result.unwrap();

        self.set_min_context_slot(order_book.context.slot as u64);

        Some(Vec::from(order_book.value.unwrap().data))
    }

    pub fn set_min_context_slot(&mut self, min_context_slot: u64) -> &mut Self {
        if min_context_slot > self.min_context_slot {
            self.min_context_slot = min_context_slot;
        }
        self
    }

    fn get_mint_decimals(&self, token_mint: &Pubkey) -> u8 {
        if token_mint == serum_utils::WRAPPED_SOL_MINT {
            return 9;
        }

        // USDC and USDT cases
        if token_mint == serum_utils::USDC_MINT || token_mint == serum_utils::USDT_MINT {
            return 6;
        }

        // Sleep for 100ms to avoid rate limit
        sleep(Duration::from_millis(100));

        // RPC call to get mint's account data into decoded bytes (already base64 decoded)
        let account_data = self.retrieve_account_data_confirmed(&token_mint);
        if account_data.is_none() {
            panic!("retrieve_account_data_confirmed function failed");
        }

        // Deserialize account_data into the MINT_LAYOUT enum
        let decimals = SerumUtils::read_decimals_from_token_mint_data(&account_data.unwrap());

        decimals
    }

    fn retrieve_account_data_confirmed(&self, public_key: &Pubkey) -> Option<Vec<u8>> {
        let config = RpcAccountInfoConfig {
            encoding: Some(UiAccountEncoding::Binary),
            data_slice: None,
            commitment: Some(CommitmentConfig::confirmed()),
            min_context_slot: None,
        };
        let account_info_result = self.client.get_account_with_config(&public_key, config);

        if account_info_result.is_err() {
            eprintln!("{:?}", account_info_result.err().unwrap());
            return None;
        }

        Some(account_info_result.unwrap().value.unwrap().data)
    }

    async fn retrieve_order_book(&self, public_key: &Pubkey) -> Result<OrderBook, dyn Error> {
        if self.order_book_cache_enabled {
            // Use the cache manager if enabled
            let order_book = self.order_book_cache_manager.get_order_book(public_key);
            return order_book;
        } else {
            // Fetch fresh order book data
            let order_book = OrderBook::read_order_book(
                {
                    let rpc_result = self.client.get_account_with_config(market_id, RpcAccountInfoConfig{
                        encoding: Some(UiAccountEncoding::Binary),
                        data_slice: None,
                        commitment: Some(CommitmentConfig::processed()),
                        min_context_slot: None,
                    });
                    if rpc_result.is_err() {
                        eprintln!("{:?}", rpc_result.err().unwrap());
                        return Err(rpc_result.err().unwrap());
                    }
                    rpc_result.unwrap().value.unwrap().data
                }
            );
            return Ok(order_book);
        }
    }
}
