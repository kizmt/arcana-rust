use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::program_pack::Pack;
use solana_sdk::rpc::RpcError;
use solana_sdk::signature::Keypair;
use solana_sdk::transaction::Transaction;
use solana_transaction_status::TransactionResult;
use std::error::Error;
use std::str::FromStr;
use rocket::serde::json::from_str;
use solana_client::rpc_config::RpcProgramAccountsConfig;
use solana_client::rpc_filter::{MemcmpEncodedBytes, RpcFilterType, Memcmp};
use solana_sdk::bs58;
use crate::serum_dex::model::market::Market;
use crate::serum_dex::model::open_orders_account::OpenOrdersAccount;

const LAMPORTS_PER_SOL:                 u64 = 1_000_000_000;
const OWN_ADDRESS_OFFSET:               usize = 13;
pub(crate) const TOKEN_MINT_DECIMALS_OFFSET:       usize = 44;
const VAULT_SIGNER_NONCE_OFFSET:        usize = 45;
const BASE_MINT_OFFSET:                 usize = 53;
const QUOTE_MINT_OFFSET:                usize = 85;
const BASE_VAULT_OFFSET:                usize = 117;
const BASE_DEPOSITS_TOTAL_OFFSET:       usize = 149;
const BASE_FEES_ACCRUED_OFFSET:         usize = 157;
const QUOTE_VAULT_OFFSET:               usize = 165;
const QUOTE_DEPOSITS_TOTAL_OFFSET:      usize = 197;
const QUOTE_FEES_ACCRUED_OFFSET:        usize = 205;
const QUOTE_DUST_THRESHOLD_OFFSET:      usize = 213;
const REQUEST_QUEUE_OFFSET:             usize = 221;
const EVENT_QUEUE_OFFSET:               usize = 253;
const BIDS_OFFSET:                      usize = 285;
const ASKS_OFFSET:                      usize = 317;
const BASE_LOT_SIZE_OFFSET:             usize = 349;
const QUOTE_LOT_SIZE_OFFSET:            usize = 357;
const FEE_RATE_BPS_OFFSET:              usize = 365;
const REFERRER_REBATES_ACCRUED_OFFSET:  usize = 373;
const MARKET_ACCOUNT_SIZE:              usize = 388;

pub const SERUM_PROGRAM_ID_V3:Pubkey = Pubkey::from_str("srmqPvymJeFKQ4zGQed1GFppgkRHL9kaELCbyksJtPX").unwrap();
pub const WRAPPED_SOL_MINT:Pubkey = Pubkey::from_str("So11111111111111111111111111111111111111112").unwrap();
pub const USDC_MINT: Pubkey = Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v").unwrap();
pub const USDT_MINT: Pubkey = Pubkey::from_str("Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB").unwrap();


pub struct SerumUtils;

impl SerumUtils {
    pub fn price_number_to_lots_market(price: f64, market: &Market) -> i64 {
        return SerumUtils::price_number_to_lots(price,
                                    market.quote_decimals,
                                    market.base_lot_size,
                                    market.base_decimals,
                                    market.quote_lot_size);
    }

    pub fn price_number_to_lots(price: f64, quote_decimals: i8, base_lot_size: i64, base_decimals: i8, quote_lot_size: i64) -> i64 {
        let top = (price * 10f64.powi(quote_decimals as i32) * (base_lot_size as f64)) as f64;
        let bottom = 10f64.powi(base_decimals as i32) * quote_lot_size as f64;
        (top / bottom).ceil() as i64
    }

    pub fn base_size_number_to_lots(size: f64, base_decimals: i8, base_lot_size: i64) -> i64 {
        let top = (size * 10f64.powi(base_decimals as i32)).round() as f64;
        (top / base_lot_size as f64).ceil() as i64
    }

    pub fn get_max_quote_quantity(price: f64, size: f64, market: &Market) -> i64 {
        let base_size_lots = SerumUtils::base_size_number_to_lots(size, market.base_decimals, market.base_lot_size);
        let price_lots = SerumUtils::price_number_to_lots_market(price, market);

        market.quote_lot_size * base_size_lots * price_lots
    }

    pub(crate) fn read_decimals_from_token_mint_data(account_data: &Vec<u8>) -> u8 {
        return account_data[TOKEN_MINT_DECIMALS_OFFSET];
    }




    pub fn read_own_address_pubkey(bytes: &[u8]) -> Pubkey {
        return Pubkey::try_from_slice(bytes[OWN_ADDRESS_OFFSET..OWN_ADDRESS_OFFSET + 32].clone());
    }

    pub fn find_open_orders_account_for_owner(client: &RpcClient, market_address: &Pubkey, owner_address: &Pubkey, ) -> Option<OpenOrdersAccount> {
        let data_size:u64 = 3228;

        let market_filter = RpcFilterType::Memcmp(Memcmp::new(OWN_ADDRESS_OFFSET, MemcmpEncodedBytes::Bytes(Vec::from(market_address.to_bytes()))));
        let owner_filter = RpcFilterType::Memcmp(Memcmp::new(45, MemcmpEncodedBytes::Bytes(Vec::from(owner_address.to_bytes()))));//todo check if this conversion from Pubkey toMemcmpEncodedBytes is as expected
        let data_size_filter = RpcFilterType::DataSize(data_size);

        let config = RpcProgramAccountsConfig {
            filters: Some(vec![market_filter, owner_filter, data_size_filter]),
            account_config: Default::default(),
            with_context: None,
        };

        let program_accounts = client.get_program_accounts_with_config(&SERUM_PROGRAM_ID_V3, config);
        if program_accounts.is_err() {
            eprintln!("{}", program_accounts.err().unwrap());
            return None;
        }

        if let Some((key, program_account)) = program_accounts.unwrap().into_iter().next() {
            let data = program_account.data;
            let mut open_orders_account = OpenOrdersAccount::read_open_orders_account(data);
            open_orders_account.set_own_pubkey(key);

            return Some(open_orders_account);
        } else {
            return None;
        }
    }


    // pub fn read_vault_signer_nonce(bytes: &[u8]) -> Result<u64, Box<dyn Error>> {
    //     // Implement the function here
    // }
    //
    // pub fn read_base_deposits_total(bytes: &[u8]) -> Result<u64, Box<dyn Error>> {
    //     // Implement the function here
    // }
    //
    // pub fn read_base_fees_accrued(bytes: &[u8]) -> Result<u64, Box<dyn Error>> {
    //     // Implement the function here
    // }
    //
    // pub fn read_quote_vault_offset(bytes: &[u8]) -> Result<Pubkey, Box<dyn Error>> {
    //     // Implement the function here
    // }
}