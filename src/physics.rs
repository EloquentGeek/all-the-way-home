use bevy::prelude::*;

use crate::{
    game::yup::{CharacterState, Yup},
    screens::ingame::playing::Level,
};

pub fn plugin(app: &mut App) {
    app.add_systems(FixedUpdate, (gravity, collision).chain());
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

fn collision(
    camera: Single<(&Camera, &GlobalTransform), With<Camera2d>>,
    images: Res<Assets<Image>>,
    level: Query<&MeshMaterial2d<ColorMaterial>, With<Level>>,
    materials: Res<Assets<ColorMaterial>>,
    mut yups: Query<(&mut CharacterState, &Transform), With<Yup>>,
) {
    let Ok(l) = level.get_single() else {
        return;
    };

    let Some(level_material) = materials.get(&l.0) else {
        return;
    };

    let Some(texture) = &level_material.texture else {
        return;
    };

    let Some(img) = images.get(texture) else {
        return;
    };

    let (camera, camera_transform) = camera.into_inner();
    for (mut state, t) in &mut yups {
        // TODO: better way to determine this?
        let feet = t.translation + Vec3::new(0., -18., 0.);
        if let Ok(collision_point) = camera.world_to_viewport(camera_transform, feet) {
            let Ok(check) = img.get_color_at(collision_point.x as u32, collision_point.y as u32)
            else {
                return;
            };
            if check.alpha() > 0. {
                // Hey, we collided with something!
                *state = CharacterState::Walking;
            }
        }
    }
}
