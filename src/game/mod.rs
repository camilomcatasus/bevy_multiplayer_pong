use std::f32::consts::PI;

use bevy::{prelude::*, utils::hashbrown::HashMap,render::camera::ScalingMode};
use bevy_ggrs::{AddRollbackCommandExtension, LocalInputs, LocalPlayers, PlayerInputs};
use crate::{custom_models::{Ball, Collidable, GameEvent, GameSettings, Player}, Config};

pub const INPUT_UP: u8 = 1 << 0;
pub const INPUT_DOWN: u8 = 1 << 1;
pub const INPUT_SPECIAL: u8 = 1 << 2;
pub const INPUT_ATTACK: u8 = 1 << 3;

pub mod levels;

pub fn read_local_inputs(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    local_players: Res<LocalPlayers>
) {
    let mut local_inputs = HashMap::new();
    for handle in  &local_players.0 {
        let mut input = 0u8;

        if keys.any_pressed([KeyCode::ArrowUp, KeyCode::KeyW]) {
            input |= INPUT_UP;
        }

        if keys.any_pressed([KeyCode::ArrowDown, KeyCode::KeyS]) {
            input |= INPUT_DOWN;
        }

        if keys.any_pressed([KeyCode::Space, KeyCode::Enter]) {
            input |= INPUT_SPECIAL;
        }

        local_inputs.insert(*handle, input);
    }

    commands.insert_resource(LocalInputs::<Config>(local_inputs));
}

pub fn move_player(
    mut players: Query<(&mut Transform, &Player)>,
    inputs: Res<PlayerInputs<Config>>,
    time: Res<Time>,
) {
    for (mut transform, player) in &mut players {
        let mut scalar = 0f32;
        let (input, _) = inputs[player.handle];

        if input & INPUT_UP != 0 {
            scalar = 1f32;
        }
        if input & INPUT_DOWN != 0 {
            scalar = -1f32;
        }

        let move_delta = transform.up() * scalar * player.speed * time.delta_secs();
        transform.translation += move_delta;
    }
}

pub fn move_ball(
    time: Res<Time>,
    colliders: Query<(&Collidable, Entity)>,
    mut balls: Query<(&mut Transform, &mut Ball, &Sprite, Entity), Without<Player>>,
    mut commands: Commands,
    mut event_writer: EventWriter<GameEvent>
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
    mut event_reader: EventReader<GameEvent>,
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
        Transform::default(),
    )).add_rollback();
}

fn game_setup(
    mut commands: Commands,
) {
    commands.spawn((
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: 20.,
            },
            ..OrthographicProjection::default_2d()
        })
    ));

    levels::load_level("./levels/test.json", &mut commands, &Vec::new());
}
