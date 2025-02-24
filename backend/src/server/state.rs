use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use serde_json::{json, Value};
use tokio::sync::broadcast::{Receiver, Sender};
use crate::lobby::Lobby;
use crate::session::Session;

pub struct AppState {
    pub lobbies: HashMap<String, Arc<Mutex<Lobby>>>,    // lobbies with currently active users
    pub sessions: HashMap<String, Arc<Mutex<Session>>>, // every connection creates a session object
    pub session_lobby: HashMap<String, Arc<Mutex<Lobby>>>, // map sessions to current lobbies for easy lookup
    pub socket_session: HashMap<SocketAddr, Arc<Mutex<Session>>>,   // map sockets to session for easy lookup
    pub lobby_channel: HashMap<String, (Sender<Value>, Receiver<Value>)>,   // map sockets to session for easy lookup
}

impl AppState {
    pub fn new() -> Self {
        // initialize each of the server's objects
        let lobbies: HashMap<String, Arc<Mutex<Lobby>>> = HashMap::new();
        let sessions: HashMap<String, Arc<Mutex<Session>>> = HashMap::new();
        let session_lobby: HashMap<String, Arc<Mutex<Lobby>>> = HashMap::new();
        let socket_session: HashMap<SocketAddr, Arc<Mutex<Session>>> = HashMap::new();
        let lobby_channel: HashMap<String, (Sender<Value>, Receiver<Value>)> = HashMap::new();
        AppState { lobbies, sessions, session_lobby, socket_session, lobby_channel }
    }

    pub fn new_lobby(&mut self, player_session: Arc<Mutex<Session>>) -> Arc<Mutex<Lobby>> {
        let lobby: Lobby = Lobby::new(player_session.clone()); // create new lobby
        let new_lobby: Arc<Mutex<Lobby>> = Arc::new(Mutex::new(lobby.clone()));
        let session_token: String = player_session.lock().unwrap().access_token.clone();
        if self.session_lobby.contains_key(&session_token) {
            self.leave_lobby(&player_session) // leave the previous lobby 
        }
        let (sender, receiver) = tokio::sync::broadcast::channel::<Value>(lobby.code.parse::<usize>().unwrap());
        let _ = sender.send(json!({"data": ""}));
        // add to both self.lobbies and self.session_lobby for lobby lookup using session
        self.lobby_channel.insert(lobby.code.clone(), (sender, receiver));
        self.lobbies.insert(lobby.code.clone(), new_lobby.clone());
        self.session_lobby.insert(session_token, new_lobby.clone());
        if let Some((sender, _)) = self.lobby_channel.get(&lobby.code.clone()) {
            let _ = sender.send(json!({"data": ""}));
        }
        new_lobby
    }

    pub fn new_session(&mut self, socket: SocketAddr, nickname: Option<String>) -> Arc<Mutex<Session>> {
        let session: Arc<Mutex<Session>> = Arc::new(Mutex::new(Session::new(socket, nickname)));
        let token: String = session.lock().unwrap().access_token.clone();
        // add to both self.sessions and self.socket_session for session lookup using the socket
        self.sessions.insert(token.clone(), session.clone());
        self.socket_session.insert(socket, session.clone());
        session
    }

    pub fn move_session(&mut self, socket: SocketAddr, token: &str) -> Option<Arc<Mutex<Session>>> {
        let session = self.sessions.get_mut(token); // get session using token
        if let Some(session) = session {
            let mut s = session.lock().unwrap();
            self.socket_session.remove(&s.socket); // remove the previous socket address, client is using a different address
            s.socket = socket; // use new socket now
            self.socket_session.insert(socket, session.clone()); // and add it to the hashmap to find the session using the new address
            Some(session.clone())
        } else { None }
    }

    pub fn join_lobby(&mut self, lobby_code: &str, player_session: Arc<Mutex<Session>>) -> Result<Arc<Mutex<Lobby>>, ()> {
        if !self.lobbies.contains_key(lobby_code) { return Err(()); }
        let session_token: String = player_session.lock().unwrap().access_token.clone();
        // Check if the session is already in a session, then leave it
        if self.session_lobby.contains_key(&session_token) {
            self.leave_lobby(&player_session);
        };
        let lobby = self.lobbies.get(lobby_code).ok_or(()).cloned()?;
        // Now that the user isn't in a session, add them to a session and insert into session_lobby
        self.session_lobby.insert(session_token, lobby.clone());
        self.lobbies.entry(lobby_code.to_string()).and_modify(|lobby| {
            let mut lobby_guard = lobby.lock().unwrap();
            lobby_guard.add_player(player_session.clone());
        });
        Ok(lobby)
    }

    /// Check if the user is currently in a lobby, and remove them from the lobby if they are.
    pub fn leave_lobby(&mut self, session: &Arc<Mutex<Session>>) {
        let session_token = session.lock().unwrap().access_token.clone();
        let previous_lobby = self.session_lobby.get(&session_token).cloned();
        if let Some(lobby) = previous_lobby {
            let mut lobby_guard = lobby.lock().unwrap();
            if lobby_guard.has_player(session.clone()) {
                lobby_guard.remove_player(session.clone());
                // If the lobby becomes empty, remove it from the list of lobbies
                if !lobby_guard.has_players() {
                    self.lobbies.remove(&lobby_guard.code);
                }
            }
            self.session_lobby.remove(&session_token);
        }
    }
}