use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::{
        extract_resource::ExtractResource,
        render_resource::{AsBindGroup, ShaderRef, TextureFormat, TextureUsages},
        view::RenderLayers,
    },
    sprite::{Material2d, Material2dPlugin},
};
use tiny_bail::prelude::*;

use crate::{
    GameSet, MainCamera, assets::Masks, physics::collision::CollisionsTerrain, screens::Screen,
};

use super::rendering::GameRenderLayers;

const SHADER_ASSET_PATH: &str = "shaders/terrain.wgsl";

pub fn plugin(app: &mut App) {
    app.add_plugins(Material2dPlugin::<LevelMaterial>::default());
    app.add_systems(
        OnEnter(Screen::InGame),
        (init, init_compute_shader).chain().in_set(GameSet::Init),
    );
    app.add_systems(Update, update_cursor_position.in_set(GameSet::RecordInput));
    app.add_systems(
        RunFixedMainLoop,
        swap_textures
            .in_set(RunFixedMainLoopSystem::AfterFixedMainLoop)
            .run_if(in_state(Screen::InGame)),
    );
}

#[derive(Component, Debug)]
pub struct Level;

#[derive(Component)]
pub struct LevelCamera;

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

impl Material2d for LevelMaterial {
    fn alpha_mode(&self) -> bevy::sprite::AlphaMode2d {
        bevy::sprite::AlphaMode2d::Blend
    }

    fn fragment_shader() -> ShaderRef {
        SHADER_ASSET_PATH.into()
    }
}

// The image we'll use to display the rendered output. Everything on the main game screen and in
// the minimap is rendered to this image, which is swapped (via "ping-pong buffering") each frame
// with the handle attached to LevelMaterial.
#[derive(Resource, ExtractResource, Default, Clone)]
pub struct LevelRenderTargets {
    pub destination: Handle<Image>,
    pub source: Handle<Image>,
}

pub fn init(
    mut commands: Commands,
    images: ResMut<Assets<Image>>,
    level_targets: ResMut<LevelRenderTargets>,
    masks: Res<Masks>,
    mut materials: ResMut<Assets<LevelMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    window: Single<&Window>,
) {
    let cursor_position = r!(window.physical_cursor_position());
    let level_image = r!(images.get(&level_targets.source));

    commands.spawn((
        Name::new("Level"),
        Level,
        Mesh2d(meshes.add(Rectangle::new(
            level_image.size().x as f32,
            level_image.size().y as f32,
        ))),
        MeshMaterial2d(materials.add(LevelMaterial {
            cursor_position,
            mask_texture: masks.cursor.clone(),
            terrain_texture: level_targets.source.clone(),
        })),
        RenderLayers::layer(GameRenderLayers::Terrain.into()),
        StateScoped(Screen::InGame),
    ));

    commands.spawn((
        Name::new("Level Camera"),
        LevelCamera,
        Camera2d,
        Camera {
            // Render this first.
            order: -1,
            target: level_targets.destination.clone().into(),
            ..default()
        },
        // Only the level background lives on render layer 1, everything else is rendered normally
        // including sprites, etc.
        RenderLayers::layer(GameRenderLayers::Terrain.into()),
        StateScoped(Screen::InGame),
    ));
}

fn init_compute_shader(
    mut collisions_terrain: ResMut<CollisionsTerrain>,
    mut images: ResMut<Assets<Image>>,
    level_targets: ResMut<LevelRenderTargets>,
) {
    let level_image = r!(images.get(&level_targets.destination));
    let mut collisions_terrain_image = level_image.clone();
    collisions_terrain_image.asset_usage = RenderAssetUsages::RENDER_WORLD;
    collisions_terrain_image.texture_descriptor.format = TextureFormat::Rgba8Unorm;
    collisions_terrain_image.texture_descriptor.usage =
        TextureUsages::COPY_SRC | TextureUsages::STORAGE_BINDING;
    *collisions_terrain = CollisionsTerrain(images.add(collisions_terrain_image));
}

fn update_cursor_position(
    camera: Single<(&Camera, &GlobalTransform), With<MainCamera>>,
    level: Single<(&MeshMaterial2d<LevelMaterial>, &Transform), With<Level>>,
    mut materials: ResMut<Assets<LevelMaterial>>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    window: Single<&Window>,
) {
    if !mouse_button.pressed(MouseButton::Left) {
        return;
    }

    let (cam, cam_transform) = *camera;
    let (material_handle, material_transform) = *level;
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
        level_material.cursor_position = Vec2::new(
            texture_pos.x + window.width(),
            -texture_pos.y + window.height(),
        );
    }
}

fn swap_textures(
    mut cam: Single<&mut Camera, With<LevelCamera>>,
    collisions_terrain: ResMut<CollisionsTerrain>,
    mut images: ResMut<Assets<Image>>,
    level: Query<&MeshMaterial2d<LevelMaterial>, With<Level>>,
    mut materials: ResMut<Assets<LevelMaterial>>,
    mut level_targets: ResMut<LevelRenderTargets>,
) {
    let l = r!(level.get_single());
    let level_material = r!(materials.get_mut(&l.0));

    // Create a clone of the current destination render target, to use as a source of truth for
    // collision detection. This must be correctly formatted to be accepted by the compute shader.
    // TODO: performance concerns!
    let destination_image = r!(images.get(&level_targets.destination));
    let mut collisions_terrain_image = destination_image.clone();
    collisions_terrain_image.asset_usage = RenderAssetUsages::RENDER_WORLD;
    collisions_terrain_image.texture_descriptor.format = TextureFormat::Rgba8Unorm;
    collisions_terrain_image.texture_descriptor.usage =
        TextureUsages::COPY_SRC | TextureUsages::STORAGE_BINDING;
    images.insert(&collisions_terrain.0, collisions_terrain_image);

    // Swap the camera target and fragment shader source.
    level_material.terrain_texture = level_targets.destination.clone();
    cam.target = level_targets.source.clone().into();

    // Swap source and destination in the resource to prepare for next frame.
    let old_source = level_targets.source.clone();
    level_targets.source = level_targets.destination.clone();
    level_targets.destination = old_source;

    // Trigger change detection
    let _ = r!(materials.get_mut(&l.0));
}
