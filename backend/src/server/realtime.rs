use std::sync::Arc;
use serde_json::{json, Value};
use tokio::sync::Mutex;
use std::{collections::HashMap, net::SocketAddr};
use game::{Game, Player};
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct AppState {
    pub lobbies: HashMap<String, Lobby>,    // lobbies with currently active users
    pub sessions: HashMap<String, Arc<Mutex<Session>>>, // every connection creates a session object
    pub session_lobby: HashMap<Session, Lobby>, // map sessions to current lobbies for easy lookup
    pub socket_session: HashMap<SocketAddr, Arc<Mutex<Session>>>,   // map sockets to session for easy lookup
}

impl AppState {
    pub fn new() -> Self {
        let lobbies: HashMap<String, Lobby> = HashMap::new();
        let sessions: HashMap<String, Arc<Mutex<Session>>> = HashMap::new();
        let session_lobby: HashMap<Session, Lobby> = HashMap::new();
        let socket_session: HashMap<SocketAddr, Arc<Mutex<Session>>> = HashMap::new();
        AppState { lobbies, sessions, socket_session, session_lobby }
    }
    pub fn new_lobby(&mut self, initiator: Session) -> Result<(), ()> {
        // TODO: check if the initiator is already in a lobby, if so move them to a new lobby
        let lobby: Lobby = Lobby::new(initiator);
        self.lobbies.insert(lobby.id.clone(), lobby.clone());
        Ok(())
    }
    pub fn new_session(&mut self, socket: SocketAddr) -> Option<Arc<Mutex<Session>>> {
        let session: Session = Session::new( socket);
        let session: Arc<Mutex<Session>> = Arc::new(Mutex::new(session));
        let token: String = String::from("random-uuid");
        self.sessions.insert(token.clone(), session.clone());
        self.socket_session.insert(socket, session.clone());
        self.sessions.get(&token);
        Some(session.clone())
    }
    pub async fn move_session(&mut self, socket: SocketAddr, token: &str) -> Option<Arc<Mutex<Session>>> {
        let session = self.sessions.get_mut(token);
        if let Some(session) = session {
            let mut s = session.lock().await;
            self.socket_session.remove(&s.address);
            s.address = socket;
            self.socket_session.insert(socket, session.clone());
            Some(session.clone())
        } else { None }
    }
    pub fn join_lobby(&mut self, session: &Session) -> Result<Option<&Lobby>, ()> {
        let lobby: &mut Lobby = self.session_lobby.get_mut(session).unwrap();
        lobby.add_player(session);
        Ok(Some(lobby))
    }
    pub fn leave_lobby(&mut self, session: &Session) -> Result<Option<&Lobby>, ()> {
        let lobby: &mut Lobby = self.session_lobby.get_mut(session).unwrap();
        lobby.remove_player(session);
        Ok(None)
    }
}

#[derive(Clone)]
pub struct Lobby {
    pub id: String,
    pub game: Option<Game>,
    pub players: [Option<(Session, Option<Player>)>; 2]
}

impl Lobby {
    fn new(initiator: Session) -> Self {
        Lobby {
            id: String::from("000000"),
            game: Some(Game::new()),
            players: [Some((initiator, None)), None]
        }
    }
}
impl Lobby {
    pub fn start_game(&mut self) { todo!() }
    pub fn add_player(&mut self, player: &Session) { todo!() }
    pub fn remove_player(&mut self, player: &Session) { todo!() }
    pub fn has_players(&self) -> bool {
        self.players
            .iter()
            .all(|p| p.is_none())
    }
}

#[derive(Clone, Serialize, PartialEq, Eq, Hash)]
pub struct Session {
    pub id: String,
    pub nickname: String,
    pub address: SocketAddr,
}
impl Session {
    pub fn new(nickname: String, address: SocketAddr) -> Session {
        let id: String = String::from("0000");
        Session { id, nickname, address }
    }
}

#[derive(Debug, Deserialize, PartialEq)]
pub enum Message {
    Connection { token: Option<String> },   // pass a token to resume session after a disconnect
    CreateLobby,                            // creates a new lobby for the current session
    JoinLobby(String),                      // moves the current session to an existing lobby
    Nickname(String),                       // changes the nickname of the current session
    Move(usize)                             // move the session to a spot in their game
}

pub async fn process_messsage(message: Message, socket: SocketAddr, state: Arc<Mutex<AppState>>) -> Result<Value, ()> {
    let mut state = state.lock().await;
    match message {
        Message::Connection { token } => {
            let session = if let Some(token) = &token {
                state.move_session(socket, token).await
            } else { state.new_session(socket) };
            match session {
                Some(session) =>
                    Ok(json!({"Connection": {"token": session.lock().await.id}})),
                None => Err(()),
            }
        },
        Message::CreateLobby => todo!(),
        Message::JoinLobby(_) => todo!(),
        Message::Nickname(_) => todo!(),
        Message::Move(_) => todo!(),
    }
}