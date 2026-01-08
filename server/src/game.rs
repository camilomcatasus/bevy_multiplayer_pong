use bevy::{math::ops::{sin, cos}, prelude::*};
use common::protocol::{Inputs, PlayerId};
use lightyear::prelude::{input::native::InputMarker, *};

use crate::AppState;

pub struct GamePlugin;

#[derive(Component)]
pub struct PlayerIndex(u16);


#[derive(Bundle)]
pub struct PlayerGameBundle {
    pub player_index: PlayerIndex,
    pub transform: Transform,
    pub input_marker: InputMarker<Inputs>,
    pub mesh: Mesh2d,
    pub mesh_material: MeshMaterial2d<ColorMaterial>,
}

pub fn game_start(
    mut commands: Commands,
    players: Query<(Entity, &PlayerId)>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {



    let mut index = 0u16;
    let player_total: u16 = players.iter().len().try_into().unwrap();
    for (player_entity, player_id) in players {

        let theta = f32::from(index) / f32::from(player_total);
        const RADIUS: f32 = 5f32;

        commands.entity(player_entity).insert((
            PlayerIndex(index),
            InputMarker::<Inputs>::default(),
            Transform {
                translation: vec3(cos(theta) * RADIUS, sin(theta) * RADIUS, 0f32),
                ..Default::default()
            },
        ));

        index += 1;
    }


    //commands.spawn()
}
pub fn game_leave(
    mut commands: Commands,
    players: Query<(Entity, &PlayerId)>,
) {
    for (player_entity, _) in players {
        commands.entity(player_entity).remove::<PlayerGameBundle>();
    }
}

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Game), game_start);
    }
}
