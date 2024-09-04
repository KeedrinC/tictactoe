use std::sync::Arc;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use serde_json::{json, Value};
use std::sync::Mutex;
use std::net::SocketAddr;
use game::{Game, Player};
use serde::{Deserialize, Serialize};
use crate::state::AppState;

#[derive(Clone, Serialize)]
pub struct Lobby {
    pub id: String,
    #[serde(skip)]
    pub game: Option<Game>,
    #[serde(skip)]
    pub players: [Option<(Arc<Mutex<Session>>, Player)>; 2]
}

impl Lobby {
    pub fn new(initiator: Arc<Mutex<Session>>) -> Self {
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
        let mut rng = thread_rng();
        let token: String = (&mut rng).sample_iter(Alphanumeric)
            .take(7)
            .map(char::from)
            .collect();
        Session { token, nickname: None, address }
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