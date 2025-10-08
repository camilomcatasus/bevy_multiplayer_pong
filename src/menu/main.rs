
use bevy::{color::palettes::css::WHITE, prelude::*};
use crate::constants;
pub fn on_enter(mut commands: Commands) {
    commands.spawn(Node{
        display: Display::Flex,
        flex_direction: FlexDirection::Column,
        width: Val::Vw(100.0),
        height: Val::Vh(100.0),
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        ..Default::default()
    })
    .with_children(|parent| {
        parent.spawn( 
            button("Quick Match", MainButtonType::Quick)
        );
        parent.spawn(
            button("Join Match", MainButtonType::Join)
        );
        parent.spawn(
            button("Create Match", MainButtonType::Create)
        );
        parent.spawn(
            button("Quit", MainButtonType::Quit)
        );
    });
}

pub fn ui(mut commands: Commands) {
}



fn button_system(
    query: Query<(
        &mut Button,
        &mut BackgroundColor,
        &mut Interaction,
        Entity
    )>
) {

}

#[derive(Component)]
enum MainButtonType {
    Quick,
    Join,
    Create,
    Quit
}

fn button(text: &str, button_type: impl Component) -> impl Bundle {
    (
        Button,
        Node {
            display: Display::Flex,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            padding: UiRect::all(Val::Px(4f32)),
            ..Default::default()
        },
        BackgroundColor(constants::PRIMARY_GREEN),
        BorderColor::all(WHITE),
        children![
            Text::new(text)
        ]
    )
}

pub fn on_exit() {

}
