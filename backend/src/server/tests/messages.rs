use std::{net::SocketAddr, sync::{Arc, Mutex}};
use axum::extract::ws::Message;
use futures::{channel::mpsc::{Receiver, Sender}, SinkExt, StreamExt};
use serde::Deserialize;
use serde_json::json;
use crate::{handle_socket, tests::utils::new_socket, AppState};

fn mock_state() -> Arc<Mutex<AppState>> {
    Arc::new(Mutex::new(AppState::new()))
}

async fn setup(state: Arc<Mutex<AppState>>, connection: Option<SocketAddr>) -> (Sender<Result<Message, axum::Error>>, Receiver<Message>) {
    let connection = connection.unwrap_or(new_socket(1111));
    let (socket_write, rx) = futures::channel::mpsc::channel(1024);
    let (tx, socket_read) = futures::channel::mpsc::channel(1024);
    tokio::spawn(handle_socket(socket_write, socket_read, connection, state));
    (tx, rx)
}

#[tokio::test]
async fn test_multiple_new_connections() {
    #[derive(Debug, Deserialize)]
    struct Session { nickname: Option<String>, token: String }

    let state = mock_state(); // shared state between connections
    let (mut tx, mut rx) = setup(state.clone(), Some(new_socket(1111))).await;
    let (mut tx2, mut rx2) = setup(state.clone(), Some(new_socket(2222))).await;

    tx.send(Ok(Message::Text(json!({"Connection": {"nickname": "keedrin"}}).to_string()))).await.unwrap();
    tx2.send(Ok(Message::Text(json!({"Connection": {"nickname": "keedrin2"}}).to_string()))).await.unwrap();

    let msg = match rx.next().await.unwrap() {
        Message::Text(msg) => msg,
        other => panic!("expected a text message but got {other:?}"),
    };
    let msg2 = match rx2.next().await.unwrap() {
        Message::Text(msg) => msg,
        other => panic!("expected a text message but got {other:?}"),
    };

    let state = state.lock().unwrap();
    let response = serde_json::from_str::<Session>(&msg).unwrap();
    let record = state.sessions.get(&response.token);
    assert!(record.is_some());
    let record = record.unwrap().lock().unwrap().clone();
    assert_eq!(record.access_token, response.token);
    assert_eq!(record.nickname, response.nickname);
    assert!(record.nickname.is_some());
    assert_eq!(record.nickname.unwrap(), String::from("keedrin"));

    let response = serde_json::from_str::<Session>(&msg2).unwrap();
    let record = state.sessions.get(&response.token);
    assert!(record.is_some());
    let record = record.unwrap().lock().unwrap().clone();
    assert_eq!(record.access_token, response.token);
    assert_eq!(record.nickname, response.nickname);
    assert!(record.nickname.is_some());
    assert_eq!(record.nickname.unwrap(), String::from("keedrin2"));

    assert!(state.sessions.len() == 2);
}