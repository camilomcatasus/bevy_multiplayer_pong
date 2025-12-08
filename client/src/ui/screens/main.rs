use bevy::{color::palettes::css::WHITE, prelude::*, tasks::IoTaskPool};
use common::shared::{fetch::fetch};
use http_types::{Method, Request, Url};
use serde::{Deserialize, Serialize};

use crate::{
    constants::{
        BROKER_URL, MENU_BUTTON_HEIGHT
    }, 
    transient::Transient, 
    ui::{comps::{
        menu_button, 
        text_box::{text_box, TextInput}
    }, screens::ConnectionTask}, AppState
};

#[derive(Message, Clone, Serialize, Deserialize)]
enum MainScreenEvent {
    CreateGame,
    JoinCustomGame,
    QuickJoin,
}

fn main_screen_ui_bundle() -> impl Bundle {
    (
        Node {
            flex_direction: FlexDirection::Row,
            display: Display::Flex,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            column_gap: Val::Px(8.0),
            width: Val::Vw(100.0),
            height: Val::Vh(100.0),
            ..Default::default()
        },
        children![
            quick_button_group(),
            (
                Node {
                    height: MENU_BUTTON_HEIGHT * 2.2,
                    width: Val::Px(2.0),
                    ..Default::default()
                },
                BackgroundColor(WHITE.into()),
                BorderRadius::all(Val::Px(2.0)),
            ),
            code_join_button_group(),
        ],
        Transient
    )
}

fn quick_button_group() -> impl Bundle {
    (
        Node { 
            flex_direction: FlexDirection::Column,
            display: Display::Flex,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            column_gap: Val::Px(4.0),
            row_gap: Val::Px(4.0),
            ..Default::default()
        },
        children![
            menu_button("Quick Join", MainScreenEvent::QuickJoin),
            menu_button("Create Game", MainScreenEvent::CreateGame),
        ]
    )
}

fn code_join_button_group() -> impl Bundle {
    (
        Node { 
            flex_direction: FlexDirection::Column,
            display: Display::Flex,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            column_gap: Val::Px(4.0),
            row_gap: Val::Px(4.0),
            ..Default::default()
        },
        children![
            text_box(),
            menu_button("Join Custom", MainScreenEvent::JoinCustomGame)
        ]
    )
}

fn enter_main_screen(mut commands: Commands) {
    commands.spawn(main_screen_ui_bundle());
}



fn main_screen_handler(
    text_boxes: Query<(&TextInput, &Text)>,
    mut main_screen_events: MessageReader<MainScreenEvent>,
    mut commands: Commands,
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
        let tasks = IoTaskPool::get();
        let url = Url::parse(&url).expect("Could not parse url");
        let connect_request = Request::new(Method::Get, url);
        let task_handle = tasks.spawn(async move {

            match fetch(connect_request).await {
                Ok(mut res) => {
                    log::info!("Broker Response {res:?}");
                    res.body_json().await.ok()
                },
                Err(err) => {
                    log::error!("Broker Response Error: {err:?}");
                    None
                },
            }
        });

        commands.insert_resource(ConnectionTask(task_handle));
        commands.set_state(AppState::Waiting);
    }
}

pub struct MainScreenPlugin;

impl Plugin for MainScreenPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_message::<MainScreenEvent>()
            .add_systems(OnEnter(AppState::Menu), enter_main_screen)
            .add_systems(Update, main_screen_handler.run_if(in_state(AppState::Menu)))
            ;
    }
}


