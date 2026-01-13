use std::{net::SocketAddr, str::FromStr, sync::OnceLock, time::Duration};
use lightyear::{netcode::NetcodeServer, prelude::{server::{NetcodeConfig, ServerPlugins, ServerUdpIo, Start}, *}};
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};
use tracing_subscriber::fmt::time;
use serde::Serialize;
use common::{protocol::ProtocolPlugin, shared::{LobbyType, PROTOCOL_ID}};
use bevy::{log::{BoxedLayer, Level, LogPlugin}, prelude::*, state::app::StatesPlugin};
use clap::Parser;
use tracing_appender::{non_blocking::WorkerGuard, rolling};
use tracing_subscriber::Layer;

use crate::{check_in::CheckInPlugin, game::GamePlugin, lobby::LobbyPlugin, player_handling::PlayerHandlingPlugin};

mod check_in;
mod lobby;
mod player_handling;
mod game;

#[derive(Parser, Debug, Resource, Clone)]
pub struct Args {
    #[arg(short, long)]
    broker_port: u16,

    #[arg(short, long)]
    server_port: u16,

    #[arg(short, long)]
    public_address: String,

    #[arg(long)]
    private: bool,

    #[arg(short, long, default_value_t = 20)]
    tick_duration_ms: u64
}

#[derive(Default, States, Debug, Clone, Eq, PartialEq, Hash)]
pub(crate) enum AppState {
    #[default]
    Starting,
    Lobby,
    Game
}

#[derive(Serialize, Debug, Resource, Clone)]
pub struct LobbyInfo {
    pub address: String,
    pub port: u16,
    pub player_count: u8,
    pub player_index: u64,
    pub private_key: [u8; 32],
    pub id: u32, //PID
    pub lobby_type: LobbyType,
}

fn startup(
    mut commands: Commands,
    args: Res<Args>,
    mut exit: MessageWriter<AppExit>
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
    let pid = std::process::id();
    let socket_addr = SocketAddr::from_str(&format!("{}:{}", args.public_address, args.server_port))
        .expect("Could not parse address");
    let server = commands.spawn((
        NetcodeServer::new(NetcodeConfig {
          protocol_id: PROTOCOL_ID,
          ..Default::default()
          }),
        LocalAddr(socket_addr),
        ServerUdpIo::default(),
    )).id();

    commands.trigger(Start{entity: server});

    let client = reqwest::blocking::Client::new();

    let lobby_type = match args.private {
        true => LobbyType::Private(
            ['T', 'E', 'S', 'T', 'I', 'N', 'G', '1', '2', '3']
        ),
        false => LobbyType::Public(format!("Testing - {}", pid)),
    };

    let lobby_info = LobbyInfo {
        address: format!("{}:{}",args.public_address, args.server_port),
        port: args.server_port,
        player_count: 0,
        player_index: 0,
        private_key: [0; 32],
        id: pid,
        lobby_type
    };

    let res = client 
        .post(format!("http://127.0.0.1:{}/game-start", args.broker_port))
        .json(&lobby_info)
        .send();

    if let Err(err) = res {
        log::error!("Could not reach broker. Err: {:?}, shutting down", err);
        exit.write(AppExit::Error(1u8.try_into().unwrap()));
    }
    else {
        log::info!("Found broker, waiting for player");
    }
    commands.insert_resource(lobby_info);
    commands.set_state(AppState::Lobby);
}



static LOG_GUARD: OnceLock<WorkerGuard> = OnceLock::new();



fn main() {
    let args = Args::parse();
    /*tracing_subscriber::fmt()
        .with_ansi(true) // Enable colors
        .with_level(true) // Show log levels
        .with_target(true) // Show the module path (often sufficient)
        .with_file(true) // **Enable file name**
        .with_line_number(true) // **Enable line number**
        .with_timer(time::uptime()) // Use uptime for timestamp
        .compact()
        .with_max_level(Level::INFO)
        .init();*/

    App::new()
        .insert_resource(args.clone())
        .add_plugins((
            DefaultPlugins.set(
                LogPlugin {
                filter: "info,lightyear=debug".to_string(),
                level: bevy::log::Level::TRACE,
                ..default()
            }) ,
            CheckInPlugin,
            ProtocolPlugin, 
            LobbyPlugin,
            PlayerHandlingPlugin,
            ServerPlugins { tick_duration: Duration::from_millis(args.tick_duration_ms)},
            GamePlugin,
        ))
        .add_plugins(EguiPlugin::default())
        .add_plugins(WorldInspectorPlugin::new())
        .add_systems(Startup, startup)
        .init_state::<AppState>()
        .run();
}
