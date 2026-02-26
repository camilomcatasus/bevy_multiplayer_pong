use std::{net::{Ipv4Addr, SocketAddr}, time::Duration};

use bevy::{color::palettes::css::WHITE, input_focus::InputDispatchPlugin, prelude::*, render::{settings::{Backends, WgpuSettings}, RenderPlugin}};
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};
use ::client::{game::GamePlugin, line_renderer, ui::comps::color_picker::ColorPickerPlugin, AppState, PlayerInfo};
use ::client::custom_models::{GameEvent, Player};
use lightyear::prelude::*;
use common::protocol::ProtocolPlugin;
use lightyear::prelude::client::ClientPlugins;
use ::client::ui::screens::ScreenSystem;
use bevy::prelude::*;
use ::client::transient::TransientPlugin;


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
        ScreenSystem,
        InputDispatchPlugin,
        TransientPlugin,
        ProtocolPlugin,
        ClientPlugins { tick_duration: Duration::from_millis(20) },
        ColorPickerPlugin,
        ::client::rendering::RenderPlugin,
        GamePlugin,
    ))
    .add_plugins(EguiPlugin::default())
    .add_plugins(WorldInspectorPlugin::new())
    .init_state::<AppState>()
    .insert_resource(PlayerInfo {
        name: "Player".into(),
        color: WHITE.into(),
    })
    .add_message::<GameEvent>()
    .add_systems(Startup, setup)
    .add_systems(Update, line_renderer::render_lines.run_if(in_state(AppState::Game)))
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
        transform.with_translation(LEFT * 2.)
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
        transform.with_translation(RIGHT * 2.)
    ));

}

fn setup(
    mut commands: Commands,
) {
    let client_addr = SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), 0);
    commands.spawn((
        Client::default(),
        LocalAddr(client_addr),
        //PeerAddr(connection_data.server_addr),
        Link::new(None),
        ReplicationReceiver::default(),
        UdpIo::default(),
    ));
    commands.spawn((
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: bevy::camera::ScalingMode:: FixedVertical { 
                viewport_height: 40.,
            },
            ..OrthographicProjection::default_2d()
        })
    ));

}
