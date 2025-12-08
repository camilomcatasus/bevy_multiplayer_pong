use std::{net::{Ipv4Addr, SocketAddr}, time::Duration};

use bevy::{color::palettes::css::WHITE, input_focus::InputFocus, prelude::*, tasks::{block_on, futures_lite::future, Task}};
use common::shared::ConnectionResponse;
use lightyear::{netcode::{ConnectToken, NetcodeClient}, prelude::{client::NetcodeConfig, *}};

use crate::{
    transient::Transient, 
    ui::{comps::menu_button, screens::ConnectionTask}, 
    AppState
};


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
    commands.insert_resource(WaitingState::ConnectToken);
}

#[derive(Message, Clone)]
struct WaitingMessage(Option<&'static str>);

#[derive(Resource)]
enum WaitingState {
    ConnectToken,
    Connecting,
}

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
    connection_task: ResMut<ConnectionTask>,
    waiting_state: ResMut<WaitingState>,
) {

    match *waiting_state {
        WaitingState::ConnectToken => connect_token_wait(&mut commands, connection_task, waiting_state),
        WaitingState::Connecting => (),
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

fn connect_token_wait(
    commands: &mut Commands,
    mut connection_task: ResMut<ConnectionTask>,
    mut waiting_state: ResMut<WaitingState>,
) {
    if let Some(connection_response) = block_on(future::poll_once(&mut connection_task.0)) {
        match connection_response {
            Some(connection_data) => {
                info!("Got connection response: {connection_data:?}");
                let Ok(connection_token) = ConnectToken::try_from_bytes(&connection_data.connect_token) else {

                    commands.spawn(crate::ui::comps::error_popup("Server connect token invalid".into()));
                    commands.set_state(AppState::Menu);
                    return;
                };
                let client_addr = SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), 0);
                let auth = Authentication::Token(connection_token);
                let client = commands.spawn((
                    Client::default(),
                    LocalAddr(client_addr),
                    PeerAddr(connection_data.server_addr),
                    Link::new(None),
                    ReplicationReceiver::default(),
                    NetcodeClient::new(auth, NetcodeConfig::default()).unwrap(),
                    UdpIo::default(),
                )).id();

                commands.trigger(Connect { entity: client});

                *waiting_state = WaitingState::Connecting;
            }
            None => {
                commands.spawn(crate::ui::comps::error_popup("Could not connect to server".into()));
                commands.set_state(AppState::Menu);
            }
        }
    }
}

pub(crate) fn handle_connected(
    trigger: On<Add, Connected>,
    query: Query<&RemoteId, With<Client>>,
    mut commands: Commands,
) {
    let Ok(client_id) = query.get(trigger.entity) else {
        error!("What");
        return;
    };
    let client_id = client_id.0;
    /*let entity = commands
        .spawn((
            PlayerBundle::new(client_id, Vec2::ZERO),
            // we replicate the Player entity to all clients that are connected to this server
            Replicate::to_clients(NetworkTarget::All),
        ))
        .id();
    */
    info!(
        "Connected",
    );

    commands.set_state(AppState::Lobby);
}

fn exit_waiting_screen(mut commands: Commands) {
    commands.remove_resource::<ConnectionTask>();
    commands.remove_resource::<WaitingState>();
    commands.remove_resource::<LoadingTimer>();
}

pub(crate) struct WaitingScreenPlugin;

impl Plugin for WaitingScreenPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_message::<WaitingMessage>()
            .insert_resource(InputFocus(None))
            .add_systems(OnEnter(AppState::Waiting), enter_waiting_screen)
            .add_observer(handle_connected)
            .add_systems(Update, (waiting_text_anim, waiting_handler).run_if(in_state(AppState::Waiting)))
            .add_systems(OnExit(AppState::Waiting), exit_waiting_screen)
            ;
    }
}

