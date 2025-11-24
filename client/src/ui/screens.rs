
use bevy::prelude::*;

use crate::ui::screens::{
    main_screen::MainScreenPlugin, 
    waiting::WaitingScreenPlugin
};


mod lobby;
mod main_screen;
mod waiting;



pub struct ScreenSystem;

impl Plugin for ScreenSystem {
    fn build(&self, app: &mut App) {
        app
            .add_plugins((
                MainScreenPlugin,
                WaitingScreenPlugin,
            ))
            ;
    }
}

