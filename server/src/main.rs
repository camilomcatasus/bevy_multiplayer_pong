use std::{fs, net::SocketAddr, str::FromStr, sync::OnceLock, time::Duration};
use lightyear::{netcode::NetcodeServer, prelude::{server::{NetcodeConfig, ServerPlugins, ServerUdpIo, Start}, *}};
use log::LevelFilter;
use serde::Serialize;
use common::{protocol::ProtocolPlugin, shared::{LobbyType, PROTOCOL_ID}};
use bevy::{log::{BoxedLayer, LogPlugin}, prelude::*};
use clap::Parser;
use tracing_appender::{non_blocking::WorkerGuard, rolling};
use tracing_subscriber::Layer;

use crate::check_in::CheckInPlugin;

mod check_in;
mod lobby;
mod player_handling;

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
}



static LOG_GUARD: OnceLock<WorkerGuard> = OnceLock::new();


fn custom_layer(_app: &mut App) -> Option<BoxedLayer> {
    let pid = std::process::id();
    let file_appender = rolling::daily("logs", format!("server_{pid}.log"));
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
    let _ = LOG_GUARD.set(guard);
    Some(bevy::log::tracing_subscriber::fmt::layer()
            .with_writer(non_blocking)
            .with_file(true)
            .with_line_number(true)
            .boxed())
}

fn main() {
    let args = Args::parse();
    App::new()
        .insert_resource(args.clone())
        .add_plugins((
          MinimalPlugins,
          ServerPlugins { tick_duration: Duration::from_millis(args.tick_duration_ms)},
          LogPlugin {
            custom_layer,
            ..Default::default()
          },
          CheckInPlugin
        ))
        .add_plugins(ProtocolPlugin)
        .add_systems(Startup, startup)
        .init_state::<AppState>()
        .run();
}
