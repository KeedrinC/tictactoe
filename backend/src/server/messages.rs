use std::sync::Arc;
use serde_json::{json, Value};
use std::sync::Mutex;
use std::net::SocketAddr;
use serde::Deserialize;
use crate::state::AppState;

#[derive(Debug, Deserialize, Eq, Hash, PartialEq)]
pub enum ClientMessage { // these are messages that should be received by the connected client
    // if a token is passed we attempt to retrieve a session that currently exists
    Connection { nickname: Option<String>, access_token: Option<String> },
    Nickname { nickname: String },                 // changes the nickname of the current session
    CreateLobby,                            // creates a new lobby for the current session
    JoinLobby { id: String },               // moves the current session to an existing lobby
    StartGame,
    Move(usize)                             // move the session to a spot in their game
}

impl ClientMessage {
    pub async fn process(message: ClientMessage, socket: SocketAddr, state: Arc<Mutex<AppState>>) -> Result<Value, ()> {
        // when we get a message from a client we pass information about the client to its corresponding function
        let state: Arc<Mutex<AppState>> = state.clone();
        match message {
            ClientMessage::Connection { nickname, access_token: token }
                => ClientMessage::new_connection(state, socket, nickname, token),
            ClientMessage::Nickname { nickname } => ClientMessage::change_nickname(state, socket, nickname),
            ClientMessage::CreateLobby => ClientMessage::create_lobby(state, socket),
            ClientMessage::JoinLobby { id } => ClientMessage::join_lobby(state, socket, id),
            ClientMessage::StartGame => ClientMessage::start_game(state, socket),
            ClientMessage::Move(position) => ClientMessage::move_message(state, socket, position),
        }
    }

    fn new_connection(
        state: Arc<Mutex<AppState>>,
        socket: SocketAddr,
        nickname: Option<String>,
        access_token: Option<String>
    ) -> Result<serde_json::Value, ()> {
        let mut state = state.lock().unwrap();
        let session = match &access_token {
            Some(access_token) => state.move_session(socket, access_token),
            None => Some(state.new_session(socket, nickname.clone())),
        };
        if let Some(session) = session {
            Ok(json!({
                "type": "Connection",
                "data": *session
            }))
        } else { Err(()) }
    }

    fn change_nickname(state: Arc<Mutex<AppState>>, socket: SocketAddr, nickname: String) -> Result<serde_json::Value, ()>  {
        let state = state.lock().unwrap();
        let session = state.socket_session.get(&socket).ok_or(()).cloned()?;
        let mut session_guard = session.lock().unwrap();
        session_guard.set_nickname(&nickname);
        Ok(json!({
            "type": "Nickname",
            "data": *session_guard
        }))
    }

    fn create_lobby(state: Arc<Mutex<AppState>>, socket: SocketAddr) -> Result<serde_json::Value, ()> {
        let mut state = state.lock().unwrap();
        let session = state.socket_session.get(&socket).ok_or(()).cloned()?;
        let new_lobby = state.new_lobby(session.clone());
        let lobby_guard = new_lobby.lock().unwrap();
        Ok(json!({
            "type": "CreateLobby",
            "data": *lobby_guard
        }))
    }

    fn join_lobby(state: Arc<Mutex<AppState>>, socket: SocketAddr, id: String) -> Result<serde_json::Value, ()> {
        let mut state = state.lock().unwrap();
        let session = state.socket_session.get(&socket).ok_or(()).cloned()?;
        let lobby = state.join_lobby(&id, session.clone())?;
        let lobby_guard = lobby.lock().unwrap();
        Ok(json!({
            "type": "JoinLobby",
            "data": *lobby_guard
        }))
    }
    
    fn start_game(state: Arc<Mutex<AppState>>, socket: SocketAddr) -> Result<serde_json::Value, ()>  {
        let state = state.lock().unwrap();
        let session = state.socket_session.get(&socket).ok_or(()).cloned()?;
        let session_token = &session.lock().unwrap().access_token;
        let lobby = state.session_lobby.get(session_token).ok_or(()).cloned()?;
        let mut lobby_guard = lobby.lock().unwrap();
        lobby_guard.start_game(); // start the game attached to the lobby
        Ok(json!({
            "type": "StartGame",
            "data": *lobby_guard
        }))
    }

    fn move_message(state: Arc<Mutex<AppState>>, socket: SocketAddr, position: usize) -> Result<serde_json::Value, ()> {
        let state = state.lock().unwrap();
        let session = state.socket_session.get(&socket).ok_or(()).cloned()?;
        let session_token = &session.lock().unwrap().access_token;
        let lobby = state.session_lobby.get(session_token).ok_or(()).cloned()?;
        let mut lobby_guard = lobby.lock().unwrap();
        let players = lobby_guard.players.clone();
        if let Some(game) = lobby_guard.game.as_mut() {
            let player = players.iter().find(|&entry| {
                entry.clone().is_some_and(|(s, p)|
                    Arc::ptr_eq(&session, &s)
                        && game.current_player.eq(&Some(p)))
            });
            if let Some(Some((_, player))) = player {
                let _ = game.move_player(player, position);
            }
        }
        Ok(json!({
            "type": "StartGame",
            "data": *lobby_guard
        }))
    }
}