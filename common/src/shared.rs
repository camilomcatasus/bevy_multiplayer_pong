use std::net::SocketAddr;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
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
