use std::cell::RefMut;
use serum_dex::critbit::{Slab, SlabView};
use serum_dex::matching::OrderBookState;
use crate::serum::serum_utils::SerumUtils;

//order books just contain a price which is updated after every market reload
#[derive(Clone)]
pub struct OrderBook {
    best_price: f64,
    orders: Vec<Order>
}

impl OrderBook {
    pub(crate) fn new(slab: RefMut<Slab>, base_decimals: i8, quote_decimals: i8, base_lot_size: u64, quote_lot_size: u64) -> OrderBook {
        let max = slab.find_max().unwrap();
        let leaf_node = slab.get(max).unwrap().as_leaf().unwrap();

        let best_price = SerumUtils::price_lots_to_number(leaf_node.price().get() as i64, base_decimals, quote_decimals, base_lot_size, quote_lot_size);

        OrderBook {
            best_price
        }
    }
    pub fn get_best_bid_price(&self) -> f64 {
        self.best_price
    }

    pub fn get_best_ask_price(&self) -> f64 {
        self.best_price
    }
}