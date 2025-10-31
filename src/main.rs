use bevy::{prelude::*, render::{settings::{Backends, WgpuSettings}, RenderPlugin}};
use custom_models::{GameEvent, Player};

mod collision;
mod custom_models;
mod line_renderer;
mod game;
mod menu;
mod constants;
mod protocol;
mod shared;
mod client;
mod bundle_fn;
mod observe;
mod ui;
mod transient;


use game::spawn_ball;

use crate::menu::main::button_system;

#[derive(Default, States, Debug, Clone, Eq, PartialEq, Hash)]
pub enum AppState {
    #[default]
    Menu,
    Waiting,
    Lobby,
    Game
}

fn main() {
    let wgpu_settings = WgpuSettings {
        backends: Some(Backends::VULKAN),
        ..Default::default()
    };
    App::new()

    .add_plugins((
        DefaultPlugins.set(
            WindowPlugin {
                primary_window: Some(Window {
                // fill the entire browser window
                fit_canvas_to_parent: true,
                // don't hijack keyboard shortcuts like F5, F6, F12, Ctrl+R etc.
                prevent_default_event_handling: false,
                ..default()
            }),
            ..default()
        }).set(
            RenderPlugin {
                render_creation: wgpu_settings.into(),
                ..default()
        }),
    ))
    .init_state::<AppState>()
    .add_message::<GameEvent>()
    .add_systems(Startup, setup)
    .add_systems(Update, button_system)
    .add_systems(Update, wait_for_players.run_if(in_state(AppState::Lobby)))
    .add_systems(Update, line_renderer::render_lines.run_if(in_state(AppState::Game)))
    .add_systems(OnEnter(AppState::Menu), menu::main::on_enter)
    .add_systems(Update, menu::main::handle_clicks.run_if(in_state(AppState::Menu)))
    .run();
}


const UP: Vec3 = Vec3::new(0., 1., 0.);
const DOWN: Vec3 = Vec3::new(0., -1., 0.);
const LEFT: Vec3 = Vec3::new(-1., 0., 0.);
const RIGHT: Vec3 = Vec3::new(1., 0., 0.);

const PADDLE_SIZE : Vec2 = Vec2::new(0.3, 1.);

fn spawn_player(mut commands: Commands) {
    let transform = Transform::default();
    commands.spawn((
        Player {
            speed: 10f32,
            handle: 0,
            score: 0,
        },
            Sprite {
                color: Color::srgb(0., 0.47, 1.),
                custom_size: Some(PADDLE_SIZE),
                
                ..default()
            },
        transform.clone().with_translation(LEFT * 2.)
    ));

    commands.spawn((
        Player { 
            speed: 10f32,
            handle: 1,
            score: 0,
        },
        Sprite {
            color: Color::srgb(0., 0.47, 1.),
            custom_size: Some(PADDLE_SIZE),
            ..default()
        },
        transform.clone().with_translation(RIGHT * 2.)
    ));

    spawn_ball(&mut commands);
}

fn wait_for_players() {

}

fn setup(
    mut commands: Commands,
) {
    commands.spawn((
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: bevy::camera::ScalingMode:: FixedVertical { 
                viewport_height: 20.,
            },
            ..OrthographicProjection::default_2d()
        })
    ));

    //TODO: Load level on game_startup
    //levels::load_level("./levels/test.json", &mut commands, &Vec::new());
}
