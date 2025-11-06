use bevy::{color::palettes::css::WHITE, prelude::*};
pub const DARK_GREEN: Srgba = Srgba {
    red:0.204,
    blue: 0.306,
    green: 0.255,
    alpha: 1.0
};

pub const FOREST_GREEN: Srgba = Srgba {
    red: 0.227,
    green: 0.353,
    blue: 0.251,
    alpha: 1.0
};

pub const PRIMARY_GREEN: Srgba = Srgba {
    red: 0.345,
    green: 0.506,
    blue: 0.341,
    alpha: 1.0
};
pub const PASTEL_GREEN: Srgba = Srgba {
    red: 0.639,
    green: 0.694,
    blue: 0.541,
    alpha: 1.0,
};
pub const BEIGE: Srgba = Srgba {
    red: 0.855,
    green: 0.843,
    blue: 0.804 ,
    alpha: 1.0
};

pub const HOVERED_BUTTON: Srgba = FOREST_GREEN;
pub const HOVERED_TEXT: Srgba = BEIGE;
pub const PRIMARY_BUTTON: Srgba = PRIMARY_GREEN;
pub const PRIMARY_TEXT: Srgba = WHITE;

pub const MENU_BUTTON_WIDTH: Val = Val::Px(200.0);
pub const MENU_BUTTON_HEIGHT: Val = Val::Px(40.0);
pub const FULL: Val = Val::Percent(100.0);

pub fn full_flex() -> Node {
    Node {
        flex_direction: FlexDirection::Row,
        display: Display::Flex,
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        column_gap: Val::Px(8.0),
        width: Val::Vw(100.0),
        height: Val::Vh(100.0),
        ..Default::default()
    }
}

pub static BROKER_URL: &str = "localhost:3003";
pub const ERROR_Z_INDEX: ZIndex = ZIndex(1000);
