use core::f32;
use std::f32::consts::PI;
use std::time::Duration;

use bevy::app::App;
use bevy::math::ops::{cos, sin};
use bevy::{ecs::entity::MapEntities, prelude::*};
use lightyear::input::native::plugin::InputPlugin;
use lightyear::{core::time::TickDelta, prelude::*};
use serde::{Deserialize, Serialize};

pub const FIXED_TIMESTEP: f64 = 64.0;
pub const SEND_INTERVAL: Duration = Duration::from_millis(100);
pub struct MainChannel;

#[derive(Component, Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct PlayerIndex(pub u16);
pub const RADIUS: f32 = 7f32;
pub const PADDLE_WIDTH: f32 = 5.0;

pub fn handle_input(
    mut player_state: Mut<PlayerState>,
    inputs: &Inputs) {
    match inputs.direction {
        None => (),
        Some(Direction::Up) => {
            player_state.position += 0.04;
            player_state.position = player_state.position.min(0.8);
        },
        Some(Direction::Down) => {
            player_state.position -= 0.04;
            player_state.position = player_state.position.max(-0.8);
        },
    }
}

pub fn transform_player(player_transform: &mut Transform, player_state: &PlayerState, player_index: &PlayerIndex, player_count: f32) {
    let theta = f32::from(player_index.0) / player_count * 2.0 * PI;
    let theta_slice = PI / player_count;
    let offset_theta = theta - theta_slice * player_state.position;
    player_transform.rotation = Quat::from_axis_angle(vec3(0.0, 0.0, 1.0), offset_theta);
    player_transform.translation = vec3(cos(offset_theta) * RADIUS, sin(offset_theta) * RADIUS, 0f32);
}

pub fn move_balls(
    balls: Query<&mut BallState>,
    players: Query<(&mut PlayerState, &PlayerIndex)>,
) {
    let player_count = players.iter().len() as f32;
    let theta_slice = PI / player_count;
    for mut ball in balls {
        let circle_start = ball.transform;
        let start_theta = ball.transform.to_angle() + PI - theta_slice;
        ball.move_ball();
        let end_theta = ball.transform.to_angle() + PI - theta_slice;
        for (player_state, player_index) in &players {
            let theta = f32::from(player_index.0) / player_count * 2.0 * PI;
            if  start_theta < theta - theta_slice && 
                start_theta > theta + theta_slice &&
                end_theta < theta - theta_slice &&
                end_theta > theta + theta_slice {
                continue;
            }
                
            let mut player_transform = Transform::default();
            transform_player(&mut player_transform, player_state, player_index, player_count);
            let collision_query = swept_circle_obb_collision(circle_start, ball.transform, 0.5, &player_transform);
            match collision_query {
                None => (),
                Some(collision_info)  => {
                    ball.direction = ball.direction.reflect(collision_info.normal);
                    ball.transform = player_transform.translation.xy() + collision_info.collision_point + ball.direction * collision_info.reflected_time * ball.speed;
                    info!("Collision detected: {collision_info:?}");
                    info!("Ball: {ball:?}");
                }
            }
        }

        if ball.transform.length() > RADIUS + 2.0 {
            ball.transform = Vec2::ZERO;
        }
    }
}

#[derive(Debug)]
struct CollisionInfo {
    collision_point: Vec2,
    normal: Vec2,
    reflected_time: f32,
}

fn swept_circle_obb_collision(
    circle_start: Vec2,
    circle_end: Vec2,
    radius: f32,
    rect_transform: &Transform,
) -> Option<CollisionInfo> {
    let rect_angle = rect_transform.rotation.to_axis_angle().1;
    let rotation_matrix = Vec2::from_angle(-rect_angle);
    let world_matrix = Vec2::from_angle(rect_angle);

    let delta_start = circle_start - rect_transform.translation.xy();
    let local_start = delta_start.rotate(rotation_matrix);
    let delta_end = circle_end - rect_transform.translation.xy();
    let local_end = delta_end.rotate(rotation_matrix);
    let local_collision = swept_circle_aabb(
        local_start, 
        local_end, 
        radius, 
        Vec2::new(1.0, PADDLE_WIDTH) / 2.0
    )?;

    let world_collision = CollisionInfo {
        collision_point: local_collision.collision_point.rotate(world_matrix),
        normal: local_collision.normal.rotate(world_matrix),
        reflected_time: local_collision.reflected_time,
    };

    Some(world_collision)
}

