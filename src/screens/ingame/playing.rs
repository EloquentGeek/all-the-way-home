use bevy::{
    prelude::*,
    render::{
        render_resource::{AsBindGroup, ShaderRef, TextureUsages},
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
    app.init_resource::<TerrainRenderTarget>();
    app.add_systems(Update, draw_alpha_gpu.run_if(in_state(Screen::InGame)));
    app.add_plugins(Material2dPlugin::<LevelMaterial>::default());
}

#[derive(Resource, Default)]
pub struct TerrainRenderTarget {
    pub texture: Handle<Image>,
}

#[derive(Component, Debug)]
pub struct Level;

#[derive(Component, Debug)]
pub struct Obstacle;

#[derive(Component)]
pub struct MovementSpeed(pub f32);

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
        RenderLayers::from_layers(&[0, 1]),
        StateScoped(Screen::InGame),
    ));

    // Render blended GPU result (terrain plus mask) to render target for collision detection
    let terrain_texture = r!(images.get(&textures.level));
    let mut render_texture = terrain_texture.clone();
    render_texture.texture_descriptor.usage = TextureUsages::COPY_DST
        | TextureUsages::COPY_SRC
        | TextureUsages::TEXTURE_BINDING
        | TextureUsages::RENDER_ATTACHMENT;

    let texture_handle = images.add(render_texture);
    commands.insert_resource(TerrainRenderTarget {
        texture: texture_handle.clone(),
    });

    commands.spawn((
        Name::new("RenderTarget Camera"),
        Camera2d,
        Camera {
            target: texture_handle.clone().into(),
            clear_color: Color::WHITE.into(),
            ..default()
        },
        RenderLayers::layer(1),
        StateScoped(Screen::InGame),
    ));

    // Debug image
    commands.spawn((
        Name::new("Debug Terrain RenderTarget"),
        Sprite {
            image: texture_handle.clone(),
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
