use std::sync::{Arc, Mutex};
use game::{Game, Player};
use rand::{thread_rng, Rng};
use serde::Serialize;
use crate::session::Session;

#[derive(Clone, Debug, Serialize)]
pub struct Lobby {
    pub code: String,
    #[serde(skip)]
    pub game: Option<Game>,
    #[serde(skip)]
    pub players: [Option<(Arc<Mutex<Session>>, Player)>; 2]
}

impl PartialEq for Lobby {
    fn eq(&self, other: &Self) -> bool {
        self.code == other.code
    }
}

impl Lobby {
    pub fn new(initiator: Arc<Mutex<Session>>) -> Self {
        let code = (0..4)
            .map(|_| thread_rng().gen_range(0..10))
            .map(|n| n.to_string())
            .collect::<String>();
        let mut lobby = Lobby { code, game: None, players: [None, None] };
        lobby.add_player(initiator);
        lobby
    }
    pub fn start_game(&mut self) {
        self.game = Some(Game::new());
    }
    pub fn add_player(&mut self, player: Arc<Mutex<Session>>) -> &mut Self {
        let index: usize = self.players[0].is_some() as usize;
        self.players[index] = Some((player, match self.players[0] {
            Some((_, player)) => match player {
                Player::X => Player::O,
                Player::O => Player::X,
            },
            None => [Player::X, Player::O][thread_rng().gen_bool(0.5) as usize]
        }));
        self
    }
    pub fn remove_player(&mut self, player: Arc<Mutex<Session>>) -> &mut Self {
        let mut players: Vec<Option<(Arc<Mutex<Session>>, Player)>> = self.players
            .iter()
            .filter_map(|entry| {
                if let Some((p, _)) = entry.clone() {
                    if Arc::ptr_eq(&p, &player) {
                        None
                    } else { Some(entry.clone()) }
                } else { None }
            })
            .collect();
        let diff = self.players.len() - players.len();
        for _ in 0..(diff) { players.push(None); }
        self.players.clone_from_slice(&players[0..]);
        self
    }
    pub fn has_players(&self) -> bool {
        self.players
            .iter()
            .any(|p| p.is_some())
    }
    pub fn player_count(&self) -> u8 {
        self.players
            .iter()
            .fold(0, |acc, player|
                acc + if player.is_some() { 1 } else { 0 })
    }
}