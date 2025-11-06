use bevy::{color::palettes::{css::BLACK, tailwind::RED_500}, prelude::*};

use crate::constants::{full_flex, ERROR_Z_INDEX};


#[derive(Component)]
pub struct ErrorText;
pub fn error(text: String) -> impl Bundle {
    (
        full_flex(),
        ERROR_Z_INDEX,
        Visibility::Visible,
        children![
            (
                Node {
                    border: UiRect::all(Val::Px(2.0)),
                    ..Default::default()
                },
                BorderColor::all(RED_500),
                BackgroundColor(BLACK.into()),
                children![
                    ErrorText,
                    Text::new(text),
                    TextColor(RED_500.into()),
                ]
            )
        ]
    )
}
