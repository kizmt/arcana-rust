use solana_sdk::pubkey::Pubkey;

#[derive(Debug)]
pub struct Order {
    pub price: i64,
    pub quantity: i64,
    pub client_order_id: i64,
    pub float_price: f64,
    pub float_quantity: f64,
    pub owner: Pubkey,
    pub max_quote_quantity: i64,
    pub client_id: i64,
    pub order_type_layout: OrderTypeLayout,
    pub self_trade_behavior_layout: SelfTradeBehaviorLayout,
    pub buy: bool,
}

impl Order {
    pub fn new(
        price: i64,
        quantity: i64,
        client_order_id: i64,
        float_price: f32,
        float_quantity: f32,
        owner: Pubkey,
        max_quote_quantity: i64,
        client_id: i64,
        order_type_layout: OrderTypeLayout,
        self_trade_behavior_layout: SelfTradeBehaviorLayout,
        buy: bool,
    ) -> Self
    {
        Order {
            price,
            quantity,
            client_order_id,
            float_price,
            float_quantity,
            owner,
            max_quote_quantity,
            client_id,
            order_type_layout,
            self_trade_behavior_layout,
            buy,
        }
    }
}

impl std::fmt::Display for Order {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Order{{ price={}, quantity={}, client_order_id={}, float_price={}, float_quantity={}, owner={}, max_quote_quantity={}, client_id={}, order_type_layout={:?}, self_trade_behavior_layout={:?}, buy={} }}",
            self.price, self.quantity, self.client_order_id, self.float_price, self.float_quantity, self.owner, self.max_quote_quantity, self.client_id, self.order_type_layout, self.self_trade_behavior_layout, self.buy
        )
    }
}
