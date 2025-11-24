
use std::process::Stdio;
use std::sync::Arc;
use std::time::{Duration, Instant};
use axum::Json;
use log::{info, warn};
use tokio::process::Command;
use tokio::sync::Mutex;
use common::shared::{ConnectionResponse, ShortCode};

use crate::error::Error;
use crate::{public, Config, LobbyInfo, LobbyMaps, LobbyType, CONFIG};

pub async fn create_game(app_state: &AppState, private: bool) -> Result<u32, Error> {
    let port = { 
        let mut free_ports = app_state.free_ports.lock().await;
        if free_ports.is_empty() {
            return Err(Error::NoFreePorts)
        }
        let last_index = free_ports.len() - 1;
        free_ports.remove(last_index)
    };

    //TODO
    
    let mut commands = Command::new(&CONFIG.server_path);

    commands
        .arg("--broker-port")
        .arg(CONFIG.private_port.to_string())
        .arg("--server-port")
        .arg(port.to_string())
        .arg("--public-address")
        .arg("127.0.0.1");

    if private {
        commands.arg("--private");
    }

    let mut child = commands.spawn()?;

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

    let _ = child.kill().await;
    {
        let mut free_ports = app_state.free_ports.lock().await;
        free_ports.push(port);
    }

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

    pub async fn remove_lobbies(&self, lobby_ids: &Vec<u32>) {
        let mut freed_ports = {
            let mut lobby_maps = self.lobbies.lock().await;
            let mut freed_ports: Vec<u16> = Vec::new();
            for lobby_id in lobby_ids {
                let lobby_info_opt = lobby_maps.lobbies.remove(lobby_id);
                if let Some(mut lobby_info) = lobby_info_opt {
                    if let LobbyType::Private(short_code) = lobby_info.lobby_type {
                        lobby_maps.short_codes.remove(&short_code);
                    }

                    if let Some(process) = &mut lobby_info.process {
                        let kill_result = process.start_kill();
                        warn!("Attempt to kill process: {:?}", kill_result);
                    }
                    freed_ports.push(lobby_info.port);
                }
            }
            freed_ports
        };
        let mut free_ports = self.free_ports.lock().await;
        free_ports.append(&mut freed_ports);
    }

    pub async fn get_lobby_response(&self, lobby_id: &u32) -> Result<Json<ConnectionResponse>, Error> {
        let mut lobby_maps = self.lobbies.lock().await;
        let lobby_info = lobby_maps.lobbies.get_mut(lobby_id)
            .ok_or(Error::NotFound)?;
        lobby_info.connection_response()
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

const LOBBY_VALID_WAIT_S: u64 = 5;
const PUBLIC_LOBBY_HANDLER_SLEEP_S: u64 = 5;
const VALIDATION_SLEEP_MS: u64 = 500;

pub async fn lobby_validation(app_state: AppState) {
    tokio::spawn(async move {
        loop {
            {
                let stalled_lobbies: Vec<u32> = {
                    let mut lobby_maps = app_state.lobbies.lock().await;
                    lobby_maps.lobbies
                        .values_mut()
                        .filter(|lobby_info| lobby_info.last_checkin.elapsed() > Duration::from_secs(LOBBY_VALID_WAIT_S))
                        .map(|lobby_info| lobby_info.id)
                        .collect()
                };

                app_state.remove_lobbies(&stalled_lobbies).await;
            }
            
            tokio::time::sleep(Duration::from_millis(VALIDATION_SLEEP_MS)).await;
        }
    });
}

pub async fn public_lobby_handler(app_state: AppState) {
    tokio::spawn(async move {
        loop {
            {
                let public_game_count = {
                    let lobby_map = app_state.lobbies.lock().await;
                    lobby_map.lobbies.values().filter(|lobby_info| lobby_info.lobby_type.is_public()).count()
                };

                if public_game_count < CONFIG.public_games.into() {
                    let new_game_count:usize = (CONFIG.public_games as usize) - public_game_count; 
                    info!("Creating {new_game_count} public lobbies");
                    for _ in 0..new_game_count {
                        //TODO: Logging
                        let _ = create_game(&app_state, false).await;
                    }
                }
            }
            tokio::time::sleep(Duration::from_millis(PUBLIC_LOBBY_HANDLER_SLEEP_S)).await;
        }
    });
}
