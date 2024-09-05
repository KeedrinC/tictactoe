use std::sync::{Arc, Mutex};
use game::{Game, Player};
use rand::{thread_rng, Rng};
use serde::Serialize;
use crate::session::Session;

#[derive(Clone, Serialize)]
pub struct Lobby {
    pub code: String,
    #[serde(skip)]
    pub game: Option<Game>,
    #[serde(skip)]
    pub players: [Option<(Arc<Mutex<Session>>, Player)>; 2]
}

impl Lobby {
    pub fn new(initiator: Arc<Mutex<Session>>) -> Self {
        let mut rng = thread_rng();
        let code = (0..4)
            .map(|_| rng.gen_range(0..10))
            .map(|n| n.to_string())
            .collect::<String>();
        let player: Player = if rng.gen_bool(0.5) { Player:: X } else { Player::O };
        Lobby {
            code,
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