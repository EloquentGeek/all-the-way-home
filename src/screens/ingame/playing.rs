use bevy::{asset::RenderAssetUsages, prelude::*};

use crate::{assets::Levels, screens::Screen};

pub fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::InGame), init);
    // app.add_systems(FixedPreUpdate, set_level_mask);
    app.add_systems(Update, draw_alpha);
}

#[derive(Component, Debug)]
pub struct Level;

#[derive(Component, Debug)]
pub struct Obstacle;

#[derive(Component)]
pub struct MovementSpeed(pub f32);

pub fn init(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    textures: Res<Levels>,
) {
    let buf = images.get(&textures.level).unwrap();
    let mut img = buf.clone();
    img.asset_usage = RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD;
    let img_handle = images.add(img);

    commands.spawn((
        Name::new("Level"),
        Level,
        Mesh2d(meshes.add(Rectangle::new(1920., 1080.))),
        MeshMaterial2d(materials.add(img_handle)),
        StateScoped(Screen::InGame),
    ));
}

fn draw_alpha(
    mut images: ResMut<Assets<Image>>,
    level: Query<&MeshMaterial2d<ColorMaterial>, With<Level>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    window: Single<&Window>,
) {
    if !mouse_button.pressed(MouseButton::Left) {
        return;
    }

    let Ok(l) = level.get_single() else {
        return;
    };

    let Some(level_material) = materials.get(&l.0) else {
        return;
    };

    let Some(texture) = &level_material.texture else {
        return;
    };

    let Some(img) = images.get_mut(texture) else {
        return;
    };

    if let Some(cursor_pos) = window.physical_cursor_position() {
        for x in (cursor_pos.x as u32) - 10..(cursor_pos.x as u32) + 10 {
            for y in (cursor_pos.y as u32) - 10..(cursor_pos.y as u32) + 10 {
                if let Ok(existing_colour) = img.get_color_at(x, y) {
                    let _ = img.set_color_at(x, y, existing_colour.with_alpha(0.));
                }
            }
        }
        // TODO: This is just a workaround for an apparent regression from RRW, similar to old
        // issue https://github.com/bevyengine/bevy/issues/1161.
        materials.get_mut(&l.0);
    }
    // for x in 0..1920 {
    //     for y in 0..1080 {
    //         let _ = img.set_color_at(x, y, Color::srgba(1., 1., 1., 0.5));
    //     }
    // }
}

fn set_level_mask(
    mut images: ResMut<Assets<Image>>,
    level: Single<&MeshMaterial2d<ColorMaterial>, With<Level>>,
    materials: Res<Assets<ColorMaterial>>,
) {
    // NOTE: set alpha values by taking the fourth value of four bytes, which will be the alpha.
    // From example cpu_draw:
    //
    // let pixel_bytes = image.pixel_bytes_mut(UVec3::new(x, y, 0)).unwrap();
    // convert our f32 to u8
    // pixel_bytes[3] = (a * u8::MAX as f32) as u8;
    //
    // note that get_color_at and set_color_at also exist, marginally slower but easier to read?

    let Some(level_material) = materials.get(&level.0) else {
        return;
    };

    let Some(texture) = &level_material.texture else {
        return;
    };

    let Some(img) = images.get_mut(texture) else {
        return;
    };

    for x in 0..1920 {
        for y in 0..1080 {
            if let Ok(col) = img.get_color_at(x, y) {
                if col.alpha() > 0.5 {
                    let _ = img.set_color_at(x, y, Color::srgba(1., 1., 1., 0.5));
                }
            }
        }
    }
}
