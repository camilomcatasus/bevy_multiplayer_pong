use bevy::{
    app::Plugin, gizmos, prelude::*
};

use crate::AppState;

#[component]
struct PlayerDisplay;



fn handler(
    mut commands: Commands
) {
}

pub struct LobbyScreenPlugin;

impl Plugin for LobbyScreenPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(AppState::Lobby), enter)
            .add_systems(Update, handler.run_if(in_state(AppState::Lobby)))
            ;
    }
}
