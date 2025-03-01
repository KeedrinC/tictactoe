use std::sync::{Arc, Mutex};
use game::{Game, Player};
use rand::{thread_rng, Rng};
use serde::{ser::SerializeStruct, Serialize, Serializer};
use crate::session::Session;

#[derive(Clone, Debug)]
pub struct Lobby {
    pub code: String,
    pub game: Option<Game>,
    pub players: [Option<(Arc<Mutex<Session>>, Player)>; 2]
}

impl Serialize for Lobby {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer {
        #[derive(Serialize)]
        struct SerializedPlayer { port: u16, symbol: String }
        let p: Vec<Option<SerializedPlayer>> = self.players.iter().map(|player|
            if let Some((session, player)) = player {
                let session = session.lock().unwrap();
                Some(SerializedPlayer {
                    port: session.socket.port(),
                    symbol: match player {
                        Player::X => "X".to_string(),
                        Player::O => "O".to_string(),
                    }
                })
            } else { None }
        ).collect();
        let mut s = serializer.serialize_struct("Lobby", 3)?;
        s.serialize_field("code", &self.code)?;
        s.serialize_field("players", &p)?;
        s.end()
    }
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
        if self.player_count() == 2 {
            self.game = Some(Game::new());
        }
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
    pub fn has_player(&self, session: Arc<Mutex<Session>>) -> bool {
        self.players.iter().any(|player| {
            if let Some(p) = player {
                Arc::ptr_eq(&p.0, &session)
            } else { false }
        })
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