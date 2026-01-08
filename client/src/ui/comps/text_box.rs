use bevy::{color::palettes::{css::WHITE, tailwind::BLUE_200}, input::{keyboard::KeyboardInput, ButtonState}, input_focus::{FocusedInput, InputFocus}, prelude::*};
use crate::{constants::{FULL, MENU_BUTTON_HEIGHT, MENU_BUTTON_WIDTH}, observe::observe, ui::mixins};

#[derive(Component)]
pub struct TextInput;

#[derive(EntityEvent)]
#[entity_event(propagate, auto_propagate)]
pub struct TextChanged {
    pub entity: Entity,
    pub text: String,
}

pub fn text_box(label: impl Component, text: String) -> impl Bundle {
    (
        Node {
            border: UiRect::all(Val::Px(2.0)),
            padding: UiRect::all(Val::Px(4.0)),
            width: MENU_BUTTON_WIDTH,
            height: MENU_BUTTON_HEIGHT,
            ..Default::default()
        },
        mixins::hover_border_color(WHITE.into(), BLUE_200.into()),
        children![
            (
                TextInput,
                Text::new(text),
                Node {
                    width: FULL,
                    height: FULL,
                    ..Default::default()
                },
                observe(move |trigger: On<Pointer<Click>>, mut input_focus: ResMut<InputFocus>| {
                    input_focus.set(trigger.entity);
                }),
                observe(
                    move |
                        trigger: On<FocusedInput<KeyboardInput>>, 
                        mut query: Query<&mut Text>,
                        mut commands: Commands,
                    | {
                    let Ok(mut text) = query.get_mut(trigger.focused_entity) else {
                        return;
                    };
                    if trigger.input.state == ButtonState::Released {
                        return;
                    }
                    if trigger.input.key_code == KeyCode::Backspace {
                        if !text.0.is_empty() {
                            text.0.pop();
                        }
                        return;
                    }

                    let Some(ref key_text) = trigger.input.text else {
                        return;
                    };

                    if key_text.is_ascii() {
                        text.0 += key_text;
                    }

                    commands.trigger(TextChanged {
                        entity: trigger.focused_entity,
                        text: text.0.clone(),
                    })
                }),
                label
            )
        ]
    )
}
