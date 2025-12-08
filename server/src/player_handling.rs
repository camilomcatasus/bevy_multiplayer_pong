use bevy::prelude::*;
use common::protocol::{ClientJoin, PlayerId};
use lightyear::prelude::{MessageReceiver, RemoteId, ServerMultiMessageSender};

use crate::AppState;



#[derive(Resource)]
pub struct PlayerList {
    pub players: Vec<PlayerId>,
    pub iteration: u64
}

pub struct PlayerHandlingPlugin;

fn on_player_join(
    mut receiver: Query<(Entity, &RemoteId, &mut MessageReceiver<ClientJoin>)>,
    mut multi_writer: ServerMultiMessageSender,
    mut commands: Commands,
    state: Res<State<AppState>>

) {
    for (entity, rid, mut message_receiver) in receiver {
        for message in message_receiver.receive() {
            // Maybe add some name filtering here
            match state.get() {
                AppState::Lobby => (),
                AppState::Game => ()
            }
        }
    }
}

impl Plugin for PlayerHandlingPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(PlayerList {
            players: Vec::new(),
            iteration: 0,
        })
        .add_systems(Update, on_player_join);

    }
}
