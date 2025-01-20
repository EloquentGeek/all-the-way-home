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
    app.init_resource::<MinimapRenderTarget>();
    app.add_systems(OnEnter(Screen::InGame), init);
}

#[derive(Component)]
pub struct MinimapCamera;

#[derive(Resource, Default)]
pub struct MinimapRenderTarget {
    pub texture: Handle<Image>,
}

fn get_minimap_transform(image_size: &Vec2, screen_size: &Vec2, scale_factor: f32) -> Transform {
    let actual_size = image_size / 2. * scale_factor;
    Transform::from_xyz(
        20. - (screen_size.x / 2.) + actual_size.x,
        // TODO: fix
        -150. + (screen_size.y / 2.) + actual_size.y,
        0.,
    )
    .with_scale(Vec3::splat(scale_factor))
}

fn init(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut minimap: ResMut<MinimapRenderTarget>,
    window: Single<&Window>,
) {
    // Render to image for minimap.
    let mut minimap_image = Image::new_fill(
        Extent3d {
            width: 2560,
            height: 1440,
            ..default()
        },
        TextureDimension::D2,
        &[0, 0, 0, 0],
        TextureFormat::Bgra8UnormSrgb,
        RenderAssetUsages::default(),
    );
    // TODO: feels like we need DST but not SRC here? Find out for sure. This even seems to work
    // without COPY_DST. Ask Discord?
    minimap_image.texture_descriptor.usage =
        TextureUsages::COPY_DST | TextureUsages::TEXTURE_BINDING | TextureUsages::RENDER_ATTACHMENT;
    minimap.texture = images.add(minimap_image);

    commands.spawn((
        Name::new("Minimap Camera"),
        MinimapCamera,
        Camera2d,
        Camera {
            clear_color: Color::WHITE.into(),
            // Render this first.
            order: -1,
            target: minimap.texture.clone().into(),
            ..default()
        },
        StateScoped(Screen::InGame),
    ));

    commands.spawn((
        Name::new("Minimap"),
        RenderLayers::layer(1),
        Sprite {
            image: minimap.texture.clone(),
            ..Default::default()
        },
        StateScoped(Screen::InGame),
        // TODO: magic numbers
        get_minimap_transform(&Vec2::new(2560., 1440.), &window.size(), 0.1),
    ));
}
