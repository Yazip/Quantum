use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;
use tungstenite::protocol::Message;
use std::net::SocketAddr;
use futures_util::{StreamExt, SinkExt};
use crate::auth;
use std::env;
use crate::db::user::{create_user, authenticate_user, NewUser};
use jsonwebtoken::{encode, Header, EncodingKey};
use crate::auth::Claims;
use chrono::{Utc, Duration};
use serde_json::json;
use sqlx::PgPool;
use crate::db::message::{NewMessage, send_message};
use crate::db::message::get_messages_for_chat;
use uuid::Uuid;
use redis::aio::Connection;
use redis::AsyncCommands;
use serde_json;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::db::message::{edit_message, delete_message};
use crate::db::reaction::set_reaction;
use crate::db::message::forward_message;
use crate::db::chat::create_group_chat;
use crate::db::chat::add_user_to_chat;
use crate::db::chat::remove_user_from_chat;
use crate::db::chat::get_chat_members;
use crate::db::chat::is_user_in_chat;
use crate::db::user::user_exists;

pub async fn run_ws_server(addr: &str, pool: PgPool, redis: Arc<Mutex<Connection>>) {
    let listener = TcpListener::bind(addr).await.expect("Failed to bind");

    println!("WebSocket server running on {}", addr);

    while let Ok((stream, addr)) = listener.accept().await {
        tokio::spawn(handle_connection(stream, addr, pool.clone(), Arc::clone(&redis)));
    }
}

fn generate_jwt(user_id: &str) -> String {
    let secret = std::env::var("JWT_SECRET").unwrap_or("secret".to_string());

    let claims = Claims {
        sub: user_id.to_string(),
        exp: (Utc::now() + Duration::hours(2)).timestamp() as usize,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    ).unwrap()
}

