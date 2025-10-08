use core::f32;
use std::f32::consts::PI;

use bevy_asset_loader::loading_state::{LoadingState, LoadingStateAppExt};
use bevy_matchbox::prelude::*;
use bevy::{prelude::*, render::{camera::ScalingMode, settings::{Backends, WgpuSettings}, RenderPlugin}, utils::HashMap};
use bevy_ggrs::*;
use matchbox_socket::{WebRtcSocket, PeerId};
use custom_models::{Ball, Collidable, GameEvent, GameSettings, Player};

mod collision;
mod custom_models;
mod line_renderer;
mod game;
mod menu;
mod constants;


use game::{
    read_local_inputs,
    move_ball,
    move_player,
    spawn_ball,
    handle_game_events
};

#[derive(Default, States, Debug, Clone, Eq, PartialEq, Hash)]
pub enum AppState {
    #[default]
    Menu,
    Lobby,
    Game
}

pub type Config = bevy_ggrs::GgrsConfig<u8, PeerId>;
fn main() {
        let mut wgpu_settings = WgpuSettings::default();
        wgpu_settings.backends = Some(Backends::VULKAN);
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
            GgrsPlugin::<Config>::default()
        ))
        .init_state::<AppState>()
        .rollback_component_with_clone::<Transform>()
        .rollback_resource_with_copy::<GameSettings>()
        .add_event::<GameEvent>()
        .add_systems(Update, wait_for_players.run_if(in_state(AppState::Lobby)))
        .add_systems(Update, line_renderer::render_lines.run_if(in_state(AppState::Game)))
        .add_systems(ReadInputs, game::read_local_inputs)
        .add_systems(GgrsSchedule, (move_player, move_ball, handle_game_events).chain())
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
    ))
    .add_rollback();

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
    ))
    .add_rollback();

    spawn_ball(&mut commands);
}

fn start_matchbox_socket(mut commands: Commands) {
    let room_url = "ws://127.0.0.1:3536/extreme_bevy?next=2";
    info!("connecting to matchbox server: {room_url}");
    commands.insert_resource(MatchboxSocket::new_unreliable(room_url));
}

fn wait_for_players(mut commands: Commands, mut socket: ResMut<MatchboxSocket>) {
    if socket.get_channel(0).is_err() {
        return; // we've already started
    }

    // Check for new connections
    socket.update_peers();
    let players = socket.players();

    let num_players = 2;
    if players.len() < num_players {
        return; // wait for more players
    }

    info!("All peers have joined, going in-game");

    // create a GGRS P2P session
    let mut session_builder = ggrs::SessionBuilder::<Config>::new()
        .with_num_players(num_players)
        .with_input_delay(2);

    for (i, player) in players.into_iter().enumerate() {
        session_builder = session_builder
            .add_player(player, i)
            .expect("failed to add player");
    }


    // move the channel out of the socket (required because GGRS takes ownership of it)
    let channel = socket.take_channel(0).unwrap();

    // start the GGRS session
    let ggrs_session = session_builder
        .start_p2p_session(channel)
        .expect("failed to start session");

    commands.insert_resource(bevy_ggrs::Session::P2P(ggrs_session));
}

