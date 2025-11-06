use std::{net::{Ipv4Addr, SocketAddr}, sync::Arc, time::Duration};

use bevy::{color::palettes::css::WHITE, input_focus::InputFocus, prelude::*, state::commands, tasks::{block_on, futures_lite::future, AsyncComputeTaskPool, IoTaskPool, Task}};
use common::shared::ConnectionResponse;
use lightyear::{netcode::{ConnectToken, NetcodeClient}, prelude::{client::NetcodeConfig, *}};
use serde::{Deserialize, Serialize};

use crate::{
    constants::{BROKER_URL, MENU_BUTTON_HEIGHT}, transient::Transient, ui::comps::{menu_button, text_box::{text_box, TextInput}}, AppState
};

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


#[derive(Resource)]
struct ConnectionTask(Task<Option<ConnectionResponse>>);

fn main_screen_handler(
    text_boxes: Query<(&TextInput, &Text)>,
    mut connection_task: ResMut<ConnectionTask>,
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
        let task_handle = tasks.spawn(async move {
            let Ok(res) = reqwest::get(url).await else {
                return None;
            };

            res.json().await.ok()
        });

        connection_task.0 = task_handle;
        commands.set_state(AppState::Waiting);
    }
}


const LOADING_TEXT: &str = "Loading";
#[derive(Resource)]
struct LoadingTimer(Timer);

#[derive(Component)]
struct LoadingText;

fn enter_waiting_screen(mut commands: Commands) {
    commands.spawn((
        Node {
            flex_direction: FlexDirection::Column,
            display: Display::Flex,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            column_gap: Val::Px(8.0),
            row_gap: Val::Px(8.0),
            width: Val::Vw(100.0),
            height: Val::Vh(100.0),
            ..Default::default()
        },
        children![
            (
                LoadingText,
                Text::new(LOADING_TEXT),
                TextFont {
                    font_size: 60.0,
                    ..Default::default()
                },
                TextColor(WHITE.into()),
                
            ),
            (
                menu_button("Cancel", WaitingMessage(None))
            )
        ],
        Transient
    ));

    commands.insert_resource(LoadingTimer(Timer::new(Duration::from_secs(1), TimerMode::Repeating)));
}

#[derive(Message, Clone)]
struct WaitingMessage(Option<&'static str>);

fn waiting_text_anim(
    mut text_query: Query<&mut Text, With<LoadingText>>, 
    time: Res<Time>, 
    mut loading_timer: ResMut<LoadingTimer>
) {

    loading_timer.0.tick(time.delta());
    if loading_timer.0.is_finished() {
        let Ok(mut loading_text) = text_query.single_mut() else {
            return;
        };

        if loading_text.0.len() < LOADING_TEXT.len() + 3 {
            loading_text.0 += ".";
        }
        else {
            loading_text.0 = LOADING_TEXT.to_string();
        }
    }
}

fn waiting_handler(
    mut commands: Commands, 
    mut messages: MessageReader<WaitingMessage>,
    mut connection_task: ResMut<ConnectionTask>,
) {

    if let Some(connection_response) = block_on(future::poll_once(&mut connection_task.0)) {
        match connection_response {
            Some(connection_data) => {

                let Ok(connection_token) = ConnectToken::try_from_bytes(&connection_data.connect_token) else {
                    return;
                };
                let client_addr = SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), 0);
                let auth = Authentication::Token(connection_token);
                commands.spawn((
                    Client::default(),
                    LocalAddr(client_addr),
                    PeerAddr(connection_data.server_addr),
                    Link::new(None),
                    ReplicationReceiver::default(),
                    NetcodeClient::new(auth, NetcodeConfig::default()).unwrap(),
                    UdpIo::default(),
                ));
            }
            None => {
                commands.spawn(crate::ui::comps::error_popup("Could not connect to server".into()));
                commands.set_state(AppState::Menu);
            }
        }

        return;
    }

    for message in messages.read() {
        match message.0 {
            None => {
                commands.set_state(AppState::Menu);
            },
            Some(error_text) => {
                commands.spawn(crate::ui::comps::error_popup(error_text.to_string()));
                commands.set_state(AppState::Menu);
            },
        }
    }
}


#[derive(Message, Clone, Serialize, Deserialize)]
pub enum MainScreenEvent {
    CreateGame,
    JoinCustomGame,
    QuickJoin,
}

pub struct ScreenSystem;

impl Plugin for ScreenSystem {
    fn build(&self, app: &mut App) {
        app
            .add_message::<MainScreenEvent>()
            .add_message::<WaitingMessage>()
            .insert_resource(InputFocus(None))
            .add_systems(OnEnter(AppState::Menu), enter_main_screen)
            .add_systems(Update, main_screen_handler.run_if(in_state(AppState::Menu)))
            .add_systems(OnEnter(AppState::Waiting), enter_waiting_screen)
            .add_systems(PreUpdate, (waiting_text_anim, waiting_handler).run_if(in_state(AppState::Waiting)))
            ;
    }
}

