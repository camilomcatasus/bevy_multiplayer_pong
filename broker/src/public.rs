use axum::{extract::{Path, State}, 
    response::IntoResponse, 
    routing::{post, get}, 
    Json, 
    Router,
};
use serde::Serialize;
use common::shared::{ConnectionResponse, ShortCode};
use tower_http::trace::TraceLayer;

use crate::{error::Error, AppState, LobbyInfo, LobbyType};


pub fn create_router(app_state: &AppState) -> Router {
    Router::new()
        .route("/public-lobbies", get(public_lobbies))
        .route("/create-lobby", get(create_game))
        .route("/join-request/{short_code}", get(join_request))
        .route("/quick-join", get(quick_join))
        .with_state(app_state.clone())
        .layer(TraceLayer::new_for_http())
        
}

#[derive(Serialize)]
pub struct LobbyDisplayInfo {
    pub name: String,
    pub id: String,
    pub player_count: u8
}

pub async fn quick_join(
    State(app_state): State<AppState>,
) -> Result<impl IntoResponse, Error> {
    let mut lobby_maps = app_state.lobbies.lock().await;
    let smallest_public_lobby: &mut LobbyInfo = lobby_maps.lobbies.values_mut()
        .filter(|lobby_info| lobby_info.lobby_type.is_public())
        .min_by_key(|lobby_info| lobby_info.player_count)
        .ok_or(Error::ServerError)?;

    smallest_public_lobby.connection_response()
}

pub async fn join_request(
    State(app_state): State<AppState>,
    Path(short_code): Path<ShortCode>
) -> Result<impl IntoResponse, Error> {
    let mut lobby_maps = app_state.lobbies.lock().await;
    let lobby_id = *lobby_maps.short_codes
        .get(&short_code)
        .ok_or(Error::NotFound)?;
    let lobby_info = lobby_maps.lobbies.get_mut(&lobby_id).ok_or(Error::NotFound)?;
    lobby_info.connection_response()
}

async fn public_lobbies(
    State(app_state): State<AppState>,
) -> impl IntoResponse {
    let lobby_map =  app_state.lobbies.lock().await;
    let lobby_list : Vec<LobbyDisplayInfo> = lobby_map.lobbies.values()
        .filter_map(|lobby_info| 
            match &lobby_info.lobby_type {
                LobbyType::Public(name) => Some(LobbyDisplayInfo {
                    name: name.to_string(), 
                    id: lobby_info.id.to_string(),
                    player_count: lobby_info.player_count
                }),
                LobbyType::Private(_) => None,
            }
        )
        .collect();

    Json(lobby_list)
}

pub async fn create_game(
    State(app_state): State<AppState>,
) -> Result<impl IntoResponse, Error> {
    let lobby_id = crate::broker::create_game(&app_state, true).await?;
    app_state.get_lobby_response(&lobby_id).await
}
