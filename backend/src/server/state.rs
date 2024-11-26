use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use crate::lobby::Lobby;
use crate::session::Session;

#[derive(Clone)]
pub struct AppState {
    pub lobbies: HashMap<String, Arc<Mutex<Lobby>>>,    // lobbies with currently active users
    pub sessions: HashMap<String, Arc<Mutex<Session>>>, // every connection creates a session object
    pub session_lobby: HashMap<String, Arc<Mutex<Lobby>>>, // map sessions to current lobbies for easy lookup
    pub socket_session: HashMap<SocketAddr, Arc<Mutex<Session>>>,   // map sockets to session for easy lookup
}

impl AppState {
    pub fn new() -> Self {
        let lobbies: HashMap<String, Arc<Mutex<Lobby>>> = HashMap::new();
        let sessions: HashMap<String, Arc<Mutex<Session>>> = HashMap::new();
        let session_lobby: HashMap<String, Arc<Mutex<Lobby>>> = HashMap::new();
        let socket_session: HashMap<SocketAddr, Arc<Mutex<Session>>> = HashMap::new();
        AppState { lobbies, sessions, socket_session, session_lobby }
    }

    pub fn new_lobby(&mut self, player_session: Arc<Mutex<Session>>) -> Arc<Mutex<Lobby>> {
        let lobby: Lobby = Lobby::new(player_session.clone());
        let new_lobby = Arc::new(Mutex::new(lobby.clone()));
        let session_token = player_session.lock().unwrap().token.clone();
        // now make sure the lobby is in both self.lobbies and self.session_lobby
        let _ = self.leave_lobby(player_session.clone());
        self.lobbies.insert(lobby.code, new_lobby.clone());
        self.session_lobby.insert(session_token, new_lobby.clone());
        new_lobby
    }

    pub fn new_session(&mut self, socket: SocketAddr, nickname: Option<String>) -> Arc<Mutex<Session>> {
        let session: Session = Session::new(socket, nickname);
        let session: Arc<Mutex<Session>> = Arc::new(Mutex::new(session));
        let token: String = session.lock().unwrap().token.clone();
        self.sessions.insert(token.clone(), session.clone());
        self.socket_session.insert(socket, session.clone());
        session
    }

    pub fn move_session(&mut self, socket: SocketAddr, token: &str) -> Option<Arc<Mutex<Session>>> {
        let session = self.sessions.get_mut(token);
        if let Some(session) = session {
            let mut s = session.lock().unwrap();
            self.socket_session.remove(&s.address);
            s.address = socket;
            self.socket_session.insert(socket, session.clone());
            self.socket_session.get_mut(&socket).cloned()
        } else { None }
    }

    // the commented lines do not work
    pub fn join_lobby(&mut self, lobby_code: &str, player_session: Arc<Mutex<Session>>) -> Result<Arc<Mutex<Lobby>>, ()> {
        if !self.lobbies.contains_key(lobby_code) { return Err(()); }
        let session_token = player_session.lock().unwrap().token.clone();
        // Check if the session is already in a session, then leave it
        if self.session_lobby.contains_key(&session_token) {
            let _ = self.leave_lobby(player_session.clone());
        };
        let lobby = self.lobbies.get(lobby_code).unwrap().clone();
        // Now that the user isn't in a session, add them to a session and insert into session_lobby
        self.session_lobby.insert(session_token, lobby.clone());
        self.lobbies.entry(lobby_code.to_string()).and_modify(|lobby| {
            let mut lobby_guard = lobby.lock().unwrap();
            lobby_guard.add_player(player_session.clone());
        });
        Ok(lobby)
    }
    pub fn leave_lobby(&mut self, session: Arc<Mutex<Session>>) -> Result<Option<Arc<Mutex<Lobby>>>, ()> {
        let session_token = session.lock().unwrap().token.clone();
        if let Some(lobby) = self.session_lobby.get(&session_token) {
            if let Ok(mut lobby_guard) = lobby.lock() {
                if lobby_guard.has_player(session.clone()) {
                    lobby_guard.remove_player(session.clone());
                    if lobby_guard.player_count() == 0 {
                        self.lobbies.remove(&lobby_guard.code);
                    }
                }
                return Ok(None);
            }
        }
        self.session_lobby.remove(&session_token);
        Err(())
    }
}