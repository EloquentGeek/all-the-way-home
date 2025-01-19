use bevy::prelude::*;

use crate::game::yup::CharacterState;

pub fn plugin(app: &mut App) {
    app.add_systems(FixedUpdate, gravity);
}

#[derive(Component, Debug)]
pub struct Gravity;

fn gravity(mut has_gravity: Query<(&CharacterState, &mut Transform), With<Gravity>>) {
    for (state, mut t) in &mut has_gravity {
        if *state == CharacterState::Falling {
            t.translation.y -= 3.;
        }
    }
}

// fn collision(
//     camera: Single<(&Camera, &GlobalTransform), With<MainCamera>>,
//     images: Res<Assets<Image>>,
//     terrain: Res<TerrainRenderTarget>,
//     mut yups: Query<(&mut CharacterState, &Transform), With<Yup>>,
// ) {
//     let level_texture = r!(images.get(&terrain.texture));
//
//     let (camera, camera_transform) = camera.into_inner();
//     for (mut state, t) in &mut yups {
//         // TODO: better way to determine this?
//         let feet = t.translation + Vec3::new(0., -18., 0.);
//         let collision_point = r!(camera.world_to_viewport(camera_transform, feet));
//         let pixel =
//             r!(level_texture.get_color_at(collision_point.x as u32, collision_point.y as u32));
//         if pixel.alpha() > 0. {
//             // Hey, we collided with something!
//             *state = CharacterState::Walking;
//         } else {
//             // We're still falling, OR we've STARTED falling again.
//             *state = CharacterState::Falling;
//         }
//     }
// }
