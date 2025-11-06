use bevy::prelude::*;
use crate::observe::observe;

pub fn mixin(base_color: Color, hover_colour: Color) -> impl Bundle {
    (
        BorderColor::all(base_color),
        observe(hover_start(hover_colour)),
        observe(hover_end(base_color)),
        observe(touch_end(base_color)),
    )
}

fn hover_start(
    hover_colour: Color,
) -> impl Fn(On<Pointer<Over>>, Query<&mut BorderColor>) {
    move |trigger, mut border_color| {
        let Ok(mut border_color) = border_color.get_mut(trigger.entity) else {
            return;
        };

        border_color.set_all(hover_colour);
    }
}

fn hover_end(base_colour: Color) -> impl Fn(On<Pointer<Out>>, Query<&mut BorderColor>) {
    move |trigger, mut border_color| {
        let Ok(mut border_color) = border_color.get_mut(trigger.entity) else {
            return;
        };

        border_color.set_all(base_colour);
    }
}

fn touch_end(base_colour: Color) -> impl Fn(Trigger<Pointer<Click>>, Query<&mut BorderColor>) {
    move |trigger, mut border_color| {
        let Ok(mut border_color) = border_color.get_mut(trigger.entity) else {
            return;
        };

        border_color.set_all(base_colour);
    }
}
