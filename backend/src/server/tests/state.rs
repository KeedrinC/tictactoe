use std::sync::{Arc, Mutex};
use crate::{lobby::Lobby, session::Session, state::AppState, tests::utils::new_socket};

// TODO: make separate modules for each group of tests
// TODO: lobby exists function to share between join_lobby, etc.

#[test]
fn test_new_session() {
    let mut state: AppState = AppState::new();
    let session = state.new_session(
        new_socket(1111),
        Some(String::from("keedrin")));
    let session = session.lock().unwrap();

    assert_eq!(session.socket.port(), 1111);
    assert_eq!(session.nickname, Some("keedrin".to_string()));
}

#[test]
fn test_move_session() {
    let mut state: AppState = AppState::new();
    let (address, new_address, different_address) = (new_socket(1111), new_socket(2222), new_socket(3333));
    let first_connection = state.new_session(address, Some(String::from("keedrin")));
    let first_connection = first_connection.lock().unwrap().clone();
    let second_connection = state.move_session(new_address, &first_connection.access_token).unwrap();
    let second_connection = second_connection.lock().unwrap();

    let different_connection = state.new_session(different_address, Some(String::from("keedrin")));
    let different_connection = different_connection.lock().unwrap().clone();

    assert_eq!(first_connection.socket, address);
    assert_ne!(first_connection.socket, new_address);
    assert_ne!(first_connection.socket, different_address);

    assert_eq!(second_connection.socket, new_address);
    assert_ne!(second_connection.socket, address);
    assert_ne!(second_connection.socket, different_address);

    assert_eq!(first_connection.access_token, second_connection.access_token);
    assert_ne!(first_connection.access_token, different_connection.access_token);
}

#[test]
fn test_new_lobby() {
    let mut state: AppState = AppState::new();
    let session: Arc<Mutex<Session>> = state.new_session(new_socket(1111), Some(String::from("player")));
    let lobby: Arc<Mutex<Lobby>> = state.new_lobby(session.clone());

    let code = &lobby.lock().unwrap().code;
    let token = &session.lock().unwrap().access_token;
    assert!(state.lobbies.contains_key(code));
    assert!(state.session_lobby.contains_key(token));

    assert!(!state.lobbies.contains_key("random_code"));
    assert!(!state.session_lobby.contains_key("random_token"));
    assert!(Arc::ptr_eq(state.lobbies.get(code).unwrap(), &lobby));
    assert!(Arc::ptr_eq(state.session_lobby.get(token).unwrap(), &lobby));
    // TODO: test to see if the contain the same users
}

#[test]
fn test_join_lobby_and_leaves_previous_lobby() {
    let mut state: AppState = AppState::new();
    let player: Arc<Mutex<Session>> = state.new_session(new_socket(1111), Some(String::from("player")));
    let friend: Arc<Mutex<Session>> = state.new_session(new_socket(2222), Some(String::from("friend")));
    let player_lobby: Arc<Mutex<Lobby>> = state.new_lobby(player.clone());
    let friend_lobby: Arc<Mutex<Lobby>> = state.new_lobby(friend.clone());
    
    let player_token = player.lock().unwrap().access_token.clone();
    let friend_token = friend.lock().unwrap().access_token.clone();
    let player_lobby_guard = player_lobby.lock().unwrap().clone();
    let friend_lobby_guard = friend_lobby.lock().unwrap().clone();
    let player_lobby_code: &String = &player_lobby_guard.code.clone();
    let friend_lobby_code: &String = &friend_lobby_guard.code.clone();

    assert_eq!(state.lobbies.len(), 2); // player and friend have their own separate lobbies
    assert_eq!(player_lobby_guard.player_count(), 1); // player in their own created lobby
    assert_eq!(friend_lobby_guard.player_count(), 1); // friend in their own created lobby

    // Make sure both lobbies exist in the state object
    assert!(state.lobbies.contains_key(player_lobby_code));
    assert!(state.lobbies.contains_key(friend_lobby_code));
    assert!(state.session_lobby.contains_key(&player_token));
    assert!(state.session_lobby.contains_key(&friend_token));

    // Make sure the pointers in the lobbies HashMaps point to the same pointers we created to test
    assert!(Arc::ptr_eq(state.lobbies.get(player_lobby_code).unwrap(), &player_lobby));
    assert!(Arc::ptr_eq(state.lobbies.get(friend_lobby_code).unwrap(), &friend_lobby));
    assert!(Arc::ptr_eq(state.session_lobby.get(&player_token).unwrap(), &player_lobby));
    assert!(Arc::ptr_eq(state.session_lobby.get(&friend_token).unwrap(), &friend_lobby));

    // player wants to join friend's lobby, so we use the join_lobby function
    state.join_lobby(friend_lobby_code, player.clone()).unwrap();

    assert_eq!(state.lobbies.len(), 1);
    assert!(!state.lobbies.contains_key(player_lobby_code)); // player's previous lobby shouldn't exist anymore
    assert!(state.lobbies.contains_key(friend_lobby_code)); // friend's lobby should still exist
    assert!(Arc::ptr_eq(state.lobbies.get(friend_lobby_code).unwrap(), &friend_lobby));
    assert!(Arc::ptr_eq(state.session_lobby.get(&player_token).unwrap(), &friend_lobby)); // both players are in friend's lobby
    assert!(Arc::ptr_eq(state.session_lobby.get(&friend_token).unwrap(), &friend_lobby));
    let lobby = state.lobbies.get(friend_lobby_code).unwrap().lock().unwrap();
    assert_eq!(lobby.player_count(), 2); // both should now be in the lobby
}

