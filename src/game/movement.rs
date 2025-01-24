use bevy::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_systems(FixedUpdate, movement);
}

#[derive(Component)]
pub struct MovementSpeed(pub f32);

fn movement(mut moving_objects: Query<(&MovementSpeed)>) {
    // for (mut lv, speed) in &mut moving_objects {
    //     lv.x = speed.0;
    // }
}
