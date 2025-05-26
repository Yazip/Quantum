use redis::aio::Connection;
use redis::Client;

pub async fn init_redis() -> Connection {
    let redis_url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1/".to_string());
    let client = Client::open(redis_url).expect("Invalid Redis URL");
    client
	.get_async_connection()
        .await
        .expect("Failed to connect to Redis")
}