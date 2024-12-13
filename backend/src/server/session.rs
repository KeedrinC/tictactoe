use std::net::SocketAddr;
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use serde::Serialize;

#[derive(Clone, Debug, Serialize, PartialEq, Eq, Hash)]
pub struct Session {
    pub access_token: String,
    pub nickname: Option<String>,
    #[serde(skip)]
    pub socket: SocketAddr,
}

impl Session {
    pub fn new(socket: SocketAddr, nickname: Option<String>) -> Self {
        let mut rng = thread_rng();
        let token: String = (&mut rng).sample_iter(Alphanumeric)
            .take(7)
            .map(char::from)
            .collect();
        Session { access_token: token, nickname, socket }
    }
    pub fn set_nickname(&mut self, nickname: &str) {
        self.nickname = Some(String::from(nickname));
    }
}