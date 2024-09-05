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

#[cfg(test)]
mod tests {
    use std::{net::{IpAddr, Ipv4Addr, SocketAddr}, sync::{Arc, Mutex}};
    use super::Lobby;
    use crate::session::Session;
    fn setup_session() -> Arc<Mutex<Session>> {
        let session = Session::new(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 1111), Some(String::from("keedrin")));
        Arc::new(Mutex::new(session))
    }
    #[test]
    fn test_new_lobby() {
        let session = setup_session();
        let lobby = Lobby::new(session);
        assert!(lobby.has_players());
        assert!(lobby.game.is_none());
        assert_eq!(lobby.player_count(), 1);
    }
    #[test]
    fn test_start_game() {
        let session = setup_session();
        let mut lobby = Lobby::new(session);
        assert!(lobby.game.is_none());
        lobby.start_game();
        assert!(lobby.game.is_some());
    }
    #[test]
    fn test_add_and_remove_player() {
        let session = setup_session();
        let mut lobby = Lobby::new(session.clone());
        assert!(lobby.has_players());
        assert_eq!(lobby.player_count(), 1);
        lobby.remove_player(session.clone());
        assert!(!lobby.has_players());
        assert_eq!(lobby.player_count(), 0);
    }
    #[test]
    fn test_add_players() {
        let session = setup_session();
        let second_session = Arc::new(Mutex::new(Session::new(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 2222), Some(String::from("keedrin2")))));
        let mut lobby = Lobby::new(session.clone());
        lobby.remove_player(session.clone());
        assert!(!lobby.has_players());
        assert_eq!(lobby.player_count(), 0);
        lobby.add_player(session.clone());
        lobby.add_player(second_session.clone());
        assert!(lobby.has_players());
        assert_eq!(lobby.player_count(), 2);
    }
    #[test]
    fn test_has_players() {
        let session = setup_session();
        let mut lobby = Lobby::new(session.clone());
        lobby.remove_player(session.clone());
        assert!(!lobby.has_players());
        assert_eq!(lobby.player_count(), 0);
        lobby.add_player(session.clone());
        assert!(lobby.has_players());
        assert_eq!(lobby.player_count(), 1);
    }
}