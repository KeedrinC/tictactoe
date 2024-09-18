use std::{net::SocketAddr, sync::{Arc, Mutex}};
use crate::{session::Session, state::AppState, tests::utils::new_socket};

fn test_session() -> Arc<Mutex<Session>> {
    let address: SocketAddr = new_socket(1111);
    Arc::new(Mutex::new(Session::new(address, Some(String::from("keedrin")))))
}

#[test]
fn test_new_session() {
    let mut state: AppState = AppState::new();
    let address = new_socket(1111);
    let session = state.new_session(address, Some(String::from("keedrin")));
    assert!(session.is_some());
    let session = session.unwrap();
    let session = session.lock().unwrap();
    assert_eq!(session.address, address);
}

#[test]
fn test_move_session() {
    let mut state: AppState = AppState::new();
    let (address, new_address, different_address) = (new_socket(1111), new_socket(2222), new_socket(3333));
    let first_connection = state.new_session(address, Some(String::from("keedrin"))).unwrap();
    let first_connection = first_connection.lock().unwrap().clone();
    let second_connection = state.move_session(new_address, &first_connection.token).unwrap();
    let second_connection = second_connection.lock().unwrap();

    let different_connection = state.new_session(different_address, Some(String::from("keedrin"))).unwrap();
    let different_connection = different_connection.lock().unwrap().clone();

    assert_eq!(first_connection.address, address);
    assert_ne!(first_connection.address, new_address);
    assert_ne!(first_connection.address, different_address);

    assert_eq!(second_connection.address, new_address);
    assert_ne!(second_connection.address, address);
    assert_ne!(second_connection.address, different_address);

    assert_eq!(first_connection.token, second_connection.token);
    assert_ne!(first_connection.token, different_connection.token);
}

#[test]
fn test_new_lobby() {
    let mut state: AppState = AppState::new();
    let mut session: Arc<Mutex<Session>> = test_session();
    let lobby = state.new_lobby(&mut session).unwrap();
    let lobby = lobby.clone();
    let session = session.lock().unwrap();

    assert!(state.lobbies.contains_key(&lobby.code));
    assert!(state.session_lobby.contains_key(&session.token));
    assert_eq!(state.lobbies.get(&lobby.code), Some(&lobby));
    assert_eq!(state.session_lobby.get(&*session.token), Some(&lobby));
}

#[test]
fn test_join_lobby() {
    let mut state: AppState = AppState::new();
    let mut session = state.new_session(new_socket(1111), Some(String::from("keedrin"))).unwrap();
    let lobby = state.new_lobby(&mut session).unwrap().to_owned();
    state.join_lobby(&lobby.code, &mut session);
    let session = session.lock().unwrap();

    assert!(state.lobbies.contains_key(&lobby.code));
    assert!(state.session_lobby.contains_key(&session.token));
    assert_eq!(state.lobbies.get(&lobby.code), Some(&lobby));
    assert_eq!(state.session_lobby.get(&*session.token), Some(&lobby));
}

#[test]
fn test_leave_lobby() {
    let mut state: AppState = AppState::new();
    let mut session = state.new_session(new_socket(1111), Some(String::from("keedrin"))).unwrap();
    let mut another_session: Arc<Mutex<Session>> = state.new_session(new_socket(2222), Some(String::from("keedrin"))).unwrap();
    let lobby = state.new_lobby(&mut session).unwrap().to_owned();

    let s = session.clone();
    let s = (*s).lock().unwrap();
    
    state.join_lobby(&lobby.code, &mut session);
    assert_eq!(state.lobbies.get(&lobby.code), Some(&lobby));
    assert_eq!(state.session_lobby.get(&*s.token), Some(&lobby));
    
    drop(s);
    state.leave_lobby(&mut session);
    // assert!(state.lobbies.contains_key(&lobby.code));
    // assert!(state.session_lobby.contains_key(&s.token));
    // assert_eq!(state.lobbies.get(&lobby.code), None);
    // assert_eq!(state.session_lobby.get(&*s.token), None);
}