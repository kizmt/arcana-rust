use std::collections::HashMap;
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;
use solana_client::rpc_client::RpcClient;

use p2p_solanaj::core::PublicKey;
use p2p_solanaj::rpc::client::RpcClient;
use p2p_solanaj::rpc::error::RpcError;
use p2p_solanaj::rpc::types::AccountInfo;
use p2p_solanaj::rpc::types::Commitment;
use p2p_solanaj::rpc::types::RpcSendTransactionConfig;
use p2p_solanaj::rpc::RpcException;
use p2p_solanaj::rpc::RpcRequestConfig;
use p2p_solanaj::rpc::RpcSender;
use p2p_solanaj::token::utils::read_decimals_from_token_mint_data;
use p2p_solanaj::utils::base64::decode_base64;
use p2p_solanaj::utils::base64::encode_base64;
use p2p_solanaj::utils::base64::encode_base64_bytes;
use serde_json::json;
use tokio::runtime::Runtime;
use tokio::sync::RwLock;
use tokio::time::sleep as async_sleep;
use tokio::time::Duration as AsyncDuration;

pub struct MarketBuilder {
    client: RpcClient,
    public_key: PublicKey,
    retrieve_orderbooks: bool,
    retrieve_event_queue: bool,
    retrieve_decimals_only: bool,
    order_book_cache_enabled: bool,
    min_context_slot: i64,
    built: bool,
    base64_account_info: Option<Vec<u8>>,
    order_book_cache_manager: Option<Arc<RwLock<OrderBookCacheManager>>>,
    decimals_cache: Arc<RwLock<HashMap<PublicKey, i8>>>,
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
            decimals_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn set_retrieve_orderbooks(&mut self, retrieve_orderbooks: bool) -> &mut Self {
        self.retrieve_orderbooks = retrieve_orderbooks;
        self
    }

    pub fn set_order_book_cache_enabled(&mut self, order_book_cache_enabled: bool) -> &mut Self {
        self.order_book_cache_enabled = order_book_cache_enabled;
        if order_book_cache_enabled {
            self.order_book_cache_manager =
                Some(Arc::new(RwLock::new(OrderBookCacheManager::new(
                    self.client.clone(),
                ))));
        } else {
            self.order_book_cache_manager = None;
        }
        self
    }



    pub fn set_retrieve_event_queue(&mut self, retrieve_event_queue: bool) -> &mut Self {
        self.retrieve_event_queue = retrieve_event_queue;
        self
    }

    pub fn set_retrieve_decimals_only(&mut self, retrieve_decimals_only: bool) -> &mut Self {
        self.retrieve_decimals_only = retrieve_decimals_only;
        self
    }



    async fn get_mint_decimals(&self, token_mint: PublicKey) -> Result<i8, RpcException> {
        if token_mint == SerumUtils::WRAPPED_SOL_MINT {
            return Ok(9);
        }

        if token_mint == SerumUtils::USDC_MINT || token_mint == SerumUtils::USDT_MINT {
            return Ok(6);
        }

        async_sleep(AsyncDuration::from_millis(250)).await; // 250ms sleep

        let account_data = self.retrieve_account_data_confirmed(token_mint).await?;

        let decimals = read_decimals_from_token_mint_data(&account_data)?;

        Ok(decimals)
    }



    async fn retrieve_account_data_confirmed(&self, key: PublicKey) -> Result<Vec<u8>, RpcException> {
        let rpc_sender = RpcSender::new(&self.client);

        let request_config = RpcRequestConfig {
            encoding: Some(RpcSendTransactionConfig::Encoding::Base64),
            commitment: Some(Commitment::Confirmed),
            ..RpcRequestConfig::default()
        };

        let account_info = rpc_sender
            .get_account_info(key, Some(request_config))
            .await?;

        let account_data_base64 = account_info
            .value
            .get_data()
            .first()
            .cloned()
            .unwrap_or_default();

        let account_data = decode_base64(&account_data_base64)?;

        Ok(account_data)
    }

    async fn get_base_quote_decimals(&self, market: &Market) -> Result<(i8, i8), RpcException> {
        let base_mint = market.get_base_mint();
        let quote_mint = market.get_quote_mint();

        let base_decimals = self.get_mint_decimals(base_mint).await?;
        let quote_decimals = self.get_mint_decimals(quote_mint).await?;

        Ok((base_decimals, quote_decimals))
    }

    async fn retrieve_order_books(
        &self,
        market: &Market,
    ) -> Result<(OrderBook, OrderBook), RpcException> {
        let base_mint = market.get_base_mint();
        let quote_mint = market.get_quote_mint();

        let base_decimals = self.get_mint_decimals(base_mint).await?;
        let quote_decimals = self.get_mint_decimals(quote_mint).await?;

        let base_lot_size = market.get_base_lot_size();
        let quote_lot_size = market.get_quote_lot_size();

        let (bid_order_book, ask_order_book) = tokio::try_join!(
            self.retrieve_order_book(market.get_bids(), base_decimals, quote_decimals),
            self.retrieve_order_book(market.get_asks(), base_decimals, quote_decimals)
        )?;

        Ok((bid_order_book, ask_order_book))
    }

    async fn retrieve_order_book(
        &self,
        key: PublicKey,
        base_decimals: i8,
        quote_decimals: i8,
    ) -> Result<OrderBook, RpcException> {
        if let Some(order_book_cache_manager) = &self.order_book_cache_manager {
            let order_book_cache_manager = order_book_cache_manager.read().await;
            if let Some(order_book) = order_book_cache_manager.get_order_book(&key) {
                return Ok(order_book.clone());
            }
        }

        let account_data_base64 = self.retrieve_account_data_for_key(key).await?;
        let order_book = OrderBook::read_order_book(
            &account_data_base64,
            base_decimals,
            quote_decimals,
        )?;

        if let Some(order_book_cache_manager) = &self.order_book_cache_manager {
            let mut order_book_cache_manager = order_book_cache_manager.write().await;
            order_book_cache_manager.cache_order_book(key, order_book.clone());
        }

        Ok(order_book)
    }
}








use std::collections::HashMap;
use std::sync::Mutex;
use serum_dex::state::Market;
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;

pub struct ArcanaBackgroundCache<'a> {
    rpc_client: RpcClient,
    cached_markets: Mutex<HashMap<Pubkey, Market<'a>>>, // You need to implement Market struct
}

impl ArcanaBackgroundCache {
    pub fn new(rpc_client: RpcClient) -> Self {
        Self {
            rpc_client,
            cached_markets: Mutex::new(HashMap::new()),
        }
    }

    pub fn background_cache_markets(&self) {
        let program_id = Pubkey::new(&[0; 32]); // Replace with actual program ID
        let program_accounts = match self.rpc_client.get_program_accounts_with_data(&program_id) {
            Ok(accounts) => accounts,
            Err(err) => {
                eprintln!("Error fetching program accounts: {:?}", err);
                return;
            }
        };

        let mut cached_markets = self.cached_markets.lock().unwrap();
        for (pubkey, account) in program_accounts {
            let market: Market = match Market::unpack(&account.data) {
                Ok(market) => market,
                Err(err) => {
                    eprintln!("Error unpacking market data: {:?}", err);
                    continue;
                }
            };

            // Ignore fake/erroneous market accounts
            if market.own_address == Pubkey::new(&[0; 32]) {
                continue;
            }

            cached_markets.insert(pubkey, market);
        }
    }
}
