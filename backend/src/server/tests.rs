use std::{net::{IpAddr, Ipv4Addr, SocketAddr}, sync::Arc};
use axum::extract::ws::Message;
use futures::{channel::mpsc::{Receiver, Sender}, SinkExt, StreamExt};
use serde_json::json;
use tokio::sync::Mutex;
use crate::{handle_socket, AppState};

fn mock_state() -> AppState {
    let state: AppState = AppState::new();
    let address: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 1111);
    state
}

async fn setup() -> (Sender<Result<Message, axum::Error>>, Receiver<Message>) {
    // Need to use "futures" channels rather than "tokio" channels as they implement `Sink` and `Stream`
    let (socket_write, rx) = futures::channel::mpsc::channel(1024);
    let (tx, socket_read) = futures::channel::mpsc::channel(1024);
    let connection = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 1111);
    tokio::spawn(handle_socket(socket_write, socket_read, connection, Arc::new(Mutex::new(mock_state()))));
    (tx, rx)
}

#[tokio::test]
async fn test_new_connection() {
    let (mut tx, mut rx) = setup().await;
    tx.send(Ok(Message::Text(json!({"Connection": {}}).to_string())))
        .await
        .unwrap();
    let msg = match rx.next().await.unwrap() {
        Message::Text(msg) => dbg!(msg),
        other => panic!("expected a text message but got {other:?}"),
    };
    // assert_eq!(msg, realtime::Message::Connection { token: Some(String::from("random-uuid")) });
}