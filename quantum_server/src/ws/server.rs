use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;
use tungstenite::protocol::Message;
use std::net::SocketAddr;
use futures_util::{StreamExt, SinkExt};
use crate::auth;
use std::env;

pub async fn run_ws_server(addr: &str) {
    let listener = TcpListener::bind(addr).await.expect("Failed to bind");

    println!("WebSocket server running on {}", addr);

    while let Ok((stream, addr)) = listener.accept().await {
        tokio::spawn(handle_connection(stream, addr));
    }
}

async fn handle_connection(stream: tokio::net::TcpStream, addr: SocketAddr) {
    let ws_stream = accept_async(stream).await.expect("WebSocket handshake failed");

    println!("New WebSocket connection from {}", addr);

    let (mut write, mut read) = ws_stream.split();

    while let Some(msg) = read.next().await {
        if let Ok(Message::Text(ref text)) = msg {
            if let Ok(json_msg) = serde_json::from_str::<serde_json::Value>(&text) {
                match json_msg["type"].as_str() {
                    Some("auth") => {
                        let token = json_msg["token"].as_str().unwrap_or("");
                        let secret = env::var("JWT_SECRET").unwrap_or("mysecret".to_string());
                        match auth::verify_jwt(token, &secret) {
                            Ok(data) => {
                                println!("User authenticated: {}", data.claims.sub);
                                let _ = write.send(Message::Text(r#"{"status": "authenticated"}"#.into())).await;
                            }
                            Err(_) => {
                                let _ = write.send(Message::Text(r#"{"error": "invalid_token"}"#.into())).await;
                                break;
                            }
                        }
                    }
                    _ => {
                        match msg {
                            Ok(Message::Text(text)) => {
                                println!("Received: {}", text);
                                let response = format!("Echo: {}", text);
                                let _ = write.send(Message::Text(response)).await;
                            }
                            Ok(Message::Binary(_)) => {}
                            Ok(Message::Close(_)) => break,
                            _ => {}
                        }
                    }
                }
            }
        }
    }

    println!("Connection {} closed", addr);
}