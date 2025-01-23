use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::{
        render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages},
        view::RenderLayers,
    },
};

use crate::screens::Screen;

pub fn plugin(app: &mut App) {
    app.init_resource::<LevelRenderTarget>();
    app.add_systems(OnEnter(Screen::InGame), init);
}

#[derive(Component)]
pub struct LevelCamera;

#[derive(Component)]
pub struct Minimap;

// The image we'll use to display the rendered output. Everything on the main game screen and in
// the minimap is rendered to this image, which is swapped (via "ping-pong buffering") each frame
// with the handle attached to LevelMaterial.
#[derive(Resource, Default)]
pub struct LevelRenderTarget {
    pub texture: Handle<Image>,
}

fn init(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut level: ResMut<LevelRenderTarget>,
) {

    // TODO: eventually, spawn a minimap image here
    // commands.spawn((
    //     Name::new("Minimap"),
    //     Minimap,
    //     RenderLayers::layer(1),
    //     Sprite {
    //         image: level.texture.clone(),
    //         ..Default::default()
    //     },
    //     StateScoped(Screen::InGame),
    //     Transform::from_scale(Vec3::splat(0.1)),
    // ));
}
