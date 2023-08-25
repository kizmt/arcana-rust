use std::sync::{Arc, Mutex};
use solana_sdk::account::Account;
use crate::strategies::open_book_bot::OpenBookBot;
use crate::strategies::strategy::Strategy;

pub struct BotManager {
    pub trading_account: Option<Account>,
    bot_list: Vec<OpenBookBot>,
}

impl BotManager {
    pub fn new() -> Self {
        Self {
            trading_account: None,
            bot_list: Vec::new(),
        }
    }

    pub fn add_bot(&mut self, mut bot: OpenBookBot) {
        let executor = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(1)
            .build().expect("Error building a tokio runtime for strategy executor");
        bot.strategy.as_mut().unwrap().start(&executor);
        bot.strategy_executor = Some(executor);
        self.bot_list.push(bot);
    }

    pub fn get_bot_list(&self) -> &Vec<OpenBookBot> {
        &self.bot_list
    }
}
