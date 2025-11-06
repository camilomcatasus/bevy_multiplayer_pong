use std::{rc::Rc, sync::Arc};

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component)]
pub struct Player {
    pub speed: f32,
    pub handle: usize,
    pub score: u16,
}

#[derive(Component)]
pub struct Ball {
    pub speed: f32,
    pub direction: Vec3,
    pub col_timer: f32,
    pub collision_targets: Vec<(Entity, f32)>,
    pub last_hit: Option<Entity>
}

#[derive(Resource, Clone, Copy)]
pub struct GameSettings {
    pub max_score: u16,
    pub seed: u64,
}

#[derive(Component, Serialize, Deserialize, Clone, Debug)]
pub struct Collidable {
    pub collider_shape: ColliderShape,
    pub collision_action: CollisionAction,
}

#[derive(Message)]
pub enum GameEvent {
    BallOutOfBounds(Option<Entity>),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum CollisionAction {
    Bounce,
    Remove,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ColliderShape{
    Line {
        point: Vec2,
        slope: Vec2,
    },
    LineSegment {
        a: Vec2,
        b: Vec2,
        v2: Vec2,
    },
    Box,
}

fn cross_2d(v1: &Vec2, v2: &Vec2) -> f32 {
    return v1.x * v2.y - v1.y * v2.x;
}

impl Ball {
    pub fn will_collide(&mut self, position: &Vec2, collidable: &Collidable) -> Option<f32> {
        match collidable.collider_shape {
           ColliderShape::Box => None,
           ColliderShape::Line{ point, slope } => {
               let d = self.direction.xy();

               let point_hit_den = cross_2d(&d, &slope);
               let time_to_hit_den = cross_2d(&slope, &d);
               if point_hit_den == 0.0 || time_to_hit_den == 0.0 { return None; }
               let time_to_hit = cross_2d(&position, &slope) * cross_2d(&slope, &point) / time_to_hit_den;
               if time_to_hit > 0f32 {
                   return Some(time_to_hit / self.speed);
               }
               None
        
           },
           ColliderShape::LineSegment { a, b: _, v2 } => {
               let v1 = position - a;
               let v3 = Vec2::new(-self.direction.y, self.direction.x);
               let time_to_hit = cross_2d(&v2, &v1) / v2.dot(v3) / self.speed;
               let point_hit = v1.dot(v3) / v2.dot(v3);
               if time_to_hit > 0f32 && 
                    point_hit > 0f32 && 
                    point_hit < 1f32 {
                   return Some(time_to_hit);
               }
               
               None
           }
        }
    }

    pub fn get_reflected_direction(&mut self, collidable: &Collidable) -> Vec2 {
        match collidable.collider_shape {
            ColliderShape::Line { point: _, slope } => {
                return get_reflection(&self.direction.xy(), &slope);
            },
            ColliderShape::LineSegment { v2, .. } => {
                return get_reflection(&self.direction.xy(), &v2);
            },
            ColliderShape::Box => {
                todo!("Have not implemented box collision")
            }
        }
    }

    pub fn handle_collision(
        &mut self, 
        transform: &mut Transform, 
        collidable: &Collidable, 
        time_diff: f32,
        commands: &mut Commands,
        ball_entity: Entity,
        event_writer: &mut MessageWriter<GameEvent>,
    ) {
        match collidable.collision_action {
            CollisionAction::Bounce => {
                let reflected_direction = self.get_reflected_direction(collidable);
                transform.translation -= self.direction * self.speed * time_diff;
                self.direction = reflected_direction.extend(0f32);
                transform.translation += self.direction * self.speed * time_diff;
            },
            CollisionAction::Remove => {
                if let Ok(mut ball_commands) = commands.get_entity(ball_entity) {
                    event_writer.write(GameEvent::BallOutOfBounds(self.last_hit));
                    ball_commands.despawn();
                }
            }
        }
    }
}

fn get_reflection(direction: &Vec2, line_slope: &Vec2) -> Vec2 {
    let norm = Vec2::new(line_slope.y, -line_slope.x).normalize(); 
    let rev_norm = -norm;
    let norm_dist: f32 = (norm - direction.xy()).length();
    let rev_norm_dist: f32 = (rev_norm - direction.xy()).length();

    let true_norm = match norm_dist < rev_norm_dist {
        true => norm,
        false => rev_norm
    };
    
    return direction.xy() - 2f32 * (direction.xy().dot( true_norm ) * true_norm);
}
