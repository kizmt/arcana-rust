use std::sync::{Arc, Mutex};
use serum_dex::state::{Market, MarketState};
use crate::serum::market_builder::MarketBuilder;
use crate::serum::order_book::OrderBook;

//make sure while that this struct is Send and sync
pub struct MarketWrapper<'a> {
    pub market: Arc<Mutex<Market<'a>>>,
    pub base_decimals: i8,
    pub quote_decimals: i8,
    pub bid_order_book: OrderBook,
    pub ask_order_book: OrderBook,
}

impl MarketWrapper<'_> {
    pub fn reload(&mut self) {
        //todo update the order books here
        //self.market.lock().unwrap().load_bids_mut()
        //self.bid_order_book = market.bid_order_book;
        //self.ask_order_book = market.ask_order_book;
        //event queue isn't used anywhere
    }
}

impl Clone for MarketWrapper<'_> {
    fn clone(&self) -> Self {
        Self {
            market: self.market.clone().into(),
            base_decimals: self.base_decimals,
            quote_decimals: self.quote_decimals,
            bid_order_book: self.bid_order_book.clone(),
            ask_order_book: self.ask_order_book.clone(),
        }
    }
}

unsafe impl Send for MarketWrapper<'_> {}
unsafe impl Sync for MarketWrapper<'_> {}