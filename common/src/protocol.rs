use std::time::Duration;

use bevy::app::App;
use bevy::{ecs::entity::MapEntities, prelude::*};
use lightyear::{core::time::TickDelta, prelude::*};
use serde::{Deserialize, Serialize};

const FIXED_TIMESTEP: f64 = 64.0;

#[derive(Component, Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct PlayerId {
    id: PeerId,
    name: String,
}

#[derive(Component, Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct PlayerState {
    pub position: f64,
    pub anim_buffer: Vec<Animation>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Animation {
    pub anim_type: Special,
    pub origin_tick: Tick
}

#[derive(Component, Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct PlayerColor(Color);

#[derive(Bundle)]
pub struct PlayerBundle {
    pub id: PlayerId,
    pub state: PlayerState,
    pub color: PlayerColor,
}

impl Ease for PlayerState {
    fn interpolating_curve_unbounded(start: Self, end: Self) -> impl Curve<Self> {
        FunctionCurve::new(Interval::UNIT, move |t| {
            PlayerState {
                position: f64::lerp(start.position, end.position, t.into()),
                anim_buffer: end.anim_buffer.clone()
            }
        })
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum Direction {
    Up,
    Down,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
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
pub struct SettingsVoteMessage {
    pub max_score: usize,
    pub game_speed: GameSpeed,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct ClientReadyMessage;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct GameStartMessage{
    pub start_tick: Tick,
    pub max_score: usize,
    pub game_speed: GameSpeed,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Inputs {
    pub direction: Direction,
    pub special: Option<Special>,
}

impl MapEntities for Inputs {
    fn map_entities<M: bevy::ecs::entity::EntityMapper>(&mut self, _entity_mapper: &mut M) {
        
    }
}

pub struct ProtocolPlugin;

impl Plugin for ProtocolPlugin {
    fn build(&self, app: &mut App) {
        app.register_component::<PlayerId>();
        app.register_component::<PlayerState>();
        app.register_component::<PlayerColor>();
        app.register_message::<ClientJoin>().add_direction(NetworkDirection::ClientToServer);
        app.register_message::<SettingsVoteMessage>().add_direction(NetworkDirection::ClientToServer);
        app.register_message::<ClientReadyMessage>().add_direction(NetworkDirection::ClientToServer);
        app.register_message::<GameStartMessage>().add_direction(NetworkDirection::ServerToClient);
    }
}