async fn handle_connection(stream: tokio::net::TcpStream, addr: SocketAddr, pool: PgPool, redis: Arc<Mutex<Connection>>) {
    let mut authenticated_user: Option<String> = None;

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
		    
                    Some("register") => {
                        // Пытаемся распарсить NewUser
                        let payload = &json_msg["payload"];
                        let new_user = serde_json::from_value::<NewUser>(payload.clone());

                        match new_user {
                            Ok(user_data) => {
                                match create_user(user_data, &pool).await {
                                    Ok(user) => {
                                        authenticated_user = Some(user.id.to_string());
					let token = generate_jwt(&user.id.to_string());
                                        let response = json!({ "status": "ok", "token": token });
                                        let _ = write.send(Message::Text(response.to_string())).await;
                                    }
                                    Err(e) => {
                                        let err = format!("{{\"error\": \"{}\"}}", e);
                                        let _ = write.send(Message::Text(err)).await;
                                    }
                                }
                            }
                            Err(_) => {
                                let _ = write.send(Message::Text("{\"error\": \"invalid payload\"}".to_string())).await;
                            }
                        }
                    }

                    Some("login") => {
                        let payload = &json_msg["payload"];
                        let username = payload["username"].as_str().unwrap_or("");
                        let password = payload["password"].as_str().unwrap_or("");

                        match authenticate_user(username, password, &pool).await {
                            Ok(user) => {
                                authenticated_user = Some(user.id.to_string());
				let token = generate_jwt(&user.id.to_string());
                                let response = json!({ "status": "ok", "token": token });
                                let _ = write.send(Message::Text(response.to_string())).await;
                            }
                            Err(msg) => {
                                let err = format!("{{\"error\": \"{}\"}}", msg);
                                let _ = write.send(Message::Text(err)).await;
                            }
                        }
                    }

                    Some("send_message") => {
                        let Some(user_id) = &authenticated_user else {
                            let _ = write.send(Message::Text(r#"{"error": "unauthorized"}"#.to_string())).await;
                            continue;
                        };

                        let payload = &json_msg["payload"];
                        let msg_data = serde_json::from_value::<NewMessage>(payload.clone());

                        match msg_data {
                            Ok(msg) => {
                                let user_uuid = sqlx::types::Uuid::parse_str(user_id).unwrap();
                                match send_message(msg, user_uuid, &pool).await {
                                    Ok(stored) => {
                                        let response = json!({
                                            "status": "message_saved",
                                            "message_id": stored.id,
                                            "timestamp": stored.created_at
                                        });
                                        let _ = write.send(Message::Text(response.to_string())).await;
                                    }
                                    Err(e) => {
                                        let err = format!(r#"{{"error": "db_error", "detail": "{}"}}"#, e);
                                        let _ = write.send(Message::Text(err)).await;
                                    }
                                }
                            }
                            Err(_) => {
                                let _ = write.send(Message::Text(r#"{"error": "invalid message format"}"#.to_string())).await;
                            }
                        }
                    }

                    Some("get_messages") => {
                        let Some(_) = &authenticated_user else {
                            let _ = write.send(Message::Text(r#"{"error": "unauthorized"}"#.to_string())).await;
                            continue;
                        };

                        let payload = &json_msg["payload"];
                        let chat_id = payload["chat_id"].as_str().unwrap_or("");
                        let limit = payload["limit"].as_i64().unwrap_or(50);

                        match Uuid::parse_str(chat_id) {
                            Ok(chat_uuid) => {
				                let mut redis_conn = redis.lock().await;
                                let cache_key = format!("chat:{}:messages", chat_id);

                                // Пытаемся получить сообщения из Redis
                                match redis_conn.get::<_, String>(&cache_key).await {
                                    Ok(cached_json) => {
                                        let _ = write.send(Message::Text(cached_json)).await;
                                        continue;
                                    }
                                    Err(_) => {
                                        // нет кэша — продолжаем к БД
                                    }
                                }

                                // Загружаем из PostgreSQL
                                match get_messages_for_chat(chat_uuid, limit, &pool).await {
                                    Ok(messages) => {
                                        let response_json = json!({
                                            "status": "messages",
                                            "messages": messages
                                        });

                                        let response_string = serde_json::to_string(&response_json)
                                            .expect("Failed to serialize response");

                                        // Сохраняем в Redis на 60 секунд
                                        let _: () = redis_conn.set_ex(&cache_key, &response_string, 180).await.unwrap();

                                        // Отправляем клиенту
                                        let _ = write.send(Message::Text(response_json.to_string())).await;
                                    }
                                    Err(e) => {
                                        let err = format!(r#"{{"error": "db_error", "detail": "{}"}}"#, e);
                                        let _ = write.send(Message::Text(err)).await;
                                    }
                                }
                            }
                            Err(_) => {
                                let _ = write.send(Message::Text(r#"{"error": "invalid chat_id"}"#.to_string())).await;
                            }
                        }
                    }

                    Some("edit_message") => {
                        let Some(user_id) = &authenticated_user else {
                            let _ = write.send(Message::Text(r#"{"error": "unauthorized"}"#.to_string())).await;
                            continue;
                        };

                        let payload = &json_msg["payload"];
                        let msg_id = payload["message_id"].as_str().unwrap_or("");
                        let new_body = payload["new_body"].as_str().unwrap_or("");

                        if let Ok(msg_uuid) = Uuid::parse_str(msg_id) {
                            let user_uuid = Uuid::parse_str(user_id).unwrap();
                            match edit_message(msg_uuid, user_uuid, new_body.to_string(), &pool).await {
                                Ok(_) => {
                                    let _ = write.send(Message::Text(r#"{"status": "message_edited"}"#.to_string())).await;
                                }
                                Err(e) => {
                                    let err = format!(r#"{{"error": "edit_failed", "detail": "{}"}}"#, e);
                                    let _ = write.send(Message::Text(err)).await;
                                }
                            }
                        }
                    }

                    Some("delete_message") => {
                        let Some(user_id) = &authenticated_user else {
                            let _ = write.send(Message::Text(r#"{"error": "unauthorized"}"#.to_string())).await;
                            continue;
                        };

                        let payload = &json_msg["payload"];
                        let msg_id = payload["message_id"].as_str().unwrap_or("");
                        let for_all = payload["for_all"].as_bool().unwrap_or(false);

                        if let Ok(msg_uuid) = Uuid::parse_str(msg_id) {
                            let user_uuid = Uuid::parse_str(user_id).unwrap();
                            match delete_message(msg_uuid, user_uuid, for_all, &pool).await {
                                Ok(_) => {
                                    let _ = write.send(Message::Text(r#"{"status": "message_deleted"}"#.to_string())).await;
                                }
                                Err(e) => {
                                    let err = format!(r#"{{"error": "delete_failed", "detail": "{}"}}"#, e);
                                    let _ = write.send(Message::Text(err)).await;
                                }
                            }
                        }
                    }

                    Some("react") => {
                        let Some(user_id) = &authenticated_user else {
                            let _ = write.send(Message::Text(r#"{"error":"unauthorized"}"#.to_string())).await;
                            continue;
                        };

                        let payload = &json_msg["payload"];
                        let message_id = payload["message_id"].as_str().unwrap_or("");
                        let emoji = payload["emoji"].as_str().unwrap_or("👍");

                        if let Ok(msg_uuid) = Uuid::parse_str(message_id) {
                            let user_uuid = Uuid::parse_str(user_id).unwrap();
                            match set_reaction(msg_uuid, user_uuid, emoji.to_string(), &pool).await {
                                Ok(_) => {
                                    let _ = write.send(Message::Text(r#"{"status":"reaction_set"}"#.to_string())).await;
                                }
                                Err(e) => {
                                    let err = format!(r#"{{"error":"reaction_failed","detail":"{}"}}"#, e);
                                    let _ = write.send(Message::Text(err)).await;
                                }
                            }
                        }
                    }

                    Some("forward_message") => {
                        let Some(user_id) = &authenticated_user else {
                            let _ = write.send(Message::Text(r#"{"error":"unauthorized"}"#.to_string())).await;
                            continue;
                        };

                        let payload = &json_msg["payload"];
                        let chat_id = payload["chat_id"].as_str().unwrap_or("");
                        let original_id = payload["original_message_id"].as_str().unwrap_or("");

                        if let (Ok(chat_uuid), Ok(orig_uuid)) = (
                            Uuid::parse_str(chat_id),
                            Uuid::parse_str(original_id),
                        ) {
                            let sender_uuid = Uuid::parse_str(user_id).unwrap();

                            match forward_message(chat_uuid, orig_uuid, sender_uuid, &pool).await {
                                Ok(msg) => {
                                    let response = json!({
                                        "status": "message_forwarded",
                                        "message_id": msg.id,
                                        "timestamp": msg.created_at,
                                        "forwarded_from": msg.forwarded_from
                                    });
                                    let _ = write.send(Message::Text(response.to_string())).await;
                                }
                                Err(e) => {
                                    let err = format!(r#"{{"error":"forward_failed","detail":"{}"}}"#, e);
                                    let _ = write.send(Message::Text(err)).await;
                                }
                            }
                        }
                    }

                    Some("create_chat") => {
                        let Some(user_id) = &authenticated_user else {
                            let _ = write.send(Message::Text(r#"{"error": "unauthorized"}"#.to_string())).await;
                            continue;
                        };

                        let payload = &json_msg["payload"];
                        let name = payload["name"].as_str().unwrap_or("Группа без названия");

                        let members: Vec<Uuid> = payload["members"]
                            .as_array()
                            .unwrap_or(&vec![])
                            .iter()
                            .filter_map(|v| v.as_str())
                            .filter_map(|s| Uuid::parse_str(s).ok())
                            .collect();

                        let creator_uuid = Uuid::parse_str(user_id).unwrap();

                        match create_group_chat(name.to_string(), creator_uuid, members, &pool).await {
                            Ok(chat_id) => {
                                let response = json!({
                                    "status": "chat_created",
                                    "chat_id": chat_id
                                });
                                let _ = write.send(Message::Text(response.to_string())).await;
                            }
                            Err(e) => {
                                let err = format!(r#"{{"error":"create_chat_failed","detail":"{}"}}"#, e);
                                let _ = write.send(Message::Text(err)).await;
                            }
                        }
                    }

                    Some("add_to_chat") => {
                        let Some(sender_id) = &authenticated_user else {
                            let _ = write.send(Message::Text(r#"{"error": "unauthorized"}"#.to_string())).await;
                            continue;
                        };

                        let payload = &json_msg["payload"];
                        let chat_id = payload["chat_id"].as_str().unwrap_or("");
                        let new_user_id = payload["user_id"].as_str().unwrap_or("");

                        if let (Ok(chat_uuid), Ok(user_uuid), Ok(sender_uuid)) =
                            (Uuid::parse_str(chat_id), Uuid::parse_str(new_user_id), Uuid::parse_str(sender_id))
                        {
                            // Проверяем, что добавляющий состоит в чате
                            match is_user_in_chat(chat_uuid, sender_uuid, &pool).await {
                                Ok(true) => {

                                    // Проверяем, существует ли указанный пользователь
                                    match user_exists(user_uuid, &pool).await {
                                        Ok(false) => {
                                            let _ = write.send(Message::Text(r#"{"error": "user_not_found"}"#.to_string())).await;
                                            continue;
                                        }
                                        Err(e) => {
                                            let err = format!(r#"{{"error":"user_check_failed","detail":"{}"}}"#, e);
                                            let _ = write.send(Message::Text(err)).await;
                                            continue;
                                        }
                                        _ => {}
                                    }

                                    match add_user_to_chat(chat_uuid, user_uuid, &pool).await {
                                        Ok(_) => {
                                            let _ = write.send(Message::Text(r#"{"status": "user_added"}"#.to_string())).await;
                                        }
                                        Err(e) => {
                                            let err = format!(r#"{{"error":"add_failed","detail":"{}"}}"#, e);
                                            let _ = write.send(Message::Text(err)).await;
                                        }
                                    }
                                }
                                Ok(false) => {
                                    let _ = write.send(Message::Text(r#"{"error":"not_in_chat"}"#.to_string())).await;
                                }
                                Err(e) => {
                                    let err = format!(r#"{{"error":"check_failed","detail":"{}"}}"#, e);
                                    let _ = write.send(Message::Text(err)).await;
                                }
                            }
                        }
                    }

                    Some("remove_from_chat") => {
                        let Some(sender_id) = &authenticated_user else {
                            let _ = write.send(Message::Text(r#"{"error": "unauthorized"}"#.to_string())).await;
                            continue;
                        };

                        let payload = &json_msg["payload"];
                        let chat_id = payload["chat_id"].as_str().unwrap_or("");
                        let remove_id = payload["user_id"].as_str().unwrap_or("");

                        if let (Ok(chat_uuid), Ok(user_uuid)) =
                            (Uuid::parse_str(chat_id), Uuid::parse_str(remove_id))
                        {

                            // Проверяем, существует ли пользователь
                            match user_exists(user_uuid, &pool).await {
                                Ok(false) => {
                                    let _ = write.send(Message::Text(r#"{"error": "user_not_found"}"#.to_string())).await;
                                    continue;
                                }
                                Err(e) => {
                                    let err = format!(r#"{{"error":"user_check_failed","detail":"{}"}}"#, e);
                                    let _ = write.send(Message::Text(err)).await;
                                    continue;
                                }
                                _ => {}
                            }

                            // Проверяем, есть ли он в чате
                            match is_user_in_chat(chat_uuid, user_uuid, &pool).await {
                                Ok(false) => {
                                    let _ = write.send(Message::Text(r#"{"error": "user_not_in_chat"}"#.to_string())).await;
                                    continue;
                                }
                                Err(e) => {
                                    let err = format!(r#"{{"error":"chat_check_failed","detail":"{}"}}"#, e);
                                    let _ = write.send(Message::Text(err)).await;
                                    continue;
                                }
                                _ => {}
                            }

                            match remove_user_from_chat(chat_uuid, user_uuid, &pool).await {
                                Ok(_) => {
                                    let _ = write.send(Message::Text(r#"{"status": "user_removed"}"#.to_string())).await;
                                }
                                Err(e) => {
                                    let err = format!(r#"{{"error":"remove_failed","detail":"{}"}}"#, e);
                                    let _ = write.send(Message::Text(err)).await;
                                }
                            }
                        }
                    }

                    Some("get_chat_members") => {
                        let payload = &json_msg["payload"];
                        let chat_id = payload["chat_id"].as_str().unwrap_or("");

                        if let Ok(chat_uuid) = Uuid::parse_str(chat_id) {
                            match get_chat_members(chat_uuid, &pool).await {
                                Ok(members) => {
                                    let response = json!({
                                        "status": "members_list",
                                        "members": members
                                    });
                                    let _ = write.send(Message::Text(response.to_string())).await;
                                }
                                Err(e) => {
                                    let err = format!(r#"{{"error":"get_members_failed","detail":"{}"}}"#, e);
                                    let _ = write.send(Message::Text(err)).await;
                                }
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