use bevy::{ecs::entity, prelude::*};
use common::protocol::{ClientJoin, PlayerColor, PlayerId, PlayerLobbyInfo, PlayerState, SEND_INTERVAL};
use lightyear::{connection::network_target::Target, prelude::{server::ClientOf, *}};

use crate::AppState;


pub struct PlayerHandlingPlugin;

fn handle_new_client(
    trigger: On<Add, LinkOf>,
    mut commands: Commands,
) {
    commands.entity(trigger.entity)
        .insert(
            ReplicationSender::new(SEND_INTERVAL, SendUpdatesMode::SinceLastAck, false)
        );
}

fn handle_client_disconnect(
    trigger: On<Add, Disconnected>,
    clients: Query<&RemoteId, With<ClientOf>>,
    player_data: Query<(Entity, &PlayerId)>,
    mut commands: Commands,
) {
    let Ok(disconnected_client) = clients.get(trigger.entity) else {
        error!("Could not find disconnected client");
        return;
    };

    if let Some((entity, _)) = player_data.iter().find(|(_entity, player_id)| player_id.id == disconnected_client.0) {
        commands.entity(entity).despawn();
    }
    //query.iter().for_each(|);
}

impl Plugin for PlayerHandlingPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_observer(handle_new_client)
        .add_systems(Update, on_player_join)
        .add_observer(handle_client_disconnect)
        ;
    }
}


fn on_player_join(
    receiver: Query<(Entity, &RemoteId, &mut MessageReceiver<ClientJoin>)>,
    state: Res<State<AppState>>,
    mut commands: Commands,
) {
    for (entity, rid, mut message_receiver) in receiver {
        for message in message_receiver.receive() {
            // Maybe add some name filtering here
            info!("Player - {} joined", &message.name);
            commands.spawn(
                (
                    PlayerId {
                        name: message.name,
                        id: rid.0
                    },
                    PlayerColor(message.color),
                    PlayerLobbyInfo{ ready: false },
                    Replicate::to_clients(Target::All),
                )
            );
        }
    }
}

