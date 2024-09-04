use std::net::SocketAddr;
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use serde::Serialize;

#[derive(Clone, Debug, Serialize, PartialEq, Eq, Hash)]
pub struct Session {
    pub token: String,
    pub nickname: Option<String>,
    #[serde(skip)]
    pub address: SocketAddr,
}

impl Session {
    pub fn new(address: SocketAddr) -> Self {
        let mut rng = thread_rng();
        let token: String = (&mut rng).sample_iter(Alphanumeric)
            .take(7)
            .map(char::from)
            .collect();
        Session { token, nickname: None, address }
    }
    pub fn set_nickname(&mut self, nickname: &str) {
        self.nickname = Some(String::from(nickname));
    }
}