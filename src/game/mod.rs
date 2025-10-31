use std::f32::consts::PI;

use bevy::prelude::*;
use lightyear::prelude::Tick;
use crate::{
    custom_models::{
        Ball, Collidable, GameEvent, GameSettings, Player
    }, 
    protocol::{Animation, Direction, Inputs, PlayerState}
};

pub const INPUT_UP: u8 = 1 << 0;
pub const INPUT_DOWN: u8 = 1 << 1;
pub const INPUT_SPECIAL: u8 = 1 << 2;
pub const INPUT_ATTACK: u8 = 1 << 3;

pub mod levels;

const PLAYER_SPEED: f64 = 0.02f64;
const INPUT_BUFFERING_TIME: u16 = 20u16;

pub fn handle_player_input(
    mut player_state: Mut<PlayerState>,
    input: &Inputs,
    tick: Tick
) {
    match input.direction {
        Direction::Up => player_state.position += PLAYER_SPEED,
        Direction::Down => player_state.position -= PLAYER_SPEED,
    }

    if let Some(special) = &input.special {
        if let Some(last_anim) = player_state.anim_buffer.last() {
            if (last_anim.origin_tick - tick).unsigned_abs() < 
                last_anim.anim_type.get_length_tick().to_i16().unsigned_abs().saturating_sub(INPUT_BUFFERING_TIME) {
            }
        }
        else {
            player_state.anim_buffer.push(Animation {
                anim_type: special.clone(),
                origin_tick: tick
            })
        }
    }
}

pub fn move_ball(
    time: Res<Time>,
    colliders: Query<(&Collidable, Entity)>,
    mut balls: Query<(&mut Transform, &mut Ball, &Sprite, Entity), Without<Player>>,
    mut commands: Commands,
    mut event_writer: MessageWriter<GameEvent>
) {
    for (mut transform, mut ball, sprite, entity) in &mut balls {
        let sprite_rect = sprite.custom_size.unwrap();

        //let ball_bounds = transform.translation.xy() + sprite_rect;
        transform.translation += ball.direction * ball.speed * time.delta_secs();

        ball.col_timer += time.delta_secs();
        while let Some(collision_target) = ball.collision_targets.last() {
            if collision_target.1 <= ball.col_timer {
                let time_diff = ball.col_timer - collision_target.1;
                if let Ok((collidable, _)) = colliders.get(collision_target.0) {
                    ball.handle_collision(&mut transform, 
                        collidable, 
                        time_diff, 
                        &mut commands, 
                        entity,
                        &mut event_writer
                    );
                }
                ball.collision_targets.pop();
            }
            else {
                break;
            }
        }

        if ball.collision_targets.is_empty() {
            ball.col_timer = 0f32;
            for (collider_data, entity) in &colliders {
                if let Some(time_to_hit) = ball.will_collide(&transform.translation.xy(), collider_data) {
                    ball.collision_targets.push((entity, time_to_hit));
                }
            }
            println!("Collider timers: {:?}", ball.collision_targets);
        }
    }
}

pub fn handle_game_events(
    mut event_reader: MessageReader<GameEvent>,
    mut players: Query<&mut Player>,
    mut commands: Commands,
    game_settings: Res<GameSettings>,
) {
    for event in event_reader.read() {
        match event {
            GameEvent::BallOutOfBounds(last_hit_option) => {
                let mut game_ended = false;
                if let Some(last_player_hit) = last_hit_option {
                    let mut player_inst = players.get_mut(*last_player_hit).unwrap();
                    player_inst.score += 1;
                    game_ended = player_inst.score > game_settings.max_score;

                }
                if !game_ended {
                    spawn_ball(&mut commands);
                }
            }
        }
    }
}

pub fn spawn_ball(
    commands: &mut Commands
) {

    let theta: f32 = rand::random_range(0.0..(PI * 2f32));
    let direction = Vec3::new(theta.cos(), theta.sin(), 0f32).normalize();

    commands.spawn((
        Ball {
            speed: 5f32,
            direction,
            col_timer: 0f32,
            collision_targets: Vec::new(),
            last_hit: None,
        },
        Sprite {
            color: Color::srgb(1., 1., 1.),
            custom_size: Some(Vec2::new(0.25, 0.25)),
            ..default()
        },
        Transform::default()
    ));
}

fn game_setup(
    mut commands: Commands,
) {
    commands.spawn((
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: bevy::camera::ScalingMode::FixedVertical {
                viewport_height: 20.,
            },
            ..OrthographicProjection::default_2d()
        })
    ));

    levels::load_level("./levels/test.json", &mut commands, &Vec::new());
}
