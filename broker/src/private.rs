use axum::{
    extract::State, 
    response::IntoResponse, 
    routing::{post, get}, 
    Json, Router
};

use crate::{AppState, LobbyInfo, LobbyType};

pub async fn started_game(
    State(app_state): State<AppState>,
    Json(private_lobby_info): Json<LobbyInfo>,
) -> impl IntoResponse {
    log::info!("Private Lobby Started: {:?}", private_lobby_info);
    app_state.add_lobby(private_lobby_info).await;
}


pub async fn checkin(
    State(app_state): State<AppState>,
    Json(lobby_info): Json<LobbyInfo>,
) -> impl IntoResponse {
    //TODO: Logging
    let _ = app_state.update_lobby(&lobby_info).await;

}

pub fn create_router(app_state: &AppState) -> Router {
    Router::new()
        .route("/game-start", post(started_game))
        .route("/checkin", post(checkin))
        .route("/admin-panel", get(admin_panel))
        .with_state(app_state.clone())
}

pub async fn admin_panel(
    State(app_state): State<AppState>
) -> impl IntoResponse {
    let lobbies = app_state.lobbies.lock().await;
    let mut public_count = 0;
    let mut private_count = 0;
    let mut lobby_table = String::new(); 
    lobbies.lobbies.values().for_each(|lobby| {
        lobby_table += &format!("<tr><td>{}</td><td>{:?}</td></tr>", lobby.id, lobby.lobby_type);
        match lobby.lobby_type {
            LobbyType::Private(_) => private_count += 1,
            LobbyType::Public(_) => public_count += 1,
        }
    });
    axum::response::Html(format!(r#"
    <style>
    table {{
  border-collapse: collapse;
  border: 2px solid rgb(140 140 140);
  font-family: sans-serif;
  font-size: 0.8rem;
  letter-spacing: 1px;
}}

caption {{
  caption-side: bottom;
  padding: 10px;
  font-weight: bold;
}}

thead,
tfoot {{
  background-color: rgb(228 240 245);
}}

th,
td {{
  border: 1px solid rgb(160 160 160);
  padding: 8px 10px;
}}

td:last-of-type {{
  text-align: center;
}}

tbody > tr:nth-of-type(even) {{
  background-color: rgb(237 238 242);
}}

tfoot th {{
  text-align: right;
}}

tfoot td {{
  font-weight: bold;
}}

    </style>
    <div style="width: 100vw; height: 100vh;">
    
    Public Lobby Count: {public_count}<br/>
    Private Lobby Count: {private_count}
    <table>
    <thead>
    <tr>
    <th>ID</th>
    <th>LOBBY TYPE</th>
    </tr>
    </thead>
    <tbody>
    {lobby_table}
    </tbody>
    </table>
    </div>
"#))
}
