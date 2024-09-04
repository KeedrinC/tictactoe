use std::sync::Arc;
use serde_json::{json, Value};
use std::sync::Mutex;
use std::net::SocketAddr;
use serde::Deserialize;
use crate::state::AppState;

#[derive(Debug, Deserialize, Eq, Hash, PartialEq)]
pub enum Message {
    Connection { nickname: Option<String>, token: Option<String> },   // pass a token to resume session after a disconnect
    CreateLobby,                            // creates a new lobby for the current session
    JoinLobby { id: String },               // moves the current session to an existing lobby
    Nickname(String),                       // changes the nickname of the current session
    Move(usize)                             // move the session to a spot in their game
}

pub async fn process_messsage(message: Message, socket: SocketAddr, state: Arc<Mutex<AppState>>) -> Result<Value, ()> {
    let mut state = state.lock().unwrap();
    match message {
        Message::Connection { nickname, token } => {
            let session = if let Some(token) = &token {
                state.move_session(socket, token)
            } else { state.new_session(socket, nickname.clone()) };
            if let Some(session) = session {
                Ok(json!(**session))
            } else { Err(()) }
        },
        Message::CreateLobby => {
            let initiator = state.socket_session.get_mut(&socket);
            if let Some(session) = initiator {
                let session = &mut session.clone();
                let lobby = state.new_lobby(session);
                if let Some(lobby) = lobby {
                    Ok(json!(lobby))
                } else { Err(()) }
            } else { Err(()) }
        },
        Message::JoinLobby { id } => {
            let initiator = state.socket_session.get_mut(&socket);
            if let Some(session) = initiator {
                let session = &mut session.clone();
                let lobby = state.join_lobby(id, session);
                if let Some(lobby) = lobby {
                    Ok(json!(lobby))
                } else { Err(()) }
            } else { Err(()) }
        },
        Message::Nickname(_) => todo!(),
        Message::Move(_) => todo!(),
    }
}