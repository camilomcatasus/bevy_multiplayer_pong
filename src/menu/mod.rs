use bevy::prelude::*;

pub mod game;
pub mod lobby;
pub mod main;

pub fn on_exit<T: Component>(query: Query<Entity, With<T>>, mut commands: Commands) {
    for item in query {
        commands.entity(item).despawn();
    }
}


