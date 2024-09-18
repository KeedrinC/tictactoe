use std::sync::{Arc, Mutex};
use crate::{lobby::Lobby, session::Session};
use super::utils::new_socket;

fn setup_session() -> Arc<Mutex<Session>> {
    let session = Session::new(new_socket(1111), Some(String::from("keedrin")));
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
    let second_session = Arc::new(Mutex::new(Session::new(new_socket(2222), Some(String::from("keedrin2")))));
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