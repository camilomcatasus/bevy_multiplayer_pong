use std::f32::consts::PI;

use bevy::{input::keyboard::Key, prelude::*};
use common::protocol::{handle_input, Animation, Direction, Inputs, PlayerId, PlayerIndex, PlayerState};
use lightyear::{connection::direction, input::client::InputSystems, prelude::{input::native::{ActionState, InputMarker}, Predicted, Tick}};
use crate::{
    custom_models::{
        Ball, Collidable, GameEvent, GameSettings, Player
    }, AppState, 
};

pub const INPUT_UP: u8 = 1 << 0;
pub const INPUT_DOWN: u8 = 1 << 1;
pub const INPUT_SPECIAL: u8 = 1 << 2;
pub const INPUT_ATTACK: u8 = 1 << 3;

const PLAYER_SPEED: f64 = 0.02f64;
const INPUT_BUFFERING_TIME: u16 = 20u16;


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


pub fn player_input(
    mut inputs: Single<&mut ActionState<Inputs>, With<InputMarker<Inputs>>>,
    key_press: Res<ButtonInput<KeyCode>>,
) {
    let mut direction_vector = 0;
    if key_press.any_pressed([KeyCode::ArrowUp, KeyCode::KeyW]) {
        direction_vector += 1;
    }
    if key_press.any_pressed([KeyCode::ArrowDown, KeyCode::KeyS]) {
        direction_vector -= 1;
    }

    let direction = match direction_vector {
        -1 => Some(Direction::Down),
        1 => Some(Direction::Up),
        _ => None, 
    };

    inputs.0 = Inputs {
        direction,
        special: None,
    };
}

pub fn game_start(
    my_player: Single<Entity, With<Predicted>>,
    mut commands: Commands,
) {
    commands.entity(*my_player).insert(
        InputMarker::<Inputs>::default()
    );
}

pub fn player_movement(
    players: Query<&PlayerId>,
    my_player: Single<(&PlayerIndex, &ActionState<Inputs>, &mut Transform, &mut PlayerState), With<Predicted>>
) {
    let (player_index, inputs, transform, player_state) = my_player.into_inner();
    let player_count = players.iter().count();
    handle_input(
        player_count as f32,
        transform, 
        player_state,
        player_index, 
        inputs, 
    );
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedPreUpdate, 
            player_input.in_set(InputSystems::WriteClientInputs));
        app.add_systems(OnEnter(AppState::Game), game_start);
        app.add_systems(FixedUpdate, player_movement);
    }
}
