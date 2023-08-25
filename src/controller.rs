use rocket_dyn_templates::context;
use rocket_dyn_templates::Template;
use crate::AppState;
use rocket::get;
use rocket::post;

#[get("/")]
pub fn index(state: &rocket::State<AppState>) -> Template {
    let context = context! { title: "Arcana-rs" };
    Template::render("index", &context)
}

#[get("/settings")]
pub fn settings(state: &rocket::State<AppState>) -> Template {
    let context = context! {
        title: "Settings",
        rpc_endpoint: state.rpc_client.url(),
        trading_account_pubkey: state.bot_manager.trading_account.as_ref().unwrap().owner.to_string(),
    };
    Template::render("settings", &context)
}
//todo need to rewrite .jsp into tera