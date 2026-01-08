use bevy::{color::palettes::css::WHITE, ecs::observer, prelude::*, tasks::IoTaskPool};
use common::shared::{fetch::fetch};
use http_types::{Method, Request, Url};
use serde::{Deserialize, Serialize};

use crate::{
    constants::{
        BROKER_URL, MENU_BUTTON_HEIGHT, MENU_BUTTON_WIDTH
    }, observe::observe, transient::Transient, ui::{comps::{
        color_picker::{self, ColorPicked, ColorPickerMaterial}, menu_button, text_box::{text_box, TextChanged, TextInput}
    }, screens::ConnectionTask}, AppState, PlayerInfo
};

#[derive(Message, Clone, Serialize, Deserialize)]
enum MainScreenEvent {
    CreateGame,
    JoinCustomGame,
    QuickJoin,
    NOOP
}

#[derive(Component)]
struct ShortCode;


#[derive(Component)]
struct NameInput;

#[derive(Component)]
struct ColorPickerInput;

#[derive(Component)]
struct ColorPickerContainer;

fn main_screen_connection_buttons() -> impl Bundle {
    (
        Node {
            flex_direction: FlexDirection::Row,
            display: Display::Flex,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            column_gap: Val::Px(8.0),
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
    )
}

fn user_info(ui_materials: &mut ResMut<Assets<ColorPickerMaterial>>, player_info: &PlayerInfo) -> impl Bundle {
    let color = player_info.color;
    (
        Node {
            flex_direction: FlexDirection::Row,
            display: Display::Flex,
            column_gap: px(16.0),
            position_type: PositionType::Relative,
            ..Default::default()
        },
        ZIndex(100),
        children![
            (
                text_box(NameInput, player_info.name.clone()),
                observe(|trigger: On<TextChanged>, mut player_info: ResMut<PlayerInfo>| {
                    player_info.name = trigger.text.clone();
                })
            ),
            (
                menu_button(color.to_srgba().to_hex(), MainScreenEvent::NOOP),
                ColorPickerInput,
                observe(|
                    _trigger: On<Pointer<Click>>, 
                    mut picker_node: Single<&mut Node, With<ColorPickerContainer>>
                | {
                    picker_node.display = Display::Flex;
                })
            ),
            (
                Node {
                    position_type: PositionType::Absolute,
                    left: px(0),
                    right: px(0),

                    top: MENU_BUTTON_HEIGHT * 1.2,
                    display: Display::None,
                
                    ..Default::default()
                },
                ColorPickerContainer,
                children![
                    (
                        color_picker::single_color_gradient(1u32, LinearRgba::RED, px(400), ui_materials),
                        observe(|
                            trigger: On<ColorPicked>, 
                            mut picker_node: Single<&mut Node, With<ColorPickerContainer>>,
                            color_input: Single<&Children, With<ColorPickerInput>>,
                            mut button_text: Query<(&mut Text, &mut TextColor)>,
                            mut player_info: ResMut<PlayerInfo>,
                        | {
                            let Some(child_entity) = color_input.iter().last() else { return; };
                            let Ok((mut text, mut text_color)) = button_text.get_mut(child_entity) else { return; };
                            text_color.0 = trigger.color.into();
                            player_info.color = trigger.color.into();
                            let test: Srgba = trigger.color.into();
                            text.0 = test.to_hex();

                            picker_node.display = Display::None;

                        })
                    )

                ]
            )
        ]
    )
}

fn main_screen_ui_bundle(ui_materials: &mut ResMut<Assets<ColorPickerMaterial>>, player_info: &PlayerInfo) -> impl Bundle {
    (
        Node {
            flex_direction: FlexDirection::Column,
            display: Display::Flex,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            row_gap: Val::Px(8.0),
            width: vw(100.0),
            height: vh(100.0),
            ..Default::default()
        },
        children![
            user_info(ui_materials, player_info),
            (
                Node {
                    height: px(2.0),
                    width: MENU_BUTTON_WIDTH * 2.2,
                    ..Default::default()
                },
                BackgroundColor(WHITE.into()),
            ),
            main_screen_connection_buttons()
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
            menu_button("Quick Join".to_string(), MainScreenEvent::QuickJoin),
            menu_button("Create Game".to_string(), MainScreenEvent::CreateGame),
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
            text_box(ShortCode, "".to_string()),
            menu_button("Join Custom".to_string(), MainScreenEvent::JoinCustomGame)
        ]
    )
}

fn enter_main_screen(
    mut commands: Commands, 
    mut ui_materials: ResMut<Assets<ColorPickerMaterial>>,
    player_info: Res<PlayerInfo>,
) {
    commands.spawn(main_screen_ui_bundle(&mut ui_materials, &player_info));
}

fn main_screen_handler(
    short_code_ui: Single<(&TextInput, &Text), With<ShortCode>>,
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
                let (_text_input, text) = (short_code_ui.0, short_code_ui.1);
                let code = &text.0;
                format!("{BROKER_URL}/join-request/{code}")
            },
            MainScreenEvent::NOOP => continue,
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


