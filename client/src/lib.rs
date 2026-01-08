use bevy::prelude::*;

pub mod collision;
pub mod custom_models;
pub mod line_renderer;
pub mod game;
pub mod constants;
pub mod client;
pub mod bundle_fn;
pub mod observe;
pub mod ui;
pub mod transient;

#[derive(Default, States, Debug, Clone, Eq, PartialEq, Hash)]
pub enum AppState {
    #[default]
    Menu,
    Waiting,
    Lobby,
    Game
}

#[derive(Resource)]
pub struct PlayerInfo {
    pub name: String,
    pub color: Color,
}