fn swept_circle_aabb(
    start: Vec2,
    end: Vec2,
    radius: f32,
    half_extents: Vec2,
) -> Option<CollisionInfo> {
    let velocity = end - start;
    let expanded_min = -half_extents - Vec2::splat(radius);
    let expanded_max = half_extents + Vec2::splat(radius);

    if start.x >= expanded_min.x && 
        start.x <= expanded_max.x && 
        start.y >= expanded_min.y && 
        start.y <= expanded_max.y {
            warn!("Should not be happening: start - {start:?}, expanded_min - {expanded_min:?}, expanded_max - {expanded_max:?}");
        return None;
    }

    let (t_min, t_max) = ray_aabb_intersection(start, velocity, expanded_min, expanded_max)?;
    if t_min > 1.0 || t_max < 0.0 {
        return None;
    }

    let t_entry = t_min.max(0.0);
    let collision_point = start + velocity * t_entry;
    Some(CollisionInfo {
        normal: get_aabb_collision_normal(
            start,
            collision_point,
            velocity,
            radius,
            expanded_min,
            expanded_max,
            half_extents
        ),
        collision_point,
        reflected_time: 1.0 - t_entry,
    })
}

fn ray_aabb_intersection(
    origin: Vec2,
    direction: Vec2,
    aabb_min: Vec2,
    aabb_max: Vec2,
) -> Option<(f32, f32)> {
    let epsilon = 1e-10;
    let mut t_min = f32::NEG_INFINITY;
    let mut t_max = f32::INFINITY;

    if direction.x.abs() > epsilon {
        let t1 = (aabb_min.x - origin.x) / direction.x;
        let t2 = (aabb_max.x - origin.x) / direction.x;
        t_min = t_min.max(t1.min(t2));
        t_max = t_max.min(t1.max(t2));
    } else if origin.x < aabb_min.x || origin.x > aabb_max.x {
        return None;
    }

    if direction.y.abs() > epsilon {
        let t1 = (aabb_min.y - origin.y) / direction.y;
        let t2 = (aabb_max.y - origin.y) / direction.y;
        t_min = t_min.max(t1.min(t2));
        t_max = t_max.min(t1.max(t2));
    } else if origin.y < aabb_min.y || origin.y > aabb_max.y {
        return None;
    }

    if t_min > t_max { return None; };

    Some((t_min, t_max))
}

fn get_aabb_collision_normal(
    start: Vec2,
    collision_point: Vec2,
    velocity: Vec2,
    radius: f32,
    expanded_min: Vec2,
    expanded_max: Vec2,
    half_extents: Vec2,
) -> Vec2 {
    let tolerance = radius * 0.1;
    let at_left = (collision_point.x - expanded_min.x).abs() < tolerance;
    let at_right = (collision_point.x - expanded_max.x).abs() < tolerance;
    let at_bottom = (collision_point.y - expanded_min.y).abs() < tolerance;
    let at_top = (collision_point.y - expanded_max.y).abs() < tolerance;

    if (at_left || at_right) && (at_bottom || at_top) {
        let corner = Vec2::new(
            if at_left { -half_extents.x } else { half_extents.x },
            if at_bottom { -half_extents.y } else { half_extents.y }
        );

        return (collision_point - corner).normalize();
    }
    else {
        if velocity.x.abs() > velocity.y.abs() {
            return Vec2::new(if velocity.x > 0.0 { -1.0 } else { 1.0 }, 0.0);
        }
        else {
            return Vec2::new(0.0, if velocity.y > 0.0 { -1.0 } else { 1.0 })
        }
    }
}

#[derive(Component, Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct PlayerId {
    pub id: PeerId,
    pub name: String,
    pub spectating: bool,
}

