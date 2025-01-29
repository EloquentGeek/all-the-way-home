use bevy::{
    asset::RenderAssetUsages,
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
        texture::GpuImage,
    },
};
use binding_types::texture_storage_2d;
use tiny_bail::prelude::*;

use crate::game::{level::LevelRenderTargets, yup::CharacterState};

const SHADER_ASSET_PATH: &str = "shaders/collision.wgsl";

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, init);
        app.add_systems(FixedUpdate, gravity);

        // Without these, the resources are not available in the pipeline.
        app.add_plugins(ExtractResourcePlugin::<CollisionsBuffer>::default());
        app.add_plugins(ExtractResourcePlugin::<LevelRenderTargets>::default());
    }

    fn finish(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);
        render_app
            .init_resource::<CollisionsPipeline>()
            .add_systems(
                Render,
                prepare_bind_group
                    .in_set(RenderSet::PrepareBindGroups)
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

#[derive(Resource, ExtractResource, Clone)]
struct CollisionsBuffer(Handle<ShaderStorageBuffer>);

#[derive(Resource)]
struct CollisionsBufferBindGroup(BindGroup);

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
struct CollisionsNodeLabel;

#[derive(Component, Debug)]
pub struct Gravity;

fn init(
    mut commands: Commands,
    mut buffers: ResMut<Assets<ShaderStorageBuffer>>,
    mut images: ResMut<Assets<Image>>,
) {
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

    // Ensure sensibly-formatted render target image exists to initialise the compute pipeline.
    // These will get replaced once level loading begins in Screen::Intro.
    let mut blank_image = Image::new_fill(
        Extent3d {
            // TODO: can we really get away with this, or do we need to use the 2k image size like
            // the real levels? Inquiring minds want to know.
            width: 2560,
            height: 1440,
            ..default()
        },
        TextureDimension::D2,
        &[0, 0, 0, 0],
        TextureFormat::Rgba8Unorm,
        // We don't care if this image ever exists in the main world. It's entirely a GPU resource.
        RenderAssetUsages::RENDER_WORLD,
    );
    blank_image.texture_descriptor.usage |=
        TextureUsages::COPY_SRC | TextureUsages::STORAGE_BINDING;
    let blank_handle = images.add(blank_image);

    commands.insert_resource(LevelRenderTargets {
        destination: blank_handle.clone(),
        source: blank_handle.clone(),
    });
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
    mut images: ResMut<RenderAssets<GpuImage>>,
    level_targets: Res<LevelRenderTargets>,
    pipeline: Res<CollisionsPipeline>,
    render_device: Res<RenderDevice>,
) {
    let shader_storage = buffers.get(&collisions.0).unwrap();
    let destination_image = r!(images.get_mut(&level_targets.destination));

    // NOTE: forcing this from the Srgb variant will lead to incorrect gamma values. Since we're
    // mostly interested in the alpha, which remains the same, this shouldn't be a problem. See
    // https://docs.rs/bevy_color/latest/bevy_color/#conversion. We could also do away with this if
    // compute shaders ever support storage textures that are Rgba8UnormSrgb.
    destination_image.texture_format = TextureFormat::Rgba8Unorm;

    let bind_group = render_device.create_bind_group(
        None,
        &pipeline.layout,
        &BindGroupEntries::sequential((
            shader_storage.buffer.as_entire_buffer_binding(),
            destination_image.texture_view.into_binding(),
        )),
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
                (
                    storage_buffer::<Vec<u32>>(false),
                    texture_storage_2d(TextureFormat::Rgba8Unorm, StorageTextureAccess::ReadWrite),
                ),
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

        // NOTE: we need to alter the bindgroup for each new level, because the image we're passing
        // to the compute shader CHANGES each time we load a level.
        let Some(bind_group) = world.get_resource::<CollisionsBufferBindGroup>() else {
            // TODO: this is not great, but lets us await the eventual insertion of the above
            // resource when we arrive at the intro screen for each level?
            return Ok(());
        };

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
