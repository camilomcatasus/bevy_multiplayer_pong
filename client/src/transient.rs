use bevy::prelude::*;

use crate::AppState;

#[derive(Component)]
pub struct Transient;

pub struct TransientPlugin;

impl Plugin for TransientPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnExit(AppState::Menu), on_exit)
            .add_systems(OnExit(AppState::Lobby), on_exit)
            .add_systems(OnExit(AppState::Waiting), on_exit)
            .add_systems(OnExit(AppState::Game), on_exit)
            ;
    }
}
pub fn on_exit(query: Query<Entity, With<Transient>>, mut commands: Commands) {
    for item in query {
        commands.entity(item).despawn();
    }
}
