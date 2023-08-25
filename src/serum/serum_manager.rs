use std::num::NonZeroU64;
use serum_dex::instruction::NewOrderInstructionV3;
use crate::serum::market::MarketWrapper;
use crate::serum::serum_utils::SerumUtils;

pub struct SerumManager;

impl SerumManager {
    pub fn set_order_prices(&self, order: &mut NewOrderInstructionV3, market: &MarketWrapper, price: f64, amount: f64) {
        let long_price = SerumUtils::price_number_to_lots_market(price, market);
        let qty = SerumUtils::base_size_number_to_lots(amount, market.base_decimals, market.market.lock().unwrap().pc_lot_size);
        let max_quote_qty = SerumUtils::get_max_quote_quantity(price, amount, market);

        order.max_coin_qty = NonZeroU64::new(max_quote_qty + 1).unwrap();
        order.limit_price = NonZeroU64::new(long_price).unwrap();
        order.max_native_pc_qty_including_fees = NonZeroU64::new(qty).unwrap();//todo check if these set values are correct
    }
}