use solana_program::pubkey::Pubkey;

pub struct OpenOrdersAccount {
    own_pub_key: Pubkey,
}

impl OpenOrdersAccount {
    pub fn read_open_orders_account(data: Vec<u8>) -> OpenOrdersAccount {

    }

    pub fn set_own_pubkey(&mut self, key: Pubkey) {
        self.own_pub_key = key;
    }
}