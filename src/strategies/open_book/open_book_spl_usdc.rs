
use std::num::NonZeroU64;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use bytemuck::cast;
use lazy_static::lazy_static;
use serum_dex::instruction::{cancel_order_by_client_order_id, consume_events, NewOrderInstructionV3, SelfTradeBehavior, settle_funds};
use serum_dex::matching::OrderType;
use serum_dex::matching::Side;
use serum_dex::state::{Market, MarketState};
use solana_client::rpc_client::RpcClient;
use solana_program::account_info::AccountInfo;
use solana_program::instruction::{AccountMeta, Instruction};
use solana_sdk::account::{Account, ReadableAccount};
use solana_sdk::bs58::encode;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::compute_budget::ComputeBudgetInstruction;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::transaction::Transaction;
use tokio::runtime::Runtime;
use tokio::time::{interval, sleep};
use uuid::Uuid;

use crate::pricing::jupiter_pricing_source::JupiterPricingSource;
use crate::serum::market::MarketWrapper;
use crate::serum::market_builder::MarketBuilder;
use crate::serum::serum_manager::SerumManager;
use crate::serum::serum_utils;
use crate::serum::serum_utils::{SERUM_PROGRAM_ID_V3, SerumUtils};
use crate::serum::serum_utils::pub_key;
use crate::strategies::strategy::Strategy;

const EVENT_LOOP_INITIAL_DELAY_MS: u64 = 0;
const EVENT_LOOP_DURATION_MS: u64 = 5000;
const SOL_QUOTE_SIZE: f64 = 0.1;
const MIN_MIDPOINT_CHANGE:f64 = 0.0010;

lazy_static!(
    static ref TOKEN_PROGRAM_ID: Pubkey = Pubkey::from_str("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA").unwrap();
    static ref SYSVAR_RENT_PUBKEY: Pubkey = Pubkey::from_str("SysvarRent111111111111111111111111111111111").unwrap();
);

static mut BID_CLIENT_ID: u64 = 0;
static mut ASK_CLIENT_ID: u64 = 0;

static mut FIRST_LOAD_COMPLETE: bool = false;

pub struct OpenBookSplUsdc<'a> {
    rpc_client: RpcClient,
    //market_builder: MarketBuilder<'a>,
    sol_usdc_market: MarketWrapper<'a>,
    jupiter_pricing_source: JupiterPricingSource,
    serum_manager: SerumManager,
    mm_account: Account,
    market_ooa: Pubkey,
    base_wallet: Pubkey,
    usdc_wallet: Pubkey,
    last_bid_order: Option<NewOrderInstructionV3>,
    last_ask_order: Option<NewOrderInstructionV3>,
    uuid: Uuid,
    use_jupiter: bool,
    best_bid_price: f64,
    best_ask_price: f64,
    base_ask_amount: f64,
    usdc_bid_amount: f64,
    ask_spread_multiplier: f64,
    bid_spread_multiplier: f64,
    last_placed_bid_price: f64,
    last_placed_ask_price: f64,
}

