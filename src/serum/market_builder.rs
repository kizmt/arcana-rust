use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::Duration;
use std::borrow::BorrowMut;
use bytemuck::cast;
use serum_dex::critbit::SlabView;
use serum_dex::state::{Market, MarketState};
use solana_account_decoder::UiAccountEncoding;
use solana_client::rpc_client::RpcClient;
use solana_client::rpc_config::RpcAccountInfoConfig;
use solana_program::account_info::{AccountInfo, IntoAccountInfo};
use solana_program::pubkey::Pubkey;
use solana_sdk::account::Account;
use solana_sdk::commitment_config::CommitmentConfig;
use crate::serum::market::MarketWrapper;
use crate::serum::order_book::OrderBook;
use crate::serum::serum_utils;
use crate::serum::serum_utils::{pub_key, SerumUtils};

pub struct MarketBuilder {
    client: RpcClient,
    market_id: Pubkey,
    min_context_slot: u64,
    //built: bool,
    //base64_account_info: Option<Arc<Mutex<AccountInfo<'a>>>>,
    decimals_cache: HashMap<Pubkey, i8>,
}

impl MarketBuilder {
    pub fn new(client: RpcClient, public_key: Pubkey) -> Self {
        MarketBuilder {
            client,
            market_id: public_key,
            min_context_slot: 0,
            //built: false,
            //base64_account_info: None,
            decimals_cache: HashMap::new(),
        }
    }


    pub fn build(&mut self) -> MarketWrapper {
        let acc = self.client.get_account_with_config(&self.market_id, RpcAccountInfoConfig {
            encoding: None,
            data_slice: None,
            commitment: Some(CommitmentConfig::processed()),
            min_context_slot: Some(self.min_context_slot, )
        })
            .expect("Error occurred while getting account info")
            .value;

        let account_info = (self.market_id, acc.unwrap()).into_account_info();

        let market = Market::load(account_info.as_ref(), &self.market_id, true)
            .expect("Error loading market");


        let base_mint = pub_key(market.coin_mint);
        let quote_mint = pub_key(market.pc_mint);

        let mut base_decimals: i8 = 0;
        let mut quote_decimals: i8 = 0;

        if self.decimals_cache.contains_key(&base_mint) {
            base_decimals = *self.decimals_cache.get(&base_mint).unwrap();
        } else {
            base_decimals = self.get_mint_decimals(&base_mint);
            self.decimals_cache.insert(base_mint, base_decimals);
        }

        if self.decimals_cache.contains_key(&quote_mint) {
            quote_decimals = *self.decimals_cache.get(&quote_mint).unwrap();
        } else {
            quote_decimals = self.get_mint_decimals(&quote_mint);
            self.decimals_cache.insert(quote_mint, quote_decimals);
        }

        let bids_pubkey = pub_key(market.bids);
        let bid_acc = self.client.get_account_with_commitment(&bids_pubkey, CommitmentConfig::processed()).unwrap().value.unwrap();
        let asks_pubkey = pub_key(market.asks);
        let ask_acc = self.client.get_account_with_commitment(&asks_pubkey, CommitmentConfig::processed()).unwrap().value.unwrap();
        let mut x = (bids_pubkey, bid_acc);
        let bids_info = x.into_account_info();
        let mut x1 = (asks_pubkey, ask_acc);
        let asks_info = x1.into_account_info();

        let bid_orders = market.load_bids_mut(
            &bids_info
        ).unwrap();
        let ask_orders = market.load_asks_mut(
            &asks_info
        ).unwrap();

        //self.built = true;
        MarketWrapper {
            base_decimals,
            quote_decimals,
            bid_order_book: OrderBook::new(bid_orders, base_decimals, quote_decimals, market.coin_lot_size, market.pc_lot_size),
            ask_order_book: OrderBook::new(ask_orders, base_decimals, quote_decimals, market.coin_lot_size, market.pc_lot_size),
            market: Arc::new(Mutex::new(market)),
        }
    }

    fn get_mint_decimals(&self, token_mint: &Pubkey) -> i8 {
        if token_mint == &*serum_utils::WRAPPED_SOL_MINT {
            return 9;
        }

        // USDC and USDT cases
        if token_mint == &*serum_utils::USDC_MINT || token_mint == &*serum_utils::USDT_MINT {
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
}