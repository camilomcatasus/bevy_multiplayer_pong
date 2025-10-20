use bevy::{color::palettes::css::WHITE, prelude::*};

use crate::constants::{self, HOVERED_BUTTON, PRIMARY_BUTTON, DARK_GREEN};
pub fn on_enter(mut commands: Commands) {
    commands.spawn((
            Node{
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                width: Val::Vw(100.0),
                height: Val::Vh(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(4.0),
                ..Default::default()
            }, MainUI
    ))
    .with_children(|parent| {
        spawn_button(parent, "Quick Match", MainButtonType::Quick);
        spawn_button(parent, "Join Match", MainButtonType::Join);
        spawn_button(parent, "Create Match", MainButtonType::Create);
        spawn_button(parent, "Quit", MainButtonType::Quit);
    });
}

pub fn handle_clicks(
    query: Query<
        (&MainButtonType, &Interaction), 
        (Changed<Interaction>, With<Button>)
    >,
    commands: Commands,
    mut exit: EventWriter<AppExit>) {
    for (button_type, interaction) in &query {
        match (button_type, interaction) {
            (MainButtonType::Create, Interaction::Pressed) => (),
            (MainButtonType::Join, Interaction::Pressed) => (),
            (MainButtonType::Quick, Interaction::Pressed) => (),
            (MainButtonType::Quit, Interaction::Pressed) => {exit.send_default();},
            (_, _) => ()
        }
    }
}

fn handle_quick_join() {

}

pub fn button_system(
    mut interaction_query: Query<(
        &mut BackgroundColor,
        &mut Interaction,
        &Children,
        Entity
    ),
    (Changed<Interaction>, With<Button>)>
) {
    //TODO: Maybe change text and make colors prettier?
    for (mut background_color, interaction, _children, _entity) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                background_color.0 = DARK_GREEN.into();
            },
            Interaction::Hovered => {
                background_color.0 = HOVERED_BUTTON.into();
            },
            Interaction::None => {
                background_color.0 = PRIMARY_BUTTON.into();
            }
        }
    }
}

#[derive(Component)]
pub struct MainUI;

#[derive(Component)]
pub enum MainButtonType {
    Quick,
    Join,
    Create,
    Quit
}

fn spawn_button(cb: &mut ChildBuilder, text: &str, button_type: impl Component) {
    cb.spawn((
        Button,
        Node {
            display: Display::Flex,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            padding: UiRect::all(Val::Px(4f32)),
            width: Val::Px(320.0),
            
            
            ..Default::default()
        },
        BackgroundColor(constants::PRIMARY_BUTTON.into()),
        button_type
    )).with_child((
        Text(text.to_string()),
        TextColor(WHITE.into())
    ));
}

pub fn on_exit(query: Query<Entity, With<MainUI>>) {

}
