use bevy::prelude::*;
use common::protocol::{ClientJoin, SettingsVoteMessage};
use lightyear::prelude::{MessageReceiver, RemoteId, Replicate, ServerMultiMessageSender};

use crate::AppState;


fn client_settings_vote(
    mut receiver: Query<(Entity, &RemoteId, &mut MessageReceiver<SettingsVoteMessage>)>,

) {
}

pub struct LobbyPlugin;

impl Plugin for LobbyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, client_settings_vote);
    }
}
