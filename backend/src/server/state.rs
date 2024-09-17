use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use crate::lobby::Lobby;
use crate::session::Session;

#[derive(Clone)]
pub struct AppState {
    pub lobbies: HashMap<String, Lobby>,    // lobbies with currently active users
    pub sessions: HashMap<String, Arc<Mutex<Session>>>, // every connection creates a session object
    pub session_lobby: HashMap<String, Lobby>, // map sessions to current lobbies for easy lookup
    pub socket_session: HashMap<SocketAddr, Arc<Mutex<Session>>>,   // map sockets to session for easy lookup
}

impl AppState {
    pub fn new() -> Self {
        let lobbies: HashMap<String, Lobby> = HashMap::new();
        let sessions: HashMap<String, Arc<Mutex<Session>>> = HashMap::new();
        let session_lobby: HashMap<String, Lobby> = HashMap::new();
        let socket_session: HashMap<SocketAddr, Arc<Mutex<Session>>> = HashMap::new();
        AppState { lobbies, sessions, socket_session, session_lobby }
    }

    pub fn new_lobby(&mut self, initiator: &mut Arc<Mutex<Session>>) -> Option<&mut Lobby> {
        let lobby: Lobby = Lobby::new(initiator.clone());
        self.lobbies.insert(lobby.code.clone(), lobby.clone());
        let lobby = self.lobbies.get_mut(&lobby.code);
        if let Some(lobby) = lobby {
            let session = &initiator.lock().unwrap().token;
            Some(self.session_lobby.entry(session.to_string()).and_modify(|l| {
                l.remove_player(initiator.clone()); // remove them from their previous lobby
                *l = lobby.clone() // now change it to the new lobby
            }).or_insert(lobby.clone()))
        } else { None }
    }

    pub fn new_session(&mut self, socket: SocketAddr, nickname: Option<String>) -> Option<Arc<Mutex<Session>>> {
        let session: Session = Session::new(socket, nickname);
        let session: Arc<Mutex<Session>> = Arc::new(Mutex::new(session));
        let token = &session.lock().unwrap().token;
        self.sessions.insert(token.clone(), session.clone());
        self.socket_session.insert(socket, session.clone());
        self.sessions.get_mut(token).cloned()
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

    pub fn join_lobby(&mut self, id: &str, session: &mut Arc<Mutex<Session>>) -> Option<&mut Lobby> {
        let lobby = self.lobbies.get_mut(id).unwrap();
        lobby.add_player(session.clone());
        Some(lobby)
    }

    pub fn leave_lobby(&mut self, session: &mut Arc<Mutex<Session>>) -> Option<&mut Lobby> {
        let session_id = &session.lock().unwrap().token;
        if let Some(lobby) = self.lobbies.get_mut(session_id) {
            lobby.remove_player(session.clone());
            self.session_lobby.remove(session_id);
            Some(lobby)
        } else { None }
    }
}