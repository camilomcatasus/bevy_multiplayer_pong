use std::process::exit;
use serde::{Serialize, Deserialize};
use common::shared::LobbyType;
use bevy::{prelude::*, render::{settings::WgpuSettings, RenderPlugin}};
use clap::Parser;

use crate::check_in::CheckInPlugin;

mod check_in;

#[derive(Parser, Debug, Resource)]
pub struct Args {
    #[arg(short, long)]
    broker_port: u16,

    #[arg(short, long)]
    server_port: u16,

    #[arg(short, long)]
    public_address: String,
}

#[derive(Serialize, Debug, Resource, Clone)]
pub struct LobbyInfo {
    pub address: String,
    pub player_count: u8,
    pub player_index: u64,
    pub private_key: [u8; 32],
    pub id: u32, //PID
    pub lobby_type: LobbyType,
}

fn main() {
    let args = Args::parse();
    let pid = std::process::id();
    let client = reqwest::blocking::Client::new();
    let lobby_info = LobbyInfo {
        address: format!("{}:{}",args.public_address, args.server_port),
        player_count: 0,
        player_index: 0,
        private_key: [0; 32],
        id: pid,
        lobby_type: LobbyType::Public(
            "Test".to_string()
        )
    };

    let res = client 
        .post(format!("http://127.0.0.1:{}/game-start", args.broker_port))
        .json(&lobby_info)
        .send();

    if let Err(err) = res {
        println!("Could not reach broker. Err: {:?}, shutting down", err);
        exit(0);
    }

    App::new()
        .add_plugins(DefaultPlugins.set(RenderPlugin {
            render_creation: WgpuSettings {
                backends: None,
                ..Default::default()
            }.into(),
            ..Default::default()
        }))
        .insert_resource(lobby_info)
        .insert_resource(args)
        .add_plugins(CheckInPlugin)
        .run();
}
