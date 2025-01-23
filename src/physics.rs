use bevy::{
    prelude::*,
    render::{
        Render, RenderApp, RenderSet,
        extract_resource::{ExtractResource, ExtractResourcePlugin},
        gpu_readback::{Readback, ReadbackComplete},
        render_asset::RenderAssets,
        render_graph::{self, RenderGraph, RenderLabel},
        render_resource::{binding_types::storage_buffer, *},
        renderer::{RenderContext, RenderDevice},
        storage::{GpuShaderStorageBuffer, ShaderStorageBuffer},
    },
};
use tiny_bail::prelude::*;

use crate::{
    game::{
        minimap::{LevelCamera, LevelRenderTarget},
        yup::CharacterState,
    },
    screens::{
        Screen,
        ingame::playing::{Level, LevelMaterial},
    },
};

const SHADER_ASSET_PATH: &str = "shaders/collision.wgsl";

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, init);
        app.add_systems(FixedUpdate, gravity);
        app.add_plugins(ExtractResourcePlugin::<CollisionsBuffer>::default());
        app.add_systems(
            FixedUpdate,
            swap_textures
                .in_set(RenderSet::PostCleanup)
                .run_if(in_state(Screen::InGame)),
        );
    }

    fn finish(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);
        render_app
            .init_resource::<CollisionsPipeline>()
            .add_systems(
                Render,
                prepare_bind_group
                    .in_set(RenderSet::PrepareBindGroups)
                    // We don't need to recreate the bind group every frame
                    .run_if(not(resource_exists::<CollisionsBufferBindGroup>)),
            );

        // Add the compute node as a top level node to the render graph
        // (this means it will only execute once per frame). See:
        // https://github.com/bevyengine/bevy/blob/main/examples/shader/gpu_readback.rs
        render_app
            .world_mut()
            .resource_mut::<RenderGraph>()
            .add_node(CollisionsNodeLabel, CollisionsNode::default());
    }
}

fn swap_textures(
    mut cam: Single<&mut Camera, With<LevelCamera>>,
    level: Query<&MeshMaterial2d<LevelMaterial>, With<Level>>,
    mut materials: ResMut<Assets<LevelMaterial>>,
    mut target: ResMut<LevelRenderTarget>,
) {
    let l = r!(level.get_single());
    let level_material = r!(materials.get_mut(&l.0));
    let old_target_texture = target.texture.clone();
    target.texture = level_material.terrain_texture.clone();
    level_material.terrain_texture = old_target_texture;
    cam.target = target.texture.clone().into();

    // Trigger change detection
    let _ = r!(materials.get_mut(&l.0));
}

#[derive(Resource, ExtractResource, Clone)]
struct CollisionsBuffer(Handle<ShaderStorageBuffer>);

#[derive(Resource)]
struct CollisionsBufferBindGroup(BindGroup);

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
struct CollisionsNodeLabel;

#[derive(Component, Debug)]
pub struct Gravity;

fn init(mut commands: Commands, mut buffers: ResMut<Assets<ShaderStorageBuffer>>) {
    // TODO: figure out magic number avoidance later!
    // TODO: can we use a dynamic-sized buffer with web builds?
    let mut collisions = ShaderStorageBuffer::from(vec![0u32; 200]);
    // TODO: do we need DST here?
    collisions.buffer_description.usage |= BufferUsages::COPY_SRC;
    let collisions = buffers.add(collisions);

    commands
        .spawn(Readback::buffer(collisions.clone()))
        .observe(|trigger: Trigger<ReadbackComplete>| {
            // This matches the type which was used to create the `ShaderStorageBuffer` above,
            // and is a convenient way to interpret the data.
            // let data: Vec<u32> = trigger.event().to_shader_type();
            // info!("Buffer {:?}", data);
        });
    // NOTE: need to make sure nothing accesses this resource before OnEnter(Screen::InGame), or
    // else init the resource with a default.
    commands.insert_resource(CollisionsBuffer(collisions));
}

fn gravity(mut has_gravity: Query<(&CharacterState, &mut Transform), With<Gravity>>) {
    for (state, mut t) in &mut has_gravity {
        if *state == CharacterState::Falling {
            t.translation.y -= 3.;
        }
    }
}

fn prepare_bind_group(
    buffers: Res<RenderAssets<GpuShaderStorageBuffer>>,
    collisions: Res<CollisionsBuffer>,
    mut commands: Commands,
    pipeline: Res<CollisionsPipeline>,
    render_device: Res<RenderDevice>,
) {
    let shader_storage = buffers.get(&collisions.0).unwrap();
    let bind_group = render_device.create_bind_group(
        None,
        &pipeline.layout,
        &BindGroupEntries::sequential((shader_storage.buffer.as_entire_buffer_binding(),)),
    );
    commands.insert_resource(CollisionsBufferBindGroup(bind_group));
}

#[derive(Resource)]
struct CollisionsPipeline {
    layout: BindGroupLayout,
    pipeline: CachedComputePipelineId,
}

impl FromWorld for CollisionsPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let layout = render_device.create_bind_group_layout(
            None,
            &BindGroupLayoutEntries::sequential(
                ShaderStages::COMPUTE,
                (storage_buffer::<Vec<u32>>(false),),
            ),
        );
        let shader = world.load_asset(SHADER_ASSET_PATH);
        let pipeline_cache = world.resource::<PipelineCache>();
        let pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some("Collisions compute shader".into()),
            layout: vec![layout.clone()],
            push_constant_ranges: Vec::new(),
            shader: shader.clone(),
            shader_defs: Vec::new(),
            entry_point: "main".into(),
            zero_initialize_workgroup_memory: false,
        });
        CollisionsPipeline { layout, pipeline }
    }
}

#[derive(Default)]
struct CollisionsNode {}

impl render_graph::Node for CollisionsNode {
    fn run(
        &self,
        _graph: &mut render_graph::RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), render_graph::NodeRunError> {
        let pipeline_cache = world.resource::<PipelineCache>();
        let pipeline = world.resource::<CollisionsPipeline>();
        let bind_group = world.resource::<CollisionsBufferBindGroup>();

        if let Some(init_pipeline) = pipeline_cache.get_compute_pipeline(pipeline.pipeline) {
            let mut pass =
                render_context
                    .command_encoder()
                    .begin_compute_pass(&ComputePassDescriptor {
                        label: Some("Collisions compute pass"),
                        ..default()
                    });

            pass.set_bind_group(0, &bind_group.0, &[]);
            pass.set_pipeline(init_pipeline);
            // TODO: figure out config values/constants.
            pass.dispatch_workgroups(200, 1, 1);
        }
        Ok(())
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
//