#[test]
fn test_leave_lobby() {
    let mut state: AppState = AppState::new();
    let player: Arc<Mutex<Session>> = state.new_session(new_socket(1111), Some(String::from("player")));
    let friend: Arc<Mutex<Session>> = state.new_session(new_socket(2222), Some(String::from("friend")));
    let player_lobby: Arc<Mutex<Lobby>> = state.new_lobby(player.clone());
    let friend_lobby: Arc<Mutex<Lobby>> = state.new_lobby(friend.clone());
    
    let player_token = player.lock().unwrap().access_token.clone();
    let friend_token = friend.lock().unwrap().access_token.clone();
    let player_lobby_guard = player_lobby.lock().unwrap().clone();
    let friend_lobby_guard = friend_lobby.lock().unwrap().clone();
    let player_lobby_code: &String = &player_lobby_guard.code.clone();
    let friend_lobby_code: &String = &friend_lobby_guard.code.clone();

    assert_eq!(state.lobbies.len(), 2); // player and friend have their own separate lobbies
    assert_eq!(player_lobby_guard.player_count(), 1); // player in their own created lobby
    assert_eq!(friend_lobby_guard.player_count(), 1); // friend in their own created lobby

    // Make sure both lobbies exist in the state object
    assert!(state.lobbies.contains_key(player_lobby_code));
    assert!(state.lobbies.contains_key(friend_lobby_code));
    assert!(state.session_lobby.contains_key(&player_token));
    assert!(state.session_lobby.contains_key(&friend_token));

    // Make sure the pointers in the lobbies HashMaps point to the same pointers we created to test
    assert!(Arc::ptr_eq(state.lobbies.get(player_lobby_code).unwrap(), &player_lobby));
    assert!(Arc::ptr_eq(state.lobbies.get(friend_lobby_code).unwrap(), &friend_lobby));
    assert!(Arc::ptr_eq(state.session_lobby.get(&player_token).unwrap(), &player_lobby));
    assert!(Arc::ptr_eq(state.session_lobby.get(&friend_token).unwrap(), &friend_lobby));

    // player wants to join friend's lobby, so we use the join_lobby function
    state.join_lobby(friend_lobby_code, player.clone()).unwrap();

    assert_eq!(state.lobbies.len(), 1);
    assert!(!state.lobbies.contains_key(player_lobby_code)); // player's previous lobby shouldn't exist anymore
    assert!(state.lobbies.contains_key(friend_lobby_code)); // friend's lobby should still exist
    assert!(Arc::ptr_eq(state.lobbies.get(friend_lobby_code).unwrap(), &friend_lobby));
    assert!(Arc::ptr_eq(state.session_lobby.get(&player_token).unwrap(), &friend_lobby)); // both players are in friend's lobby
    assert!(Arc::ptr_eq(state.session_lobby.get(&friend_token).unwrap(), &friend_lobby));

    assert_eq!(friend_lobby.lock().unwrap().player_count(), 2); // both should now be in the friend's lobby
    state.leave_lobby(&player); // should be one player left, we only remove the session_lobby entry
    assert_eq!(friend_lobby.lock().unwrap().player_count(), 1); // only one should be in the lobby
    state.leave_lobby(&friend); // should be no players left, we the session_lobby entry and lobbies entry
}