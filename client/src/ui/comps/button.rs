use bevy::{color::palettes::css::WHITE, ecs::event::EntityEvent};
use bevy::prelude::*;
use crate::constants::{MENU_BUTTON_HEIGHT, MENU_BUTTON_WIDTH};
use crate::{
    constants::{HOVERED_BUTTON, PRIMARY_BUTTON}, 
    ui::mixins,
    observe::observe
};
pub fn menu_button<M>(
    text: &'static str,
    message: M,
) -> impl Bundle
where 
    M: Message + Clone,
{
    (
        Node {
            display: Display::Flex,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            padding: UiRect::all(Val::Px(4f32)),
            width: MENU_BUTTON_WIDTH, 
            height: MENU_BUTTON_HEIGHT,
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
        observe(move |_: On<Pointer<Click>>, mut message_writer: MessageWriter<M>| {
            message_writer.write(message.clone());
        })
    )
}
