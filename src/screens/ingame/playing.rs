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
    MainCamera,
    assets::{Levels, Masks},
    game::minimap::{LevelCamera, LevelRenderTarget},
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
    #[texture(1)]
    #[sampler(2)]
    pub terrain_texture: Handle<Image>,
    #[texture(3)]
    #[sampler(4)]
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
    mut images: ResMut<Assets<Image>>,
    mut level_target: ResMut<LevelRenderTarget>,
    masks: Res<Masks>,
    mut materials: ResMut<Assets<LevelMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    textures: Res<Levels>,
    window: Single<&Window>,
) {
    let cursor_position = r!(window.physical_cursor_position());
    let base_level_image = r!(images.get_mut(&textures.level.clone()));
    let mut target1 = base_level_image.clone();
    target1.texture_descriptor.usage =
        TextureUsages::COPY_DST | TextureUsages::TEXTURE_BINDING | TextureUsages::RENDER_ATTACHMENT;
    let target2 = target1.clone();
    let handle1 = images.add(target1);
    level_target.texture = images.add(target2);

    commands.spawn((
        Name::new("Level"),
        Level,
        Mesh2d(meshes.add(Rectangle::new(2560., 1440.))),
        MeshMaterial2d(materials.add(LevelMaterial {
            cursor_position,
            mask_texture: masks.cursor.clone(),
            terrain_texture: handle1.clone(),
        })),
        RenderLayers::layer(1),
        StateScoped(Screen::InGame),
    ));

    commands.spawn((
        Name::new("Level Camera"),
        LevelCamera,
        Camera2d,
        Camera {
            // Render this first.
            order: -1,
            target: level_target.texture.clone().into(),
            ..default()
        },
        // Only the level background lives on render layer 1, everything else is rendered normally
        // including sprites, etc.
        RenderLayers::layer(1),
        StateScoped(Screen::InGame),
    ));
}

fn draw_alpha_gpu(
    camera: Single<(&Camera, &GlobalTransform), With<MainCamera>>,
    level: Query<(&MeshMaterial2d<LevelMaterial>, &Transform), With<Level>>,
    mut materials: ResMut<Assets<LevelMaterial>>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    window: Single<&Window>,
) {
    if !mouse_button.pressed(MouseButton::Left) {
        return;
    }

    let (cam, cam_transform) = *camera;
    let (material_handle, material_transform) = r!(level.get_single());
    let level_material = r!(materials.get_mut(&material_handle.0));
    if let Some(cursor_pos) = window.cursor_position() {
        // Convert the cursor pos to world coords. So, for the centre of the window, (640, 360)
        // will become (0, 0). Note that this flips the y value in Bevy, so we'll need to flip it
        // again later. This step should allow us to scroll the image and still get a reliable
        // cursor position.
        let world_pos = r!(cam.viewport_to_world_2d(cam_transform, cursor_pos));

        // Convert the world pos to pixel coords within the mesh texture.
        let texture_pos = material_transform
            .compute_matrix()
            .inverse()
            .transform_point3(world_pos.extend(0.));

        // This final step is necessary to offset the position passed to the shader by the window
        // dimensions. Without it, we'll get a "ghost image" showing in the bottom right corner of
        // the window when the shader draws to the centre of it! We also invert the y value again.
        // TODO: magic numbers.
        level_material.cursor_position = Vec2::new(texture_pos.x + 1280., -texture_pos.y + 720.);
    }
}
