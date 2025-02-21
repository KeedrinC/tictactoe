use std::sync::Arc;
use serde_json::{json, Value};
use std::sync::Mutex;
use std::net::SocketAddr;
use serde::Deserialize;
use crate::state::AppState;

#[derive(Debug, Deserialize, Eq, Hash, PartialEq)]
#[serde(tag = "type", content = "data")]
pub enum ClientMessage { // these are messages that should be received by the connected client
    // connection: if a token is passed we attempt to retrieve a session that currently exists
    Connection { nickname: Option<String>, access_token: Option<String> },
    Nickname { nickname: String },          // changes the nickname of the current session
    CreateLobby,                            // creates a new lobby for the current session
    JoinLobby { code: String },             // moves the current session to an existing lobby
    StartGame,
    Move { position: usize }                // move the session to a spot in their game
}

impl ClientMessage {
    pub async fn process(message: ClientMessage, socket: SocketAddr, state: Arc<Mutex<AppState>>) -> Result<Value, String> {
        // when we get a message from a client we pass information about the client to its corresponding function
        let state: Arc<Mutex<AppState>> = state.clone();
        match message {
            ClientMessage::Connection { nickname, access_token }
                => ClientMessage::new_connection(state, socket, nickname, access_token),
            ClientMessage::Nickname { nickname } => ClientMessage::change_nickname(state, socket, nickname),
            ClientMessage::CreateLobby => ClientMessage::create_lobby(state, socket),
            ClientMessage::JoinLobby { code } => ClientMessage::join_lobby(state, socket, code),
            ClientMessage::StartGame => ClientMessage::start_game(state, socket),
            ClientMessage::Move { position } => ClientMessage::move_message(state, socket, position),
        }
    }

    fn new_connection(
        state: Arc<Mutex<AppState>>,
        socket: SocketAddr,
        nickname: Option<String>,
        access_token: Option<String>
    ) -> Result<serde_json::Value, String> {
        let mut state = state.lock().unwrap();
        let session = match &access_token {
            Some(access_token) => state.move_session(socket, access_token),
            None => Some(state.new_session(socket, nickname.clone())),
        };
        if let Some(session) = session {
            let response = json!({
                "type": "Session",
                "data": *session
            });
            tracing::info!("new_connection {}", response);
            tracing::info!("new_connection number of sessions {}", state.sessions.len());
            Ok(response)
        } else { Err("could not connect".to_string()) }
    }

    fn change_nickname(state: Arc<Mutex<AppState>>, socket: SocketAddr, nickname: String) -> Result<serde_json::Value, String>  {
        let state = state.lock().unwrap();
        let session = state.socket_session.get(&socket).ok_or("couldn't find session based on socket").cloned()?;
        let mut session_guard = session.lock().unwrap();
        session_guard.set_nickname(&nickname);
        let response = json!({
            "type": "Session",
            "data": *session_guard
        });
        tracing::info!("change_nickname {}", response);
        Ok(response)
    }

    fn create_lobby(state: Arc<Mutex<AppState>>, socket: SocketAddr) -> Result<serde_json::Value, String> {
        let mut state = state.lock().unwrap();
        let session = state.socket_session.get(&socket).ok_or("couldn't find session based on socket").cloned()?;
        let new_lobby = state.new_lobby(session.clone());
        let lobby_guard = new_lobby.lock().unwrap();
        let response = json!({
            "type": "Lobby",
            "data": *lobby_guard
        });
        tracing::info!("create_lobby {}", response);
        tracing::info!("create_lobby number of lobbies {}", state.lobbies.len());
        Ok(response)
    }

    fn join_lobby(state: Arc<Mutex<AppState>>, socket: SocketAddr, code: String) -> Result<serde_json::Value, String> {
        let mut state = state.lock().unwrap();
        let session = state.socket_session.get(&socket).ok_or("couldn't find session").cloned()?;
        let lobby = state.join_lobby(&code, session.clone()).map_err(|_| "couldn't join lobby")?;
        let lobby_guard = lobby.lock().unwrap();
        let response = json!({
            "type": "Lobby",
            "data": *lobby_guard
        });
        tracing::info!("join_lobby {}", response);
        Ok(response)
    }
    
    fn start_game(state: Arc<Mutex<AppState>>, socket: SocketAddr) -> Result<serde_json::Value, String>  {
        let state = state.lock().unwrap();
        let session = state.socket_session.get(&socket).ok_or("couldn't find session based on socket").cloned()?;
        let session_token = &session.lock().unwrap().access_token;
        let lobby = state.session_lobby.get(session_token).ok_or("couldn't find lobby based on session").cloned()?;
        let mut lobby_guard = lobby.lock().unwrap();
        lobby_guard.start_game(); // start the game attached to the lobby
        let response = json!({
            "type": "StartGame",
            "data": *lobby_guard
        });
        tracing::info!("join_lobby {}", response);
        Ok(response)
    }

    fn move_message(state: Arc<Mutex<AppState>>, socket: SocketAddr, position: usize) -> Result<serde_json::Value, String> {
        let state = state.lock().unwrap();
        let session = state.socket_session.get(&socket).ok_or("couldn't find session based on socket").cloned()?;
        let session_token = &session.lock().unwrap().access_token;
        let lobby = state.session_lobby.get(session_token).ok_or("couldn't find lobby based on session").cloned()?;
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
            let response = json!({
                "type": "Move",
                "data": game.board
            });
            tracing::info!("move_message {}", response);
            return Ok(response);
        }
        Err("unhandled error".to_string())
    }
}