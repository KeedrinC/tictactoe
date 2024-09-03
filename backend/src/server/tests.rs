use std::{net::{IpAddr, Ipv4Addr, SocketAddr}, sync::{Arc, Mutex}};
use axum::extract::ws::Message;
use futures::{channel::mpsc::{Receiver, Sender}, SinkExt, StreamExt};
use serde_json::json;
use crate::{handle_socket, AppState};

fn mock_state() -> AppState {
    let state: AppState = AppState::new();
    state
}

async fn setup() -> (Sender<Result<Message, axum::Error>>, Receiver<Message>, Arc<Mutex<AppState>>) {
    let connection = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 1111);
    // Need to use "futures" channels rather than "tokio" channels as they implement `Sink` and `Stream`
    let (socket_write, rx) = futures::channel::mpsc::channel(1024);
    let (tx, socket_read) = futures::channel::mpsc::channel(1024);
    let state = Arc::new(Mutex::new(mock_state()));
    tokio::spawn(handle_socket(socket_write, socket_read, connection, state.clone()));
    (tx, rx, state)
}

#[tokio::test]
async fn test_new_connection() {
    let (mut tx, mut rx, state) = setup().await;
    tx.send(Ok(Message::Text(json!({"Connection": {"nickname": "keedrin"}}).to_string()))).await.unwrap();
    let msg = match rx.next().await.unwrap() {
        Message::Text(msg) => dbg!(msg),
        other => panic!("expected a text message but got {other:?}"),
    };
    // assert_eq!(msg, realtime::Message::Connection { token: Some(String::from("random-uuid")) });
}