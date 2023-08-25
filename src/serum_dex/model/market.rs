use std::fmt;
use solana_sdk::pubkey::Pubkey;
use crate::serum_dex::model::order_book::OrderBook;
use crate::serum_dex::model::serum_utils::SerumUtils;

#[derive(Debug, Eq, PartialEq, Hash)]
pub struct Market {
pub    account_flags: AccountFlags,
pub    own_address: Pubkey,
pub    vault_signer_nonce: i64,
pub    base_mint: Pubkey,
pub    quote_mint: Pubkey,
pub    base_vault: Pubkey,
pub    base_deposits_total: i64,
pub    base_fees_accrued: i64,
pub    quote_vault: Pubkey,
pub    quote_deposits_total: i64,
pub    quote_fees_accrued: i64,
pub    quote_dust_threshold: i64,
pub    request_queue: Pubkey,
pub    event_queue_key: Pubkey,
pub    bids: Pubkey,
pub    asks: Pubkey,
pub    base_lot_size: u64,
pub    quote_lot_size: u64,
pub    fee_rate_bps: i64,
pub    referrer_rebates_accrued: i64,
pub    bid_order_book: OrderBook,
pub    ask_order_book: OrderBook,
pub    event_queue: EventQueue,
pub    base_decimals: u8,
pub    quote_decimals: u8,
}

impl Market {
    pub fn read_market(data: &[u8]) -> Market {
        let account_flags = AccountFlags::read_account_flags(data);
        let own_address = SerumUtils::read_own_address_pubkey(data);
        let vault_signer_nonce = SerumUtils::read_vault_signer_nonce(data);
        let base_mint = SerumUtils::read_base_mint_pubkey(data);
        let quote_mint = SerumUtils::read_quote_mint_pubkey(data);
        let base_vault = SerumUtils::read_base_vault_pubkey(data);
        let base_deposits_total = SerumUtils::read_base_deposits_total(data);
        let base_fees_accrued = SerumUtils::read_base_fees_accrued(data);
        let quote_vault = SerumUtils::read_quote_vault_offset(data);
        let quote_deposits_total = SerumUtils::read_quote_deposits_total(data);
        let quote_fees_accrued = SerumUtils::read_quote_fees_accrued(data);
        let quote_dust_threshold = SerumUtils::read_quote_dust_threshold(data);
        let request_queue = SerumUtils::read_request_queue_pubkey(data);
        let event_queue_key = SerumUtils::read_event_queue_pubkey(data);
        let bids = SerumUtils::read_bids_pubkey(data);
        let asks = SerumUtils::read_asks_pubkey(data);
        let base_lot_size = SerumUtils::read_base_lot_size(data);
        let quote_lot_size = SerumUtils::read_quote_lot_size(data);
        let fee_rate_bps = SerumUtils::read_fee_rate_bps(data);
        let referrer_rebates_accrued = SerumUtils::read_referrer_rebates_accrued(data);

        Market {
            account_flags,
            own_address,
            vault_signer_nonce,
            base_mint,
            quote_mint,
            base_vault,
            base_deposits_total,
            base_fees_accrued,
            quote_vault,
            quote_deposits_total,
            quote_fees_accrued,
            quote_dust_threshold,
            request_queue,
            event_queue_key,
            bids,
            asks,
            base_lot_size,
            quote_lot_size,
            fee_rate_bps,
            referrer_rebates_accrued,
            bid_order_book: Default::default(), // Initialize with default values
            ask_order_book: Default::default(), // Initialize with default values
            event_queue: Default::default(),    // Initialize with default values
            base_decimals: 0,                   // Initialize with default value
            quote_decimals: 0,                  // Initialize with default value
        }
    }
}

/*
impl Default for Market {
    fn default() -> Self {
        Market {
            account_flags: Default::default(),
            own_address: Pubkey::new(&[0u8; 32]), // Initialize with default value
            vault_signer_nonce: 0,              // Initialize with default value
            base_mint: Default::default(),
            quote_mint: Default::default(),
            base_vault: Default::default(),
            base_deposits_total: 0,             // Initialize with default value
            base_fees_accrued: 0,               // Initialize with default value
            quote_vault: Default::default(),
            quote_deposits_total: 0,            // Initialize with default value
            quote_fees_accrued: 0,              // Initialize with default value
            quote_dust_threshold: 0,            // Initialize with default value
            request_queue: Default::default(),
            event_queue_key: Default::default(),
            bids: Default::default(),
            asks: Default::default(),
            base_lot_size: 0,                   // Initialize with default value
            quote_lot_size: 0,                  // Initialize with default value
            fee_rate_bps: 0,                    // Initialize with default value
            referrer_rebates_accrued: 0,        // Initialize with default value
            bid_order_book: Default::default(), // Initialize with default values
            ask_order_book: Default::default(), // Initialize with default values
            event_queue: Default::default(),    // Initialize with default values
            base_decimals: 0,                   // Initialize with default value
            quote_decimals: 0,                  // Initialize with default value
        }
    }
}
*/

impl fmt::Display for Market {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Market {{\n")?;
        write!(f, "  accountFlags={:?}\n", self.account_flags)?;
        write!(f, "  ownAddress={}\n", self.own_address)?;
        write!(f, "  vaultSignerNonce={}\n", self.vault_signer_nonce)?;
        write!(f, "  baseMint={}\n", self.base_mint)?;
        write!(f, "  quoteMint={}\n", self.quote_mint)?;
        write!(f, "  baseVault={}\n", self.base_vault)?;
        write!(f, "  baseDepositsTotal={}\n", self.base_deposits_total)?;
        write!(f, "  baseFeesAccrued={}\n", self.base_fees_accrued)?;
        write!(f, "  quoteVault={}\n", self.quote_vault)?;
        write!(f, "  quoteDepositsTotal={}\n", self.quote_deposits_total)?;
        write!(f, "  quoteFeesAccrued={}\n", self.quote_fees_accrued)?;
        write!(f, "  quoteDustThreshold={}\n", self.quote_dust_threshold)?;
        write!(f, "  requestQueue={}\n", self.request_queue)?;
        write!(f, "  eventQueue={}\n", self.event_queue_key)?;
        write!(f, "  bids={}\n", self.bids)?;
        write!(f, "  asks={}\n", self.asks)?;
        write!(f, "  baseLotSize={}\n", self.base_lot_size)?;
        write!(f, "  quoteLotSize={}\n", self.quote_lot_size)?;
        write!(f, "  feeRateBps={}\n", self.fee_rate_bps)?;
        write!(f, "  referrerRebatesAccrued={}\n", self.referrer_rebates_accrued)?;
        write!(f, "  bidOrderBook={:?}\n", self.bid_order_book)?;
        write!(f, "  askOrderBook={:?}\n", self.ask_order_book)?;
        write!(f, "  baseDecimals={}\n", self.base_decimals)?;
        write!(f, "  quoteDecimals={}\n", self.quote_decimals)?;
        write!(f, "}}")
    }
}
