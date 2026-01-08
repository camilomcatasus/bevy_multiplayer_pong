use std::time::Duration;

use bevy::{platform::collections::HashMap, prelude::*};
use common::protocol::{ClientJoin, ClientLeave, ClientReadyMessage, PlayerCard, PlayerId, PlayerListDisplay, PlayerLobbyInfo, SettingsVoteMessage};
use lightyear::prelude::{MessageReceiver, PeerId, RemoteId, Replicate, ServerMultiMessageSender};

use crate::AppState;


#[derive(Resource)]
struct LobbySettings {
    pub timer: Option<Timer>
}

fn client_settings_vote(
    mut receiver: Query<(Entity, &RemoteId, &mut MessageReceiver<SettingsVoteMessage>)>,

) {
}

fn client_ready_handling(
    mut receivers: Query<(Entity, &RemoteId, &mut MessageReceiver<ClientReadyMessage>)>,
    mut commands: Commands,
    mut lobby_info: Query<(&PlayerId, &mut PlayerLobbyInfo)>,
    mut lobby_settings: ResMut<LobbySettings>,

) {
    let mut lobby_info_map: HashMap<PeerId, _> = 
        lobby_info.iter_mut().map(|info_instance| {
            (info_instance.0.id.clone(), info_instance.1)
        }).collect();
    for mut receiver in receivers.iter_mut() {
        for message in receiver.2.receive() {
            let Some(player_lobby_info) = lobby_info_map.get_mut(&receiver.1.0) else {
                warn!("Received a ready message with no client");
                continue;
            };
            player_lobby_info.ready = message.0;
        }
    }

    let ready_count = lobby_info_map.values().filter(|l| l.ready).count();
    let player_count = lobby_info_map.values().count();

    if ready_count == player_count {
        commands.set_state(AppState::Game);
    }

    match &lobby_settings.timer.is_some() {
        false => {
            if ready_count > player_count / 2 && player_count >= 2 {
                lobby_settings.timer = Some(Timer::new(Duration::from_secs(10), TimerMode::Once))
            }
        }
        true => {
            if ready_count < player_count / 2 || player_count <= 1 {
                lobby_settings.timer = None;
            }
        }
    }
}

fn lobby_timeout(
    lobby_settings: Res<LobbySettings>, 
    mut commands: Commands
) {
    if let Some(timer) = &lobby_settings.timer {
        if timer.is_finished() {
            commands.set_state(AppState::Game);
        }
    }
}

pub struct LobbyPlugin;

impl Plugin for LobbyPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(LobbySettings {
            timer: None,
        });
        app.add_systems(Update, (client_settings_vote, client_ready_handling).run_if(in_state(AppState::Lobby)));
    }
}



