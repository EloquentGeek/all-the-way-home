use bevy::{
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef},
    sprite::{Material2d, Material2dPlugin},
};
use tiny_bail::prelude::*;

use crate::{
    MainCamera,
    assets::{Levels, Masks},
    game::minimap::MinimapRenderTarget,
    screens::Screen,
};

const SHADER_ASSET_PATH: &str = "shaders/mouse_shader.wgsl";

pub fn plugin(app: &mut App) {
    app.init_resource::<LevelViewport>();
    app.add_systems(OnEnter(Screen::InGame), init);
    app.add_systems(Update, draw_alpha_gpu.run_if(in_state(Screen::InGame)));
    app.add_plugins(Material2dPlugin::<LevelMaterial>::default());
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
    #[uniform(1)]
    pub level_viewport: Vec2,
    // TODO: find out more about samplers!
    #[texture(2)]
    #[sampler(3)]
    pub terrain_texture: Handle<Image>,
    #[texture(4)]
    #[sampler(5)]
    pub mask_texture: Handle<Image>,
}

#[derive(Resource, Deref, DerefMut)]
pub struct LevelViewport(Vec2);

impl Default for LevelViewport {
    fn default() -> Self {
        // Start at the centre of the 2k mesh
        Self(Vec2::new(640., 360.))
    }
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
    level_viewport: Res<LevelViewport>,
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
        Mesh2d(meshes.add(Rectangle::new(2560., 1440.))),
        MeshMaterial2d(materials.add(LevelMaterial {
            cursor_position,
            level_viewport: **level_viewport,
            mask_texture: masks.cursor.clone(),
            terrain_texture: textures.level.clone(),
        })),
        StateScoped(Screen::InGame),
    ));
}

fn draw_alpha_gpu(
    level: Query<&MeshMaterial2d<LevelMaterial>, With<Level>>,
    level_viewport: Res<LevelViewport>,
    mut materials: ResMut<Assets<LevelMaterial>>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    window: Single<&Window>,
) {
    if !mouse_button.pressed(MouseButton::Left) {
        return;
    }

    let l = r!(level.get_single());
    let level_material = r!(materials.get_mut(&l.0));
    if let Some(cursor_pos) = window.cursor_position() {
        let uv = Vec2::new(
            cursor_pos.x / window.width(),
            cursor_pos.y / window.height(),
        );
        level_material.cursor_position = uv;
        level_material.level_viewport = **level_viewport;
    }
}
