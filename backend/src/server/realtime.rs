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
    JoinLobby(String),                      // moves the current session to an existing lobby
    Nickname(String),                       // changes the nickname of the current session
    Move(usize)                             // move the session to a spot in their game
}

pub async fn process_messsage(message: Message, socket: SocketAddr, state: Arc<Mutex<AppState>>) -> Result<Value, ()> {
    let mut state = state.lock().unwrap();
    match message {
        Message::Connection { nickname, token } => {
            let session = if let Some(token) = &token {
                state.move_session(socket, token)
            } else { state.new_session(socket) };
            if let Some(session) = session {
                let mut session = session.lock().unwrap();
                if let Some(nickname) = nickname { session.set_nickname(&nickname); }
                Ok(json!(*session))
            } else { Err(()) }
        },
        Message::CreateLobby => todo!(),
        Message::JoinLobby(_) => todo!(),
        Message::Nickname(_) => todo!(),
        Message::Move(_) => todo!(),
    }
}