use bevy::prelude::*;
use common::protocol::{PlayerColor, PlayerId};

use crate::AppState;

#[derive(Bundle)]
struct RenderBundle {
    pub mesh: Mesh2d,
    pub material: MeshMaterial2d<ColorMaterial>
}

#[derive(Resource)]
struct Handles {
    pub mesh_handle: Handle<Mesh>
}

fn on_transform_added(
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
    let paddle_mesh = meshes.add(Rectangle::new(1.0, 5.0));
    commands.insert_resource(Handles { mesh_handle: paddle_mesh });
}

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
        app.add_systems(OnEnter(AppState::Game), on_transform_added);
        app.add_systems(OnExit(AppState::Game), on_game_leave);
    }
}
