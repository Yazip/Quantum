use std::sync::Arc;
use tokio::sync::Mutex;

mod ws;
mod auth;
mod db;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    env_logger::init();

    let pool = db::init_pool().await;

    let addr = "127.0.0.1:9001";
    let redis = Arc::new(Mutex::new(db::redis::init_redis().await));
    ws::server::run_ws_server(addr, pool, redis).await;
}