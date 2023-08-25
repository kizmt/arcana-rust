use solana_client::rpc_client::RpcClient;
use solana_client::rpc_config::RpcSendTransactionConfig;
use solana_program::system_instruction;
use solana_program::system_program;
use solana_program::token_instruction;
use solana_sdk::account::Account;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Keypair;
use solana_sdk::transaction::Transaction;
use solana_token::instruction as token_instruction;
use std::error::Error;
use std::str::FromStr;
use crate::serum_dex::model::market::Market;
use crate::serum_dex::model::order::Order;
use crate::serum_dex::model::serum_utils::SerumUtils;

struct SerumManager {
    client: RpcClient,
}

const MINIMUM_BALANCE_FOR_RENT_EXEMPTION_165: u64 = 2039280;
const REQUIRED_ACCOUNT_SPACE: u64 = 165;
const OPEN_ORDERS_ACCOUNT_DATA_SIZE: u64 = 3228;
const OPEN_ORDERS_MINIMUM_BALANCE_FOR_RENT_EXEMPTION: u64 = 23357760;

impl SerumManager {
    pub fn new(client: RpcClient) -> Self {
        Self { client }
    }

    pub fn set_order_prices(mut order: &mut Order, market: Market) {
        let long_price = SerumUtils::price_number_to_lots_market(order.float_price, &market);

        let long_quantity = SerumUtils::base_size_number_to_lots(order.float_quantity, market.base_decimals, market.base_lot_size);

        let max_quote_quantity = SerumUtils::get_max_quote_quantity(order.float_price, order.float_quantity, &market);

        order.price = long_price;
        order.quantity = long_quantity;
        order.max_quote_quantity = max_quote_quantity + 1;
    }








    pub fn place_order(&self, account: &Keypair, market: &Market, order: &Order, base_wallet: &Pubkey, quote_wallet: &Pubkey) -> Result<String, Box<dyn Error>> {
        self.validate_order(order);

        let open_orders = SerumUtils::find_open_orders_account_for_owner(&self.client, &market.own_address(), &account.pubkey())?;

        return self.place_order_internal(account, market, order, base_wallet, quote_wallet, &open_orders, None);
    }

    pub fn place_order_with_open_orders(&self, account: &Keypair, market: &Market, order: &Order, base_wallet: &Pubkey, quote_wallet: &Pubkey, open_orders_account: &OpenOrdersAccount) -> Result<String, Box<dyn Error>> {
        self.validate_order(order);

        return self.place_order_internal(account, market, order, base_wallet, quote_wallet, open_orders_account, None);
    }

    // ... other methods ...

    fn place_order_internal(
        &self,
        account: &Keypair,
        market: &Market,
        order: &Order,
        base_wallet: &Pubkey,
        quote_wallet: &Pubkey,
        open_orders_account: &OpenOrdersAccount,
        srm_fee_discount: Option<Pubkey>,
    ) -> Result<String, Box<dyn Error>>
    {
        let mut transaction = Transaction::new_with_payer(&[...], Some(&account.pubkey()));

        // ... rest of the function ...

        self.send_transaction_with_signers(&mut transaction, &[account])
    }

    // ... other methods ...

    fn send_transaction_with_signers(&self, transaction: &mut Transaction, signers: &[&Keypair], ) -> Result<String, Box<dyn Error>> {
        let config = RpcSendTransactionConfig {
            preflight_commitment: None,
            encoding: None,
        };

        let result = self
            .client
            .send_transaction_with_config(transaction, signers, config);

        match result {
            Ok(signature) => Ok(signature.to_string()),
            Err(err) => Err(Box::new(err)),
        }
    }




    pub fn place_order_with_open_orders_and_fee_discount(
        &self,
        account: &Keypair,
        market: &Market,
        order: &Order,
        base_wallet: &Pubkey,
        quote_wallet: &Pubkey,
        open_orders_account: &OpenOrdersAccount,
        srm_fee_discount: Pubkey,
    ) -> Result<String, Box<dyn Error>>
    {
        self.validate_order(order);

        self.place_order_internal(
            account,
            market,
            order,
            base_wallet,
            quote_wallet,
            open_orders_account,
            Some(srm_fee_discount),
        )
    }

    // ... other methods ...

    pub fn settle_funds_with_open_orders(
        &self,
        market: &Market,
        account: &Keypair,
        base_wallet: &Pubkey,
        quote_wallet: &Pubkey,
        open_orders_account: &OpenOrdersAccount,
    ) -> Result<String, Box<dyn Error>>
    {
        self.validate_open_orders_account(open_orders_account);
        self.settle_funds_internal(market, account, base_wallet, quote_wallet, open_orders_account)
    }

    pub fn settle_funds_for_market(
        &self,
        market: &Market,
        account: &Keypair,
        base_wallet: &Pubkey,
        quote_wallet: &Pubkey,
    ) -> Result<String, Box<dyn Error>>
    {
        let open_orders_account = SerumUtils::find_open_orders_account_for_owner(
            &self.client,
            &market.own_address,
            &account.pubkey(),
        )?;
        self.validate_open_orders_account(&open_orders_account);
        self.settle_funds_internal(market, account, base_wallet, quote_wallet, &open_orders_account)
    }
}