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
        let mut rng = thread_rng();
        let code = (0..4)
            .map(|_| rng.gen_range(0..10))
            .map(|n| n.to_string())
            .collect::<String>();
        let player: Player = if rng.gen_bool(0.5) { Player:: X } else { Player::O };
        Lobby {
            code,
            game: None,
            players: [Some((initiator, player)), None]
        }
    }
    pub fn start_game(&mut self) {
        self.game = Some(Game::new());
    }
    pub fn add_player(&mut self, player: Arc<Mutex<Session>>) -> &mut Lobby {
        let p: Player = if thread_rng().gen_bool(0.5) { Player:: X } else { Player::O };
        for x in 0..self.players.len() {
            if self.players[x].is_none() {
                self.players[x] = Some((player, p));
                break;
            }
        }
        self
    }
    pub fn remove_player(&mut self, player: Arc<Mutex<Session>>) -> &mut Lobby {
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