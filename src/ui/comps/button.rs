use bevy::{color::palettes::css::WHITE, ecs::event::EntityEvent};
use bevy::prelude::*;
use crate::observe;
use crate::{
    constants::{HOVERED_BUTTON, PRIMARY_BUTTON}, 
    ui::mixins,
    observe::observe
};
pub fn menu_button(
    text: &'static str,
    message: impl Message + Clone,
) -> impl Bundle {
    (
        Node {
            display: Display::Flex,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            padding: UiRect::all(Val::Px(4f32)),
            width: Val::Px(320.0),
            ..Default::default()
        },
        Button,
        mixins::hover_color(PRIMARY_BUTTON.into(), HOVERED_BUTTON.into()),
        children![
            (
                Text(text.to_string()),
                TextColor(WHITE.into())
            )
        ],
        observe(move |_: On<Pointer<Click>>, mut commands: Commands| {
            commands.write_message(message.clone());
        })
    )
}
