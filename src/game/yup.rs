use avian2d::prelude::*;
use bevy::prelude::*;

use crate::screens::ingame::playing::{MovementSpeed, Obstacle};

#[derive(Component, Debug)]
pub struct Yup;

pub fn plugin(app: &mut App) {
    app.add_systems(PostProcessCollisions, collision_handler);
}

fn collision_handler(
    mut collisions: ResMut<Collisions>,
    obstacles: Query<Entity, With<Obstacle>>,
    mut yups: Query<(Entity, &mut MovementSpeed), With<Yup>>,
) {
    // For now, we'll ignore all collisions between yups.
    // TODO: sometimes blockers will need to be collided with!
    collisions.retain(|c| yups.get(c.entity1).is_err() || yups.get(c.entity2).is_err());

    for (yup, mut speed) in &mut yups {
        for obstacle in &obstacles {
            if collisions.contains(yup, obstacle) {
                *speed = MovementSpeed(-speed.0);
            }
        }
    }
}
