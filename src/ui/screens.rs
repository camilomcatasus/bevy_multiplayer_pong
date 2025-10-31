use bevy::prelude::*;

use crate::{
    constants::BROKER_URL, transient::Transient, ui::comps::{menu_button, text_box::TextInput}, AppState
};

fn enter_main_screen(mut commands: Commands) {
    commands.spawn((
    ));
    commands.spawn((
        menu_button("Create Game", MainScreenEvent::CreateGame),
        Transient
    ));

}

fn main_screen_handler(
    text_boxes: Query<(&TextInput, &Text)>,
    mut main_screen_events: MessageReader<MainScreenEvent>
) {
    for event in main_screen_events.read() {
        let url = match event {
            MainScreenEvent::CreateGame => {
                format!("{BROKER_URL}/create-lobby")
            },
            MainScreenEvent::QuickJoin => {
                format!("{BROKER_URL}/quick-join")
            },
            MainScreenEvent::JoinCustomGame => {
                let Ok((_text_input, text)) = text_boxes.single() else { return; };
                let code = &text.0;
                format!("{BROKER_URL}/join-request/{code}")
            }
        };
    }
}


#[derive(Message, Clone)]
pub enum MainScreenEvent {
    CreateGame,
    JoinCustomGame,
    QuickJoin,
}

pub struct ScreenSystem;

impl Plugin for ScreenSystem {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(AppState::Menu), enter_main_screen)
            .add_systems(Update, main_screen_handler.run_if(in_state(AppState::Menu)))
            ;
    }
}

