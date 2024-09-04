use std::sync::{Arc, Mutex};
use game::{Game, Player};
use rand::{thread_rng, Rng};
use serde::Serialize;
use crate::realtime::Session;

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