impl PlayerId {
    pub fn new(name: String, id: PeerId, spectating: bool) -> Self{
        Self {
            id,
            name,
            spectating
        }
    }
}

#[derive(Component, Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct PlayerLobbyInfo {
    pub ready: bool,
}

#[derive(Component, Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct PlayerState {
    pub position: f32,
    pub anim_buffer: Vec<Animation>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Animation {
    pub anim_type: Special,
    pub origin_tick: Tick
}

#[derive(Component, Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct PlayerColor(pub Color);

#[derive(Bundle)]
pub struct PlayerBundle {
    pub id: PlayerId,
    pub state: PlayerState,
    pub color: PlayerColor,
}

#[derive(Component, Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct PlayerCard(pub PeerId);

#[derive(Component, Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct PlayerListDisplay;

impl Ease for PlayerState {
    fn interpolating_curve_unbounded(start: Self, end: Self) -> impl Curve<Self> {
        FunctionCurve::new(Interval::UNIT, move |t| {
            PlayerState {
                position: f32::lerp(start.position, end.position, t),
                anim_buffer: end.anim_buffer.clone()
            }
        })
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Reflect)]
pub enum Direction {
    Up,
    Down,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Reflect)]
pub enum Special {
    Jump,
    Speed,
    Twirl,
}

impl Special {
    pub fn get_duration(&self) -> Duration {
        match self {
            Special::Jump => Duration::from_millis(500),
            Special::Speed => Duration::from_millis(2000),
            Special::Twirl => Duration::from_millis(1250),
        }
    }

    pub fn get_length_tick(&self) -> TickDelta {
        TickDelta::from_duration(self.get_duration(), Duration::from_secs_f64(1.0/64.0))
    }
}

#[derive(Component, Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct BallState {
    pub transform: Vec2,
    pub direction: Vec2,
    pub speed: f32,
}

impl BallState {
    pub fn move_ball(&mut self) {
        self.transform += self.direction * self.speed;
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum GameSpeed {
    Slow,
    Normal,
    Fast,
    Insane
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct ClientJoin {
    pub name: String,
    pub color: Color,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct ClientLeave;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct SettingsVoteMessage {
    pub max_score: usize,
    pub game_speed: GameSpeed,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct ClientReadyMessage(pub bool);

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct GameStartMessage{
    pub max_score: usize,
    pub game_speed: GameSpeed,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Default, Reflect)]
pub struct Inputs {
    pub direction: Option<Direction>,
    pub special: Option<Special>,
}

impl MapEntities for Inputs {
    fn map_entities<M: bevy::ecs::entity::EntityMapper>(&mut self, _entity_mapper: &mut M) {
        
    }
}

pub struct ProtocolPlugin;

fn ball_state_should_roll_back(this: &BallState, that: &BallState) -> bool {
    this.direction != that.direction || (this.transform - that.transform).length() >= 0.01 
}

impl Plugin for ProtocolPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputPlugin::<Inputs>::default());
        app.register_component::<PlayerId>();
        app.register_component::<PlayerState>()
            .add_prediction()
            .add_linear_interpolation();
        app.register_component::<PlayerColor>();
        app.register_component::<PlayerLobbyInfo>();
        app.register_component::<PlayerIndex>();
        app.register_component::<BallState>();


        app.add_channel::<MainChannel>(ChannelSettings { 
            mode: ChannelMode::OrderedReliable(ReliableSettings::default()), 
            ..Default::default()
        }).add_direction(NetworkDirection::Bidirectional);
        app.register_message::<ClientJoin>().add_direction(NetworkDirection::ClientToServer);
        app.register_message::<ClientLeave>().add_direction(NetworkDirection::ClientToServer);
        app.register_message::<SettingsVoteMessage>().add_direction(NetworkDirection::ClientToServer);
        app.register_message::<ClientReadyMessage>().add_direction(NetworkDirection::ClientToServer);
        app.register_message::<GameStartMessage>().add_direction(NetworkDirection::ServerToClient);
    }
}
