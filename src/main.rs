//#![feature(proc_macro_hygiene, decl_macro)] todo check this macro out

use std::net::IpAddr;
use rocket::config::Config;
use rocket::fairing::AdHoc;
use rocket_dyn_templates::{context, Template};
use rocket::routes;
use rocket::get;
use solana_client::rpc_client::RpcClient;

//PC MINT IS QUOTE CURRENCY
//COIN MINT IS THE BASE CURRENCY

mod pricing {
    pub mod jupiter_pricing_source;
    pub mod pyth_pricing_source;
}
mod strategies {
    pub mod bot_manager;
    pub mod open_book_bot;
    pub mod strategy;
    mod open_book {
        pub mod open_book_spl_usdc;
    }
}
pub mod arcana_web_config;
pub mod controller;

mod serum {
    pub mod serum_utils;
    pub mod serum_manager;
    pub mod market;
    pub mod market_builder;
    pub mod order_book;
}

use pricing::jupiter_pricing_source::JupiterPricingSource;
use pricing::pyth_pricing_source::PythPricingSource;
use crate::strategies::bot_manager::BotManager;

struct AppState {
    jupiter_pricing_source: JupiterPricingSource,
    pyth_pricing_source: PythPricingSource,
    bot_manager: BotManager,
    rpc_client: RpcClient,
}

fn main() {
    let mut config = Config::debug_default();
    config.address = IpAddr::V4("0.0.0.0".parse().unwrap());
    config.port = 8080;

    let rpc_client = arcana_web_config::rpc_client();
    let jupiter_pricing_source = JupiterPricingSource::new();
    let pyth_pricing_source = PythPricingSource::new(rpc_client); // You would need to implement this
    // let strategy_manager = StrategyManager::new();
    let bot_manager = BotManager::new();

    rocket::custom(config)
        .attach(AdHoc::on_ignite("State Configuration", |rocket| async move {
            rocket.manage(AppState {
                jupiter_pricing_source,
                pyth_pricing_source,
                bot_manager,
                rpc_client: arcana_web_config::rpc_client(),
            })
        }))
        .attach(Template::fairing())
        .mount("/", routes![controller::index, controller::settings])
        .launch();
}
