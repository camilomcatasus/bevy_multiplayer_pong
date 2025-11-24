use std::net::SocketAddr;
use serde::{Serialize, Deserialize};

#[cfg(feature = "bevy")]
pub mod fetch;

#[derive(Serialize, Deserialize, Debug)]
pub struct ConnectionResponse {
    pub server_addr: SocketAddr,
    #[serde(with = "serde_bytes")]
    pub connect_token: [u8; 2048],
}

pub type ShortCode = [char; 10];

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum LobbyType {
    Private(ShortCode),
    Public(String)
}

impl LobbyType {
    pub fn is_public(&self) -> bool {
        match self {
            LobbyType::Public(_) => true,
            LobbyType::Private(_) => false,
        }
    }
}
pub const PROTOCOL_ID: u64 = 15234_u64;
