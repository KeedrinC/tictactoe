use std::{net::SocketAddr, sync::{Arc, Mutex}};
use axum::extract::ws::Message;
use futures::{channel::mpsc::{Receiver, Sender}, SinkExt, StreamExt};
use serde::Deserialize;
use serde_json::json;
use crate::{handle_socket, session::Session, tests::utils::new_socket, AppState};

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

#[derive(Deserialize)]
struct Response<T> { data: T }
#[derive(Deserialize)]
struct ResponseSession { access_token: String, nickname: Option<String> }
#[derive(Deserialize)]
struct ResponseLobby { code: String }

#[tokio::test]
async fn test_multiple_new_connections() {
    let state = mock_state(); // shared state between connections
    let (mut tx, mut rx) = setup(state.clone(), Some(new_socket(1111))).await;
    let (mut tx2, mut rx2) = setup(state.clone(), Some(new_socket(2222))).await;

    tx.send(Ok(Message::Text(json!({"type": "Connection", "data": {"nickname": "keedrin"}}).to_string().into()))).await.unwrap();
    tx2.send(Ok(Message::Text(json!({"type": "Connection", "data": {"nickname": "keedrin2"}}).to_string().into()))).await.unwrap();
    let (msg, msg2) = (rx.next().await.unwrap(), rx2.next().await.unwrap());

    {
        let state = state.lock().unwrap();
        let response = serde_json::from_str::<Response<ResponseSession>>(&msg.to_text().unwrap()).unwrap().data;
        let record = state.sessions.get(&response.access_token).unwrap();
        test_new_connection_properties(&response, &record, "keedrin");
        let response = serde_json::from_str::<Response<ResponseSession>>(&msg2.to_text().unwrap()).unwrap().data;
        let record = state.sessions.get(&response.access_token).unwrap();
        test_new_connection_properties(&response, &record, "keedrin2");
        assert!(state.sessions.len() == 2);
    }

    tx.send(Ok(Message::Text(json!({"type": "CreateLobby"}).to_string().into()))).await.unwrap();
    let msg = rx.next().await.unwrap();
    let response = serde_json::from_str::<Response<ResponseLobby>>(&msg.to_text().unwrap()).unwrap().data;
    tx2.send(Ok(Message::Text(json!({"type": "JoinLobby", "data": {"code": &response.code}}).to_string().into()))).await.unwrap();
    rx2.next().await.unwrap();
    {
        let state = state.lock().unwrap();
        let record = state.lobbies.get(&response.code).unwrap();
        let record = record.lock().unwrap();
        assert_eq!(record.player_count(), 2);
    }

}

fn test_new_connection_properties(
    response: &ResponseSession,
    record: &Arc<Mutex<Session>>,
    nickname: &str
) {
    let record = record.lock().unwrap().clone();
    assert_eq!(record.access_token, response.access_token);
    assert_eq!(record.nickname, response.nickname);
    assert!(record.nickname.is_some());
    assert_eq!(record.nickname.unwrap(), String::from(nickname));
}