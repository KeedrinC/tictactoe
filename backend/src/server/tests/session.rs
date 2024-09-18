use crate::{session::Session, tests::utils::new_socket};

#[test]
fn test_new_session() {
    let session = Session::new(new_socket(1111), Some(String::from("keedrin")));
    assert!(session.nickname.is_some());
    assert_eq!(session.nickname, Some(String::from("keedrin")));
}

#[test]
fn test_set_session_nickname() {
    let mut session = Session::new(new_socket(1111), Some(String::from("keedrin")));
    assert!(session.nickname.is_some());
    assert_eq!(session.nickname, Some(String::from("keedrin")));
    session.set_nickname("new_nickname");
    assert_eq!(session.nickname, Some(String::from("new_nickname")));
}