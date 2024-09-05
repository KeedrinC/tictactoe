use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use crate::session::Session;

#[test]
fn test_new_session() {
    let address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 1111);
    let session = Session::new(address, Some(String::from("keedrin")));
    assert!(session.nickname.is_some());
    assert_eq!(session.nickname, Some(String::from("keedrin")));
}

#[test]
fn test_set_session_nickname() {
    let address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 1111);
    let mut session = Session::new(address, Some(String::from("keedrin")));
    assert!(session.nickname.is_some());
    assert_eq!(session.nickname, Some(String::from("keedrin")));
    session.set_nickname("new_nickname");
    assert_eq!(session.nickname, Some(String::from("new_nickname")));
}