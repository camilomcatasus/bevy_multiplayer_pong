
use std::sync::Arc;
use std::time::{Duration, Instant};
use lightyear::netcode::ConnectToken;
use log::warn;
use tokio::process::Command;
use tokio::sync::Mutex;
use common::shared::ShortCode;

use crate::error::Error;
use crate::{LobbyInfo, LobbyMaps, LobbyType, CONFIG};

pub async fn create_game(app_state: &AppState) -> Result<u32, Error> {
    let port = { 
        let mut free_ports = app_state.free_ports.lock().await;
        if free_ports.is_empty() {
            return Err(Error::NoFreePorts)
        }
        let last_index = free_ports.len() - 1;
        free_ports.remove(last_index)
    };

    //TODO
    
    let mut child = Command::new(&CONFIG.server_path)
        .arg("--broker-port")
        .arg(CONFIG.private_port.to_string())
        .arg("--server-port")
        .arg(port.to_string())
        .arg("--public-address")
        .arg("127.0.0.1")
        .spawn()?;

    if let Some(id) = child.id() {
        for _ in 0..10 {
            tokio::time::sleep(Duration::from_millis(500)).await;
            {
                let mut lobby_maps = app_state.lobbies.lock().await;
                if let Some(lobby_info) = lobby_maps.lobbies.get_mut(&id) {
                    lobby_info.process = Some(child);
                    return Ok(id)

                }
            }
        }
    }

    //TODO: Log this maybe, add it to programs that need to be killed somehow?
    let _ = child.kill().await;

    Err(Error::SawnTimeOut)
}

#[derive(Clone)]
pub struct AppState {
    pub lobbies: Arc<Mutex<LobbyMaps>>,
    pub free_ports: Arc<Mutex<Vec<u16>>>,
}

impl AppState {
    pub async fn add_lobby(&self, lobby_info: LobbyInfo) {
        let mut lobby_maps = self.lobbies.lock().await;
        match lobby_info.lobby_type {
            LobbyType::Private(short_code) => {
                lobby_maps.short_codes.insert(short_code, lobby_info.id);
                lobby_maps.lobbies.insert(lobby_info.id, lobby_info);
            }
            LobbyType::Public(_) => {
                lobby_maps.lobbies.insert(lobby_info.id, lobby_info);
            }
        }
    }

    #[deprecated]
    pub async fn remove_lobby(&self, lobby_info: &LobbyInfo) {
        let mut lobby_maps = self.lobbies.lock().await;
        match &lobby_info.lobby_type {
            LobbyType::Private(short_code) => {
                lobby_maps.short_codes.remove(short_code);
                lobby_maps.lobbies.remove(&lobby_info.id);
            }
            LobbyType::Public(_name) => {
                lobby_maps.lobbies.remove(&lobby_info.id);
            }
        }
    }

    pub async fn get_lobby_token(&self, lobby_id: &u32) -> Result<ConnectToken, Error> {
        let mut lobby_maps = self.lobbies.lock().await;
        let lobby_info = lobby_maps.lobbies.get_mut(lobby_id)
            .ok_or(Error::NotFound)?;
        lobby_info.token()
    }

    pub async fn contains_id(&self, lobby_id: &u32) -> bool {
        let lobby_maps = self.lobbies.lock().await;
        lobby_maps.lobbies.contains_key(lobby_id)
    }

    pub async fn contains_short_code(&self, short_code: &ShortCode) -> bool {
        let lobby_maps = self.lobbies.lock().await;
        lobby_maps.short_codes.contains_key(short_code)
    }

    pub async fn update_lobby(&self, lobby_info: &LobbyInfo) -> Result<(), Error> {
        let mut lobby_map = self.lobbies.lock().await;
        let existing_lobby_info = lobby_map.lobbies.get_mut(&lobby_info.id).ok_or(Error::NotFound)?;
        existing_lobby_info.player_count = lobby_info.player_count;
        existing_lobby_info.last_checkin = Instant::now();
        Ok(())
    }
}


//TODO: Get rid of magic numbers
pub async fn lobby_validation(app_state: AppState) {
    tokio::spawn(async move {
        loop {
            {
                let mut lobby_maps = app_state.lobbies.lock().await;
                let stalled_lobbies: Vec<(u32, LobbyType)> = lobby_maps.lobbies
                    .values_mut()
                    .filter(|lobby_info| lobby_info.last_checkin.elapsed() > Duration::from_secs(5))
                    .map(|lobby_info| (lobby_info.id, lobby_info.lobby_type.clone()))
                    .collect();

                for stalled_lobby in stalled_lobbies {
                    warn!("Found stalled lobby: {}", stalled_lobby.0);
                    if let Some(lobby_instance) = lobby_maps.lobbies.get_mut(&stalled_lobby.0) {
                        if let Some(process) = &mut lobby_instance.process {
                            let kill_result = process.start_kill();
                            warn!("Attempt to kill process: {:?}", kill_result);
                        }
                    }

                    match &stalled_lobby.1 {
                        LobbyType::Private(short_code) => {
                            lobby_maps.short_codes.remove(short_code);
                            lobby_maps.lobbies.remove(&stalled_lobby.0);
                        }
                        LobbyType::Public(_name) => {
                            lobby_maps.lobbies.remove(&stalled_lobby.0);
                        }
                    }
                }
            }
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
    });
}
