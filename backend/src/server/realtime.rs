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

#[derive(Debug, Deserialize)]
pub enum Message { CreateLobby, JoinLobby(String), Nickname(String), Move(usize) }
pub async fn process_messsage(message: String, socket: SocketAddr, state: Arc<Mutex<AppState>>) -> Result<Value, ()> {
    let mut state = state.lock().await;
    let session: Session = Session::new(String::from("nickname"), socket);
    match serde_json::from_str::<Message>(&message) {
        Err(_) => todo!(),
        Ok(message) => {
            match message {
                Message::CreateLobby => {
                    (*state).new_lobby(session);
                    Ok(json!({"success": true}))
                },
                Message::JoinLobby(_) => todo!(),
                Message::Nickname(_) => todo!(),
                Message::Move(_) => todo!(),
            }
        },
    }
}