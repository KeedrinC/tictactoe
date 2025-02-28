use std::sync::{Arc, MutexGuard};
use serde_json::{json, Value};
use std::sync::Mutex;
use std::net::SocketAddr;
use game::Player;
use serde::Deserialize;
use crate::lobby::Lobby;
use crate::session::Session;
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
    Move { position: usize },                // move the session to a spot in their game
    OnHover { position: usize }                // made when a player's mouse is hovered over a square
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
            ClientMessage::OnHover { position } => ClientMessage::on_hover(state, socket, position),
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
        if let Some((sender, _)) = state.lobby_channel.get(&lobby_guard.code) {
            let _ = sender.send(response.clone());
        }
        send_message(&mut state, &lobby_guard, &response);
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
        match lobby_guard.game.as_mut() {
            Some(game) => {
                let x = players.iter().find(|player| {
                    player.as_ref().is_some_and(|(s, p)|
                        Arc::ptr_eq(&session, &s) && game.current_player.eq(&Some(*p))
                )});
                if let Some(Some((_, player))) = x {
                    let _ = game.move_player(player, position);
                }
                let response = json!({
                    "type": "Move",
                    "data": game.board
                });
                tracing::info!("move_message {}", response);
                Ok(response)
            },
            None => Err("game hasn't started yet".to_string()),
        }
    }

    fn on_hover(state: Arc<Mutex<AppState>>, socket: SocketAddr, position: usize) -> Result<serde_json::Value, String> {
        let mut state = state.lock().unwrap();
        let session = get_socket_session(&mut state, socket)?;
        let session_guard = session.lock().unwrap();
        let lobby = get_socket_lobby(&mut state, &session_guard)?;
        let lobby_guard = lobby.lock().unwrap();
        let (_, player) = lobby_guard.players.iter().find(|player| {
            player.as_ref().is_some_and(|(s, _)| Arc::ptr_eq(&session, &s))
        }).unwrap().clone().unwrap();
        match &lobby_guard.game {
            Some(game) => {
                if game.current_player.eq(&Some(player)) {
                    let message = json!({
                        "type": "OnHover",
                        "data": {"symbol": match player {
                            Player::X => "X",
                            Player::O => "O",
                        }, "position": position}
                    });
                    send_message(&mut state, &lobby_guard, &message);
                    Ok(json!({}))
                } else { Err("not this player's turn".to_owned()) }
            },
            None => Err("lobby doesn't have a game".to_owned()),
        }
    }
}

fn get_socket_session(state: &mut MutexGuard<AppState>, socket: SocketAddr) -> Result<Arc<Mutex<Session>>, String> {
    let session = state.socket_session.get(&socket).ok_or("couldn't find session based on socket").cloned()?;
    Ok(session)
}

fn get_socket_lobby(state: &mut MutexGuard<AppState>, session: &MutexGuard<Session>) -> Result<Arc<Mutex<Lobby>>, String> {
    let session_token = &session.access_token;
    let lobby = state.session_lobby.get(session_token).ok_or("couldn't find lobby based on session").cloned()?;
    Ok(lobby)
}

fn send_message(state: &mut MutexGuard<AppState>, lobby: &MutexGuard<Lobby>, message: &Value) {
    if let Some((sender, _)) = state.lobby_channel.get(&lobby.code) {
        let _ = sender.send(message.clone());
    }
}