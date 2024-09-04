use std::sync::Arc;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use serde_json::{json, Value};
use std::sync::Mutex;
use std::{collections::HashMap, net::SocketAddr};
use game::{Game, Player};
use serde::{Deserialize, Serialize};

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
        self.lobbies.insert(lobby.id.clone(), lobby.clone());
        let lobby = self.lobbies.get_mut(&lobby.id);
        if let Some(lobby) = lobby {
            let session = &initiator.lock().unwrap().token;
            Some(self.session_lobby.entry(session.to_string()).and_modify(|l| {
                l.remove_player(initiator.clone()); // remove them from their previous lobby
                *l = lobby.clone() // now change it to the new lobby
            }).or_insert(lobby.clone()))
        } else { None }
    }
    pub fn new_session(&mut self, socket: SocketAddr) -> Option<&mut Arc<Mutex<Session>>> {
        let session: Session = Session::new( socket);
        let session: Arc<Mutex<Session>> = Arc::new(Mutex::new(session));
        let token = &session.lock().unwrap().token;
        self.sessions.insert(token.clone(), session.clone());
        self.socket_session.insert(socket, session.clone());
        self.sessions.get_mut(token)
    }
    pub fn move_session(&mut self, socket: SocketAddr, token: &str) -> Option<&mut Arc<Mutex<Session>>> {
        let session = self.sessions.get_mut(token);
        if let Some(session) = session {
            let mut s = session.lock().unwrap();
            self.socket_session.remove(&s.address);
            s.address = socket;
            self.socket_session.insert(socket, session.clone());
            self.socket_session.get_mut(&socket)
        } else { None }
    }
    pub fn join_lobby(&mut self, id: String, session: &mut Arc<Mutex<Session>>) -> Option<&mut Lobby> {
        let lobby = self.lobbies.get_mut(&id).unwrap();
        lobby.add_player(session.clone());
        Some(lobby)
    }
    pub fn leave_lobby(&mut self, session: &mut Arc<Mutex<Session>>) -> Option<&mut Lobby> {
        let token = &session.lock().unwrap().token;
        let lobby: &mut Lobby = self.session_lobby.get_mut(token).unwrap();
        lobby.remove_player(session.clone())
    }
}

#[derive(Clone, Serialize)]
pub struct Lobby {
    pub id: String,
    #[serde(skip)]
    pub game: Option<Game>,
    #[serde(skip)]
    pub players: [Option<(Arc<Mutex<Session>>, Player)>; 2]
}

impl Lobby {
    fn new(initiator: Arc<Mutex<Session>>) -> Self {
        let x_or_o = thread_rng().gen_bool(0.5);
        let player = if x_or_o { Player:: X } else { Player::O };
        Lobby {
            id: String::from("000000"),
            game: Some(Game::new()),
            players: [Some((initiator, player)), None]
        }
    }
    pub fn start_game(&mut self) { todo!() }
    pub fn add_player(&mut self, _player: Arc<Mutex<Session>>) -> Option<&mut Lobby> { todo!() }
    pub fn remove_player(&mut self, _player: Arc<Mutex<Session>>) -> Option<&mut Lobby> { todo!() }
    pub fn has_players(&self) -> bool {
        self.players
            .iter()
            .all(|p| p.is_none())
    }
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq, Hash)]
pub struct Session {
    pub token: String,
    pub nickname: Option<String>,
    #[serde(skip)]
    pub address: SocketAddr,
}

impl Session {
    pub fn new(address: SocketAddr) -> Self {
        let nickname: Option<String> = Some(String::from("nickname"));
        let mut rng = thread_rng();
        let token: String = (&mut rng).sample_iter(Alphanumeric)
            .take(7)
            .map(char::from)
            .collect();
        Session { token, nickname, address }
    }
    pub fn set_nickname(&mut self, nickname: &str) {
        self.nickname = Some(String::from(nickname));
    }
}

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