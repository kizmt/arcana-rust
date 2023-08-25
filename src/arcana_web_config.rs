//use rocket::request::FromData;
//use rocket::response::NamedFile;
use solana_client::rpc_client::RpcClient;

pub(crate) fn rpc_client() -> RpcClient {
    let rpc_client = RpcClient::new("https://try-rpc.mainnet.solana.blockdaemon.tech");
    //let rpc_client = RpcClient::new("http://localhost:8899".to_string()); //todo Replace with your Solana RPC URL

    rpc_client
}

struct SerumManager {
    rpc_client: RpcClient,
}

impl SerumManager {
    // Implement your SerumManager methods here
}
//
// #[derive(Serialize)]
// struct Context {
//     title: &'static str,
//     // Add other context data here
// }
//
// #[get("/")]
// fn index() -> Template {
//     let context = Context { title: "Arcana" };
//     Template::render("index", &context)
// }

// fn main() {
//     let config = Config::build(Environment::Development)
//         .address("127.0.0.1")
//         .port(8000)
//         .finalize()
//         .unwrap();
//
//     rocket::custom(config)
//         .attach(AdHoc::on_attach("Database Configuration", |rocket| {
//             Ok(rocket.manage(SerumManager {
//                 rpc_client: RpcClient,
//             }))
//         }))
//         .attach(Template::fairing())
//         .mount("/", StaticFiles::from("static"))
//         .mount("/", routes![index])
//         .launch();
// }
