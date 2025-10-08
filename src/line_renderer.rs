use bevy::prelude::*;

use crate::custom_models::{Collidable, ColliderShape};


pub fn render_lines(
    collidables: Query<&Collidable>,
    mut draw: Gizmos
) {
    for collidable in &collidables {
        match collidable.collider_shape {
            ColliderShape::LineSegment { a, b, v2: _ } => {
                draw.line_2d(a, b, Color::srgb(1.0, 0.0, 0.0))
            },
            ColliderShape::Line { point, slope } => {
                let a = point - slope * 200f32;
                let b = point + slope * 200f32;
                draw.line_2d(a, b, Color::WHITE);
            },
            ColliderShape::Box => {
                todo!("Not supported")
            }
        }
    }
}
