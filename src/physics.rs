pub mod collision;

use bevy::prelude::*;
use collision::CollisionPlugin;

use crate::game::yup::CharacterState;

pub fn plugin(app: &mut App) {
    app.add_plugins(CollisionPlugin);
    app.add_systems(FixedUpdate, gravity);
}

#[derive(Component, Debug, Default)]
pub struct Gravity;

fn gravity(mut has_gravity: Query<(&CharacterState, &mut Transform), With<Gravity>>) {
    for (state, mut t) in &mut has_gravity {
        if *state == CharacterState::Falling {
            // TODO: delta time
            t.translation.y -= 3.0;
        }

        if *state == CharacterState::Walking {
            t.translation.x += 2.0;
        }
    }
}
