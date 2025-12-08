
use bevy::{prelude::*, tasks::Task};
use common::shared::ConnectionResponse;

use crate::ui::screens::{
    lobby::LobbyScreenPlugin, main::MainScreenPlugin, waiting::WaitingScreenPlugin
};

mod lobby;
mod main;
mod waiting;

#[derive(Resource)]
pub struct ConnectionTask(Task<Option<ConnectionResponse>>);

pub struct ScreenSystem;

impl Plugin for ScreenSystem {
    fn build(&self, app: &mut App) {
        app
            .add_plugins((
                MainScreenPlugin,
                WaitingScreenPlugin,
                LobbyScreenPlugin
            ))
            ;
    }
}

