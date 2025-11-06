use axum::{extract::{Path, State}, 
    response::IntoResponse, 
    routing::{post, get}, 
    Json, 
    Router,
};
use serde::Serialize;
use common::shared::ShortCode;
use tower_http::trace::TraceLayer;

use crate::{error::Error, AppState, LobbyType};


pub fn create_router(app_state: &AppState) -> Router {
    Router::new()
        .route("/public-lobbies", get(public_lobbies))
        .route("/create-lobby", post(create_game))
        .route("/join-request/{short_code}", get(join_request))
        .with_state(app_state.clone())
        .layer(TraceLayer::new_for_http())
        
}

#[derive(Serialize)]
pub struct LobbyDisplayInfo {
    pub name: String,
    pub id: String,
    pub player_count: u8
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
    Ok(lobby_info.token()?
        .try_into_bytes()?)
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
    let lobby_id = crate::broker::create_game(&app_state).await?;
    let token = app_state.get_lobby_token(&lobby_id).await?;
    
    Ok(token.try_into_bytes()?)
}
