mod ws;
mod auth;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    env_logger::init();

    let addr = "127.0.0.1:9001";
    ws::server::run_ws_server(addr).await;
}