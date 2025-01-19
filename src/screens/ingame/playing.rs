use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::{
        render_resource::{
            AsBindGroup, Extent3d, ShaderRef, TextureDimension, TextureFormat, TextureUsages,
        },
        view::RenderLayers,
    },
    sprite::{Material2d, Material2dPlugin},
};
use tiny_bail::prelude::*;

use crate::{
    assets::{Levels, Masks},
    screens::Screen,
};

const SHADER_ASSET_PATH: &str = "shaders/mouse_shader.wgsl";

pub fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::InGame), init);
    app.init_resource::<MinimapRenderTarget>();
    app.add_systems(Update, draw_alpha_gpu.run_if(in_state(Screen::InGame)));
    app.add_plugins(Material2dPlugin::<LevelMaterial>::default());
}

#[derive(Resource, Default)]
pub struct MinimapRenderTarget {
    pub texture: Handle<Image>,
}

#[derive(Component, Debug)]
pub struct Level;

#[derive(Component, Debug)]
pub struct Obstacle;

#[derive(Component)]
pub struct MovementSpeed(pub f32);

#[derive(Component)]
pub struct MinimapCamera;

#[derive(Asset, Default, TypePath, AsBindGroup, Debug, Clone)]
pub struct LevelMaterial {
    #[uniform(0)]
    pub cursor_position: Vec2,
    // TODO: find out more about samplers!
    #[texture(1)]
    #[sampler(2)]
    pub terrain_texture: Handle<Image>,
    #[texture(3)]
    #[sampler(4)]
    pub mask_texture: Handle<Image>,
}

impl Material2d for LevelMaterial {
    fn alpha_mode(&self) -> bevy::sprite::AlphaMode2d {
        bevy::sprite::AlphaMode2d::Blend
    }

    fn fragment_shader() -> ShaderRef {
        SHADER_ASSET_PATH.into()
    }
}

pub fn init(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    masks: Res<Masks>,
    mut materials: ResMut<Assets<LevelMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut minimap: ResMut<MinimapRenderTarget>,
    textures: Res<Levels>,
    window: Single<&Window>,
) {
    let cursor_position = r!(window.physical_cursor_position());

    commands.spawn((
        Name::new("Level"),
        Level,
        Mesh2d(meshes.add(Rectangle::new(1920., 1080.))),
        MeshMaterial2d(materials.add(LevelMaterial {
            cursor_position,
            mask_texture: masks.cursor.clone(),
            terrain_texture: textures.level.clone(),
        })),
        StateScoped(Screen::InGame),
    ));

    // Render to image for minimap.
    // TODO: can we do the lemmings-like thing of displaying a viewport within the larger image?
    let mut image = Image::new_fill(
        Extent3d {
            width: 1920,
            height: 1080,
            ..default()
        },
        TextureDimension::D2,
        &[0, 0, 0, 0],
        TextureFormat::Bgra8UnormSrgb,
        RenderAssetUsages::default(),
    );
    // TODO: feels like we need DST but not SRC here? Find out for sure.
    image.texture_descriptor.usage =
        TextureUsages::TEXTURE_BINDING | TextureUsages::RENDER_ATTACHMENT;
    minimap.texture = images.add(image);

    // Source camera
    commands.spawn((
        Name::new("Minimap Camera"),
        MinimapCamera,
        Camera2d,
        Camera {
            // Render this first.
            order: -1,
            target: minimap.texture.clone().into(),
            clear_color: Color::WHITE.into(),
            ..default()
        },
        StateScoped(Screen::InGame),
    ));

    // Debug image
    commands.spawn((
        Name::new("Debug Terrain RenderTarget"),
        RenderLayers::layer(1),
        Sprite {
            image: minimap.texture.clone(),
            ..Default::default()
        },
        Transform::from_xyz(-850., 450., 0.).with_scale(Vec3::splat(0.1)),
        StateScoped(Screen::InGame),
    ));
}

fn draw_alpha_gpu(
    level: Query<&MeshMaterial2d<LevelMaterial>, With<Level>>,
    mut materials: ResMut<Assets<LevelMaterial>>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    window: Single<&Window>,
) {
    if !mouse_button.pressed(MouseButton::Left) {
        return;
    }

    let l = r!(level.get_single());
    let level_material = r!(materials.get_mut(&l.0));
    if let Some(cursor_pos) = window.physical_cursor_position() {
        level_material.cursor_position = cursor_pos;
    }
}
