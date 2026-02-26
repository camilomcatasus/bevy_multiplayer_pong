use std::f32::consts::PI;

use bevy::{math::ops::{sin, cos}, prelude::*};
use clap::builder::styling::Color;
use common::protocol::{handle_input, move_balls, transform_player, GameStartMessage, Inputs, MainChannel, PlayerColor, PlayerId, PlayerIndex, PlayerState};
use lightyear::{connection::network_target::Target, prelude::{input::native::{ActionState, InputMarker}, *}};
use crate::AppState;
use common::protocol::RADIUS;

pub struct GamePlugin;



#[derive(Bundle)]
pub struct PlayerGameBundle {
    pub player_index: PlayerIndex,
    pub transform: Transform,
    pub input_marker: InputMarker<Inputs>,
}

pub fn game_start(
    mut commands: Commands,
    players: Query<Entity, With<PlayerId>>,
    writers: Query<&mut MessageSender<GameStartMessage>>,
) {
    info!("Game Start");
    let mut index = 0u16;
    let player_total: u16 = players.iter().len().try_into().unwrap();
    let player_count = player_total as f32;
    for player_entity in players {


        let theta = f32::from(index) / player_count * 2.0 * PI;

        commands.entity(player_entity).insert((
            PlayerIndex(index),
            Transform {
                rotation: Quat::from_axis_angle(vec3(0.0, 0.0, 1.0), theta),
                translation: vec3(cos(theta) * RADIUS, sin(theta) * RADIUS, 0f32),
                ..Default::default()
            },
        ));

        index += 1;
    }

    let direction = vec2(1.0, 0.0);
    commands.spawn(
        (
            common::protocol::BallState {
                transform: Vec2::default(),
                direction,
                speed: 0.2f32,
            },
            //PredictionTarget::to_clients(NetworkTarget::All),
            Replicate::to_clients(Target::All),
        )
    );

    info!("Sending game start");
    for mut writer in writers {
        writer.send::<MainChannel>(GameStartMessage { 
            max_score: 10, 
            game_speed: common::protocol::GameSpeed::Normal })
    }
}


pub fn game_leave(
    mut commands: Commands,
    players: Query<(Entity, &PlayerId)>,
) {
    for (player_entity, _) in players {
        commands.entity(player_entity).remove::<PlayerGameBundle>();
    }
}

pub fn server_handle_input(
    players: Query<(&PlayerIndex, &ActionState<Inputs>, &mut Transform, &mut PlayerState)>
) {
    let player_count = players.iter().len() as f32;
    for (player_index, inputs, transform, player_state) in players {
        handle_input(player_state, inputs);
    }
}

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Game), game_start);
        app.add_systems(FixedUpdate, (server_handle_input, move_balls).chain().run_if(in_state(AppState::Game)));
        app.add_systems(OnExit(AppState::Game), game_leave);
    }
}
