use std::{collections::{HashMap, HashSet}, net::SocketAddr, str::FromStr, sync::Arc, time::Instant};
use axum::Json;
use broker::AppState;
use lightyear::netcode::ConnectToken;
use serde::{Serialize, Deserialize};
use tokio::{process::Child, sync::Mutex};
use lazy_static::lazy_static;
use common::shared::{
    ConnectionResponse, LobbyType, ShortCode, PROTOCOL_ID
};
use tracing_subscriber::EnvFilter;
use crate::error::Error;
use log::info;


mod private;
mod public;
mod broker;
mod error;

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub public_port: u16,

    pub private_port: u16,

    pub public_games: u8,

    pub max_game_length_s: u16,

    pub timeout_s: i32,

    pub expire_s: i32,

    pub server_path: String,

    pub allowed_ports: Vec<u16>
}

lazy_static! {
    static ref CONFIG: Config = toml::from_str(
        &std::fs::read_to_string("./config.toml").expect("Could not find config.toml in directory")
    ).expect("Could not parse config.toml");
}


pub struct LobbyMaps {
    pub short_codes: HashMap<ShortCode, u32>,
    pub lobbies: HashMap<u32, LobbyInfo>
}

#[derive(Deserialize, Debug)]
pub struct LobbyInfo {
    pub address: String,
    pub port: u16,
    pub player_count: u8,
    pub player_index: u64,
    pub private_key: [u8; 32],
    pub id: u32, //PID
    #[serde(default = "Instant::now", skip)]
    pub last_checkin: Instant,
    #[serde(skip)]
    pub process: Option<tokio::process::Child>,
    pub lobby_type: LobbyType,
}


impl LobbyInfo {
    pub fn token(&mut self) -> Result<ConnectToken, Error> {
        self.player_index += 1;
        Ok(ConnectToken::build(
            SocketAddr::from_str(&self.address)?,
            PROTOCOL_ID,
            self.player_index,
            self.private_key
        )
            .timeout_seconds(CONFIG.timeout_s)
            .expire_seconds(CONFIG.expire_s)
            .generate()?)
    }

    pub fn connection_response(&mut self) -> Result<Json<ConnectionResponse>, Error> {
        let connection_response = ConnectionResponse {
            connect_token: self.token()?.try_into_bytes()?,
            server_addr: SocketAddr::from_str(&self.address)?,
        };

        Ok(Json(connection_response))
    }
}


#[tokio::main]
async fn main() {
    assert!(usize::from(CONFIG.public_games) < CONFIG.allowed_ports.len(), "Cannot have more public games than available ports");

    let state = AppState {
        lobbies: Arc::new(Mutex::new( LobbyMaps {
            short_codes: HashMap::new(),
            lobbies: HashMap::new(),
        })),
        free_ports: Arc::new(Mutex::new(CONFIG.allowed_ports.clone().into_iter().collect()))
    };

    tracing_subscriber::fmt()
        // This allows you to use, e.g., `RUST_LOG=info` or `RUST_LOG=debug`
        // when running the app to set log levels.
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .or_else(|_| EnvFilter::try_new("broker=trace,tower_http=warn"))
                .unwrap(),
        )
        .init();

    let public_listener = tokio::net::TcpListener::bind(SocketAddr::from(([127,0,0,1], CONFIG.public_port))).await.unwrap();
    let private_listener = tokio::net::TcpListener::bind(SocketAddr::from(([127,0,0,1], CONFIG.private_port))).await.unwrap();
    info!("Starting public server on {}", CONFIG.public_port);
    info!("Starting private server on {}", CONFIG.private_port);
    tokio::join!(
        axum::serve(public_listener, public::create_router(&state)),
        axum::serve(private_listener, private::create_router(&state)),
        broker::lobby_validation(state.clone()),
        broker::public_lobby_handler(state.clone()),
    );
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GameSettings {
    pub max_players: u8,
}

