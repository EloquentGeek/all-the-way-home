use bevy::prelude::*;

use crate::screens::ingame::playing::MovementSpeed;

pub fn plugin(app: &mut App) {
    app.add_systems(FixedUpdate, movement);
}

fn movement(mut moving_objects: Query<(&MovementSpeed)>) {
    // for (mut lv, speed) in &mut moving_objects {
    //     lv.x = speed.0;
    // }
}