impl OpenBookSplUsdc<'_> {
    pub fn new(
        rpc_client: RpcClient,
        rpc_client2: RpcClient,//have to do this for the borrow checker to not shout
        market_id: Pubkey,
        jupiter_pricing_source: JupiterPricingSource,
        serum_manager: SerumManager,
        pricing_strategy: &str,
        market_ooa: Pubkey,
        base_wallet: Pubkey,
        usdc_wallet: Pubkey,
        mm_account: Account,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let mut market_builder = MarketBuilder::new(rpc_client2, market_id.clone());

        let market = market_builder.build();

        let mut use_jupiter = false;
        let mut best_bid_price = 0.;
        let mut best_ask_price = 0.;
        if pricing_strategy.eq_ignore_ascii_case("jupiter") {
            use_jupiter = true;
            let base_mint = encode(cast::<[u64;4], [u8;32]>(market.market.lock().unwrap().coin_mint)).into_string();
            if let Some(price) = jupiter_pricing_source.get_usdc_price_for_symbol(&base_mint, 1000) {
                best_bid_price = price;
                best_ask_price = price;
            }
        }

        unsafe {
            BID_CLIENT_ID = rand::random();//todo check if this could/should be replaced
            ASK_CLIENT_ID = rand::random();

            println!("Bid clientId:{} , Ask: {}", BID_CLIENT_ID, ASK_CLIENT_ID);
        }

        let uuid = Uuid::new_v4();

        Ok(Self {
            rpc_client,
            //market_builder,
            sol_usdc_market: market,
            jupiter_pricing_source,
            serum_manager,
            mm_account,
            market_ooa,
            base_wallet,
            usdc_wallet,
            last_bid_order: None,
            last_ask_order: None,
            uuid,
            use_jupiter,
            best_bid_price,
            best_ask_price,
            base_ask_amount: SOL_QUOTE_SIZE,
            usdc_bid_amount: SOL_QUOTE_SIZE,
            ask_spread_multiplier: 1.0012,
            bid_spread_multiplier: 0.9987,
            last_placed_bid_price: 0.0,
            last_placed_ask_price: 0.0,
        })
    }


    fn place_sol_ask(
            &mut self,
            sol_amount: f64,
            price: f64,
            cancel: bool,
        ) -> Result<(), Box<dyn std::error::Error>>
    {
        let sol_usdc_market = &self.sol_usdc_market;
        let market_lock = self.sol_usdc_market.market.lock().unwrap();

        let mut instructions = vec![
            ComputeBudgetInstruction::set_compute_unit_price(151_420),
            ComputeBudgetInstruction::set_compute_unit_limit(54_800),
            {
                let mut tmp = consume_events(
                    &SERUM_PROGRAM_ID_V3,
                    vec![&self.market_ooa],
                    &pub_key(market_lock.own_address),
                    &pub_key(market_lock.event_q),
                    &self.base_wallet,
                    &self.usdc_wallet,
                    5,//this value was found inside the SerumProgram.java implementation
                ).unwrap();
                tmp.accounts.push(AccountMeta::new(self.mm_account.owner, true));
                tmp
            }
        ];

        let ask_order = {
            let mut order = NewOrderInstructionV3 {
                side: Side::Ask,
                limit_price: NonZeroU64::new(1).unwrap(),
                max_coin_qty: NonZeroU64::new(1).unwrap(),
                order_type: OrderType::PostOnly,
                client_order_id: unsafe { ASK_CLIENT_ID },
                self_trade_behavior: SelfTradeBehavior::DecrementTake,
                max_native_pc_qty_including_fees: NonZeroU64::new(1).unwrap(),
                limit: 5,//todo what should limit's value be??????
            };

            self.serum_manager.set_order_prices(&mut order, &sol_usdc_market, price, sol_amount);

            order
        };

        if cancel {
            instructions.push(
                cancel_order_by_client_order_id(
                    &serum_utils::SERUM_PROGRAM_ID_V3,
                    &pub_key(market_lock.own_address),
                    &pub_key(market_lock.bids),
                    &pub_key(market_lock.asks),
                    &self.market_ooa,
                    self.mm_account.owner(),
                    &pub_key(market_lock.event_q),
                    unsafe { ASK_CLIENT_ID },
                ).unwrap()
            );
        }

        instructions.push(settle_funds(
            &SERUM_PROGRAM_ID_V3,
            &pub_key(market_lock.own_address),
            &TOKEN_PROGRAM_ID,
            &self.market_ooa,
            self.mm_account.owner(),
            &pub_key(market_lock.coin_vault),
            &self.base_wallet,
            &pub_key(market_lock.pc_vault),
            &self.usdc_wallet,
            Some(&self.usdc_wallet),
            &SerumUtils::get_vault_signer(&sol_usdc_market),
        ).unwrap());

        instructions.push(serum_dex::instruction::new_order(
            &pub_key(market_lock.own_address),
            &self.market_ooa,
            &pub_key(market_lock.req_q),
            &pub_key(market_lock.event_q),
            &pub_key(market_lock.bids),
            &pub_key(market_lock.asks),
            &self.base_wallet,
            &self.mm_account.owner,
            &pub_key(market_lock.coin_vault),
            &pub_key(market_lock.pc_vault),
            &TOKEN_PROGRAM_ID,
            &SYSVAR_RENT_PUBKEY,
            None,
            &SERUM_PROGRAM_ID_V3,
            ask_order.side,
            ask_order.limit_price,
            ask_order.max_coin_qty,
            ask_order.order_type,
            ask_order.client_order_id,
            ask_order.self_trade_behavior,
            ask_order.limit,//todo check this out
            ask_order.max_native_pc_qty_including_fees,
        ).unwrap());

        //found inside org.p2p.solanaj.programs.MemoProgram
        let program_id = Pubkey::from_str("Memo1UhkJRfHyvLMcVucJwxXeuD728EqVDDwQDxFMNo").unwrap();
        instructions.push(
            Instruction::new_with_bytes(
                program_id,
                "Liquidity by Arcana".as_bytes(),
                vec![AccountMeta::new(self.mm_account.owner, true)],
            )
        );
        drop(program_id);

        let mut place_tx = Transaction::new_with_payer(
            &instructions,
            Some(&self.mm_account.owner));//todo check if this value for payer is correct...

        let result = self.rpc_client.send_and_confirm_transaction(&place_tx);

        match result {
            Ok(signature) => {
                println!(
                    "Base Ask: {} @ {}, Tx Signature: {:?}",
                    price,
                    sol_amount,
                    signature
                );
                // Update lastAskOrder
                // last_ask_order = ask_order.clone();
                self.last_ask_order = Some(ask_order);
            }
            Err(err) => {
                eprintln!("OrderTx Error: {}", err);
            }
        }

        Ok(())
    }

    fn place_usdc_bid(
            &mut self,
            amount: f64,
            price: f64,
            cancel: bool,
        ) -> Result<(), Box<dyn std::error::Error>>
    {
        let sol_usdc_market = &self.sol_usdc_market;
        let market_lock = self.sol_usdc_market.market.lock().unwrap();
        let mut instructions = vec![
            ComputeBudgetInstruction::set_compute_unit_price(151_420),
            ComputeBudgetInstruction::set_compute_unit_limit(54_800),
            {
                let mut tmp = consume_events(
                    &SERUM_PROGRAM_ID_V3,
                    vec![&self.market_ooa],
                    &pub_key(market_lock.own_address),
                    &pub_key(market_lock.event_q),
                    &self.base_wallet,
                    &self.usdc_wallet,
                    5,//this value was found inside the SerumProgram.java implementation
                ).unwrap();
                tmp.accounts.push(AccountMeta::new(self.mm_account.owner, true));
                tmp
            }
        ];

        let bid_order = {
            let mut order = NewOrderInstructionV3 {
                side: Side::Bid,
                limit_price: NonZeroU64::new(1).unwrap(),
                max_coin_qty: NonZeroU64::new(1).unwrap(),
                order_type: OrderType::PostOnly,
                client_order_id: unsafe { BID_CLIENT_ID },
                self_trade_behavior: SelfTradeBehavior::DecrementTake,
                max_native_pc_qty_including_fees: NonZeroU64::new(1).unwrap(),
                limit: 5,//todo what should limit's value be??????
            };

            self.serum_manager.set_order_prices(&mut order, &sol_usdc_market, price, amount);

            order
        };

        if cancel {
            instructions.push(
                cancel_order_by_client_order_id(
                    &SERUM_PROGRAM_ID_V3,
                    &pub_key(market_lock.own_address),
                    &pub_key(market_lock.bids),
                                  &pub_key(market_lock.asks),
                    &self.market_ooa,
                    self.mm_account.owner(),
                    &pub_key(market_lock.event_q),
                    unsafe { BID_CLIENT_ID },
                ).unwrap()
            );
        }

        instructions.push(settle_funds(
            &SERUM_PROGRAM_ID_V3,
            &pub_key(market_lock.own_address),
            &TOKEN_PROGRAM_ID,
            &self.market_ooa,
            self.mm_account.owner(),
            &pub_key(market_lock.coin_vault),
            &self.base_wallet,
            &pub_key(market_lock.pc_vault),
            &self.usdc_wallet,
            Some(&self.usdc_wallet),
            &SerumUtils::get_vault_signer(&sol_usdc_market),
        ).unwrap());

        instructions.push(serum_dex::instruction::new_order(
            &pub_key(market_lock.own_address),
            &self.market_ooa,
            &pub_key(market_lock.req_q),
            &pub_key(market_lock.event_q),
            &pub_key(market_lock.bids),
            &pub_key(market_lock.asks),
            &self.usdc_wallet,
            &self.mm_account.owner,
            &pub_key(market_lock.coin_vault),
            &pub_key(market_lock.pc_vault),
            &TOKEN_PROGRAM_ID,
            &SYSVAR_RENT_PUBKEY,
            None,
            &SERUM_PROGRAM_ID_V3,
            bid_order.side,
            bid_order.limit_price,
            bid_order.max_coin_qty,
            bid_order.order_type,
            bid_order.client_order_id,
            bid_order.self_trade_behavior,
            bid_order.limit,//todo check this out
            bid_order.max_native_pc_qty_including_fees,
        ).unwrap());

        //program_id key found inside org.p2p.solanaj.programs.MemoProgram
        let program_id = Pubkey::from_str("Memo1UhkJRfHyvLMcVucJwxXeuD728EqVDDwQDxFMNo").unwrap();
        instructions.push(
            Instruction::new_with_bytes(
                program_id,
                "Liquidity by Arcana".as_bytes(),
                vec![AccountMeta::new(self.mm_account.owner, true)],
            )
        );
        drop(program_id);

        let mut place_tx = Transaction::new_with_payer(
            &instructions,
            Some(&self.mm_account.owner));//todo check if this value for payer is correct...

        let result = self.rpc_client.send_and_confirm_transaction(&place_tx);

        match result {
            Ok(signature) => {
                println!(
                    "Base Ask: {} @ {}, Tx Signature: {:?}",
                    amount,
                    price,
                    signature
                );
                // Update lastAskOrder
                self.last_ask_order = Some(bid_order);
            }
            Err(err) => {
                eprintln!("OrderTx Error: {}", err);
            }
        }
        Ok(())
    }
}

