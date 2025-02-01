pub mod collision;

use bevy::prelude::*;
use collision::CollisionPlugin;

use crate::game::yup::CharacterState;

pub fn plugin(app: &mut App) {
    app.add_plugins(CollisionPlugin);
    app.add_systems(FixedUpdate, gravity);
}

#[derive(Component, Debug)]
pub struct Gravity;

fn gravity(mut has_gravity: Query<(&CharacterState, &mut Transform), With<Gravity>>) {
    for (state, mut t) in &mut has_gravity {
        if *state == CharacterState::Falling {
            // TODO: delta time
            t.translation.y -= 0.5;
        }
    }
}
