use std::f32::consts::PI;

use bevy::{math::ops::{cos, sin}, prelude::*};
use common::protocol::{transform_player, BallState, PlayerColor, PlayerId, PlayerIndex, PlayerState, RADIUS};

use crate::AppState;

#[derive(Bundle)]
struct RenderBundle {
    pub mesh: Mesh2d,
    pub material: MeshMaterial2d<ColorMaterial>
}

#[derive(Resource)]
struct Handles {
    pub mesh_handle: Handle<Mesh>,
    pub ball_mesh: Handle<Mesh>,
}

fn on_game_start(
    mut commands: Commands,
    players: Query<(Entity, &PlayerColor)>,
    handles: Res<Handles>,
    mut color_materials: ResMut<Assets<ColorMaterial>>
) {
    
    for (entity, player_color) in players {
        let material_handle = color_materials.add(ColorMaterial {
            color: player_color.0,
            ..Default::default()
        });
        commands.entity(entity).insert(RenderBundle {
            mesh: Mesh2d(handles.mesh_handle.clone()),
            material: MeshMaterial2d(material_handle)
        });

    }
}

fn render_transform(
    players: Query<(&PlayerState, &mut Transform, &PlayerIndex)>,
) {
    let player_count = players.iter().count() as f32;
    for (player_state, mut player_transform, player_index) in players {
        transform_player(&mut player_transform, player_state, player_index, player_count);
    }
}

fn handle_ball_spawn(
    trigger: On<Add, BallState>,
    ball_states: Query<&BallState>,
    mut commands: Commands,
    handles: Res<Handles>,
    mut color_materials: ResMut<Assets<ColorMaterial>>,
) {
    let Ok(ball_state) = ball_states.get(trigger.entity) else { return; };
    let material_handle = color_materials.add(ColorMaterial {
        color: Color::WHITE,
        ..Default::default()
    });
    commands.entity(trigger.entity).insert((
        Transform::from_translation(ball_state.transform.extend(0f32)),
        Mesh2d(handles.ball_mesh.clone()),
        MeshMaterial2d(material_handle),
    ));
}

fn render_balls(
    balls: Query<(&BallState, &mut Transform)>
) {
    for (ball_state, mut ball_transform) in balls {
        ball_transform.translation = ball_state.transform.extend(0f32);
    }
}


fn on_game_leave(
    mut commands: Commands,
    players: Query<Entity, With<PlayerId>>
) {
    for player_entity in players {
        commands.entity(player_entity).remove::<RenderBundle>();
    }
}

pub struct RenderPlugin;

fn setup(
    mut meshes: ResMut<Assets<Mesh>>,
    mut commands: Commands,
) {
    let paddle_mesh = meshes.add(Rectangle::new(1.0, common::protocol::PADDLE_WIDTH));
    let ball_mesh = meshes.add(Circle::new(0.5));
    commands.insert_resource(
        Handles { 
            mesh_handle: paddle_mesh,
            ball_mesh 
        }
    );
}

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
        app.add_systems(OnEnter(AppState::Game), on_game_start);
        app.add_observer(handle_ball_spawn);
        app.add_systems(Update, (render_transform, render_balls).run_if(in_state(AppState::Game)));
        app.add_systems(OnExit(AppState::Game), on_game_leave);
    }
}