impl Strategy for OpenBookSplUsdc<'_> {
    fn uuid(&self) -> Uuid {
        return self.uuid;
    }

    fn start(&mut self, executor: &Runtime) {
        //let delay = Duration::from_millis(EVENT_LOOP_INITIAL_DELAY_MS);
        let duration = Duration::from_millis(EVENT_LOOP_DURATION_MS);
        let jupiter_pricing_source_clone = self.jupiter_pricing_source.clone();

        executor.block_on(async move {
            let mut interval = interval(duration);
            loop {
                interval.tick().await;
                //let mut sol_usdc_market = &mut self.sol_usdc_market;
                self.sol_usdc_market.reload();
                let market_lock = self.sol_usdc_market.market.lock().unwrap();


                if self.use_jupiter {
                    if let Some(price) = jupiter_pricing_source_clone.get_usdc_price_for_symbol(&*pub_key(market_lock.coin_mint).to_string(), 1000){
                        self.best_bid_price = price;
                        self.best_ask_price = price;
                    }
                }
                else {
                    self.best_bid_price = self.sol_usdc_market.bid_order_book.get_best_bid_price();
                    self.best_ask_price = self.sol_usdc_market.ask_order_book.get_best_ask_price();
                }

                //dropping so that self can be borrowed as mutable later
                drop(market_lock);

                //todo this most likely does not work
                // let bids_pubkey = pub_key(market_lock.bids);
                // let bid_account_info = account_info(bids_pubkey, self.rpc_client.get_account_with_commitment(&bids_pubkey, CommitmentConfig::processed()).unwrap().value.unwrap());
                // let orders = market_lock.load_orders_mut(//todo implement this
                //     &bid_account_info,
                //     Some(&bid_account_info),
                //     &SERUM_PROGRAM_ID_V3,
                //     None,
                //     None
                // );
                // let is_cancel_bid = orders.is_ok();
                let is_cancel_bid = false;

                let percentage_change_from_last_bid =
                1.0 - (self.last_placed_bid_price / (self.best_bid_price * self.bid_spread_multiplier));

                // Only place bid if we haven't placed, or the change is >= 0.1% change
                if self.last_placed_bid_price == 0. || percentage_change_from_last_bid.abs() >= MIN_MIDPOINT_CHANGE {
                    self.place_usdc_bid(self.usdc_bid_amount, self.best_bid_price * self.bid_spread_multiplier, is_cancel_bid).unwrap();
                    self.last_placed_bid_price = self.best_bid_price * self.bid_spread_multiplier;
                }

                // //todo this most likely does not work
                // let asks_pubkey = pub_key(market_lock.asks);
                // let ask_account_info = serum_utils::account_info(asks_pubkey, self.rpc_client.get_account_with_commitment(&asks_pubkey, CommitmentConfig::processed()).unwrap().value.unwrap());
                // let orders = market_lock.load_orders_mut(
                //     &ask_account_info,
                //     Some(&ask_account_info),
                //     &SERUM_PROGRAM_ID_V3,
                //     None,
                //     None
                // );

                //let is_cancel_ask = orders.is_ok();
                let is_cancel_ask = false;

                let percentage_change_from_last_ask =
                    1.0 - (self.last_placed_ask_price / (self.best_ask_price * self.ask_spread_multiplier));

                if self.last_placed_ask_price == 0. || percentage_change_from_last_ask.abs() >= MIN_MIDPOINT_CHANGE {
                    self.place_sol_ask(self.usdc_bid_amount, self.best_bid_price * self.bid_spread_multiplier, is_cancel_ask).unwrap();
                    self.last_placed_bid_price = self.best_ask_price * self.ask_spread_multiplier;
                }

                if unsafe { !FIRST_LOAD_COMPLETE } {
                    println!("Sleeping 2000ms");
                    sleep(Duration::from_millis(2000)).await;
                    println!("First Load Complete");

                    unsafe { FIRST_LOAD_COMPLETE = true; }
                }
            }
        });
    }
}


//todo fix this (all of these Send+Sync errors are because of the MarketState struct)
unsafe impl Send for OpenBookSplUsdc<'_> {}
unsafe impl Sync for OpenBookSplUsdc<'_> {}