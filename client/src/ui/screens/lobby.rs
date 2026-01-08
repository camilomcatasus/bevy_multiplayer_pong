use std::collections::HashMap;

use bevy::{
    app::Plugin, color::palettes::{css::WHITE, tailwind::GRAY_300}, gizmos, prelude::*
};
use common::protocol::{ClientReadyMessage, MainChannel, PlayerCard, PlayerColor, PlayerId, PlayerListDisplay};
use lightyear::prelude::*;

use crate::{observe::observe, transient::Transient, ui::comps::menu_button, AppState};

#[derive(Message, Clone)]
pub struct NOOP;

#[derive(Resource)]
pub struct PlayerReady(bool);

fn enter_lobby_screen(
    mut commands: Commands,
) {

    commands.insert_resource(PlayerReady(false));
    let node = (Node {
        display: Display::Flex,
        width: vw(100.0),
        height: vh(100.0),
        margin: UiRect::all(px(4.0)),
        ..Default::default()
    },
    children![
        (player_display()),
        (
            Node {
                height: percent(100.0),
                flex_grow: 1.0,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..Default::default()
            },
            children![
                (
                    menu_button("Ready".to_string(), NOOP),
                    observe(|trigger: On<Pointer<Click>>, 
                        mut writer: Single<&mut MessageSender<ClientReadyMessage>>,
                        mut player_ready: ResMut<PlayerReady>,
                        children_query: Query<&Children>,
                        mut button_text: Query<&mut Text>,
                    | {
                        let children = children_query.get(trigger.entity).unwrap();

                        let child_entity = children.iter().last().unwrap();
                        let mut text = button_text.get_mut(child_entity).unwrap();


                        player_ready.0 = !player_ready.0;
                        writer.send::<MainChannel>(ClientReadyMessage(player_ready.0));
                        text.0 = match player_ready.0 {
                            true => "Unready".to_string(),
                            false => "Ready".to_string(),
                        }
                    })
                ),
                (
                    menu_button("Leave".to_string(), NOOP),
                    observe(|
                        _: On<Pointer<Click>>, 
                        client: Single<Entity, With<Client>>,
                        mut commands: Commands
                    | {
                        commands.trigger(Disconnect { entity: *client });
                        commands.set_state(AppState::Menu);
                    })

                    
                ),
            ]
        ),
    ],
    Transient
    );

    commands.spawn(node);
}

fn on_player_disconnect(
    trigger: On<Remove, PlayerId>,
    players: Query<&PlayerId>,
    player_cards: Query<(Entity, &PlayerCard)>,
    mut commands: Commands,
) {
    let Ok(player_id) = players.get(trigger.entity) else { return; };
    for (entity, player_card) in player_cards {
        if player_card.0 == player_id.id {
            commands.entity(entity).despawn();
        }
    }
}

fn on_player_join(
    _trigger: On<Add, PlayerId>,
    state: Res<State<AppState>>,
    commands: Commands,
    player_query: Query<(Entity, &PlayerId, &PlayerColor)>,
    player_cards: Query<(Entity, &PlayerCard, &Children)>,
    text_query: Query<(&mut Text, &mut TextColor)>,
    player_display: Single<(Entity, &PlayerListDisplay)>
) {
    match **state {
        AppState::Lobby => (),
        _ => return,
    }
    handle_lobby(commands, player_query, player_cards, text_query, player_display);
}

fn handle_lobby(
    mut commands: Commands,
    player_query: Query<(Entity, &PlayerId, &PlayerColor)>,
    player_cards: Query<(Entity, &PlayerCard, &Children)>,
    mut text_query: Query<(&mut Text, &mut TextColor)>,
    player_display: Single<(Entity, &PlayerListDisplay)>
) {

    let player_card_map: HashMap<&PeerId, (Entity, &Children)> = 
        player_cards.iter().map(|(entity, player_card, children)| (&player_card.0, (entity, children))).collect();


    for (_entity, player_id, player_color) in player_query {

        match player_card_map.get(&player_id.id) {
            Some((_player_card_entity, children)) => {
                for child in children.iter() {
                    let Ok((mut text, mut text_color)) = text_query.get_mut(child) else { continue; };
                    text.0 = player_id.name.clone();
                    text_color.0 = player_color.0;
                }
            }
            None => {
                let Ok(mut display_commands) = commands.get_entity(player_display.0) 
                else { return; };
                display_commands.with_child(create_player_card(player_id, &player_color.0));
            }
        }
    }
}

fn create_player_card(player_id: &PlayerId, player_color: &Color) -> impl Bundle {
    (
        PlayerCard(player_id.id),
        Node {
            display: Display::Flex,
            width: percent(100.0),
            height: px(40.0),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: px(2.0),
            overflow: Overflow::hidden(),
            border: UiRect::all(Val::Px(1f32)),
            ..Default::default()
            
        },
        BorderColor::all(*player_color),
        children![
            (
                Text::new(player_id.name.clone()),
                TextColor(*player_color)
            ),
            /*(
                Node {
                    height: Val::Percent(80.0),
                    border: UiRect::all(Val::Px(1f32)),
                    ..Default::default()
                },
                BorderColor::all(*player_color)
            )*/
        ]
    )
}
pub(crate) fn player_display() -> impl Bundle {
    (
        Node {
            display: Display::Block,
            flex_direction: FlexDirection::Column,
            height: vh(100.0),
            width: px(320.0),
            border: UiRect::all(px(1.0)),
            row_gap: px(1.0),
            
            ..Default::default()
        },
        BackgroundColor(GRAY_300.into()),
        BorderColor::all(WHITE),
        PlayerListDisplay,
    )
}
pub struct LobbyScreenPlugin;

impl Plugin for LobbyScreenPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_message::<NOOP>()
            .add_systems(OnEnter(AppState::Lobby), (enter_lobby_screen, handle_lobby).chain())
            .add_observer(on_player_join)
            .add_observer(on_player_disconnect)
        ;
    }
}
