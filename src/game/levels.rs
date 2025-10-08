use serde::{Deserialize, Serialize};
use bevy::prelude::*;

use crate::custom_models::Collidable;


#[derive(Serialize, Deserialize)]
pub struct Level {
    pub colliders: Vec<Collidable>,
}

pub fn load_level(
    level_name: &str, 
    commands: &mut Commands, 
    colliders: &Vec<Entity>
) 
{
    for entity_id in colliders {
        commands.entity(*entity_id).despawn();
    }
    let level_data = std::fs::read_to_string(level_name).unwrap();
    let level : Level = serde_json::from_str(&level_data).unwrap();

    for collider in level.colliders {
        println!("{:?}", collider);
        commands.spawn(collider);
    }
}

pub fn save_level(level_name: &str, colliders: &Query<&Collidable>) {
    let mut col_list : Vec<Collidable> = Vec::new();
    for collider in colliders {
        col_list.push(collider.clone());
    }
    let level: Level = Level {
        colliders: col_list,
    };

    std::fs::write(level_name, serde_json::to_string_pretty(&level).unwrap());
}


