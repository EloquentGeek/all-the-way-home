use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::{
        Render, RenderApp, RenderSet,
        extract_resource::{ExtractResource, ExtractResourcePlugin},
        gpu_readback::{Readback, ReadbackComplete},
        render_asset::RenderAssets,
        render_graph::{self, RenderGraph, RenderLabel},
        render_resource::*,
        renderer::{RenderContext, RenderDevice, RenderQueue},
        storage::{GpuShaderStorageBuffer, ShaderStorageBuffer},
        texture::GpuImage,
    },
};
use binding_types::{storage_buffer, texture_storage_2d, uniform_buffer};
use tiny_bail::prelude::*;

use crate::game::{
    level::{Level, LevelRenderTargets},
    yup::Yup,
};

const SHADER_ASSET_PATH: &str = "shaders/collision.wgsl";
const YUP_COUNT: usize = 100;
// NOTE: actual number of u32's is 400, but this supports byte alignment via use of Vec4.
// In other words, we're passing 400 values, the last of each 4 will be ignored as padding.
const YUP_BUFFER_SIZE: usize = 100;

pub struct CollisionPlugin;

impl Plugin for CollisionPlugin {
    fn build(&self, app: &mut App) {
        // Without these, the resources are not available in the pipeline.
        app.add_plugins(ExtractResourcePlugin::<CollisionsBuffer>::default());
        app.add_plugins(ExtractResourcePlugin::<CollisionsTerrain>::default());
        // TODO: is this necessary, given we're passing as uniform?
        app.add_plugins(ExtractResourcePlugin::<YupBuffer>::default());
        app.add_plugins(ExtractResourcePlugin::<LevelRenderTargets>::default());

        app.add_systems(Startup, init);
        app.add_systems(FixedPostUpdate, update_yup_locations);
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

#[derive(Resource, ExtractResource, Clone, Deref, DerefMut)]
pub struct CollisionsTerrain(pub Handle<Image>);

#[derive(Resource, ExtractResource, Clone, Deref, DerefMut, ShaderType)]
struct YupBuffer {
    pub yups: [Vec4; YUP_BUFFER_SIZE],
}

impl YupBuffer {
    fn default() -> Self {
        Self {
            yups: [Vec4::ZERO; YUP_BUFFER_SIZE],
        }
    }

    fn get_uniform(&self) -> UniformBuffer<&Self> {
        UniformBuffer::from(self)
    }
}

#[derive(Resource)]
struct CollisionsBufferBindGroup(BindGroup);

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
struct CollisionsNodeLabel;

fn init(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut shader_storage_buffers: ResMut<Assets<ShaderStorageBuffer>>,
) {
    // The collisions buffer contains bit-packed information sent back from the GPU, but for now
    // just a 1 if a collision has occurred, 0 otherwise.
    let mut collisions_buffer = ShaderStorageBuffer::from(vec![0u32; YUP_COUNT]);
    collisions_buffer.buffer_description.usage |= BufferUsages::COPY_SRC | BufferUsages::STORAGE;
    let collisions = shader_storage_buffers.add(collisions_buffer);

    commands
        .spawn(Readback::buffer(collisions.clone()))
        .observe(|trigger: Trigger<ReadbackComplete>| {
            // This matches the type which was used to create the `ShaderStorageBuffer` above,
            // and is a convenient way to interpret the data.
            let data: Vec<u32> = trigger.event().to_shader_type();
            info!("Buffer {:?}", data);
        });
    // NOTE: need to make sure nothing accesses this resource before OnEnter(Screen::InGame), or
    // else init the resource with a default.
    commands.insert_resource(CollisionsBuffer(collisions));

    // Also init an empty buffer for our Yup locations.
    commands.insert_resource(YupBuffer::default());

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

    // User-visible render targets display each level terrain.
    commands.insert_resource(LevelRenderTargets {
        destination: blank_handle.clone(),
        source: blank_handle.clone(),
    });

    // Non-visible image for use as source of truth for collision detection.
    commands.insert_resource(CollisionsTerrain(blank_handle.clone()));
}

// Theory:
//
//   - we maintain a separate copy of the terrain image in compute-shader-friendly texture format
//     i.e. TextureFormat::Rgba8Unorm - at the end of each frame we clone this from the existing
//     TextureFormat::Rgba8UnormSrgb `destination` image (the current render target)
//   - switch its TextureFormat
//   - and use it as the source of truth for collision detection
//
// Cloning 2k images like this each frame seems expensive, but perhaps if there is no to-GPU copy
// involved, it can be done cheaply?
fn prepare_bind_group(
    collisions_buf: Res<CollisionsBuffer>,
    collisions_terrain: Res<CollisionsTerrain>,
    mut commands: Commands,
    mut images: ResMut<RenderAssets<GpuImage>>,
    pipeline: Res<CollisionsPipeline>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    shader_storage_buffers: Res<RenderAssets<GpuShaderStorageBuffer>>,
    yup_buf: Res<YupBuffer>,
) {
    let collisions_buffer = r!(shader_storage_buffers.get(&collisions_buf.0));
    let terrain_image = r!(images.get_mut(&collisions_terrain.0));
    let mut yup_buffer = yup_buf.get_uniform();
    yup_buffer.write_buffer(&render_device, &render_queue);

    // NOTE: forcing this from the Srgb variant will lead to incorrect gamma values. Since we're
    // mostly interested in the alpha, which remains the same, this shouldn't be a problem. See
    // https://docs.rs/bevy_color/latest/bevy_color/#conversion. We could also do away with this if
    // compute shaders ever support storage textures that are Rgba8UnormSrgb.
    terrain_image.texture_format = TextureFormat::Rgba8Unorm;

    let bind_group = render_device.create_bind_group(
        None,
        &pipeline.layout,
        &BindGroupEntries::sequential((
            // Entities and coords.
            yup_buffer.into_binding(),
            // Terrain to check for collisions.
            terrain_image.texture_view.into_binding(),
            // Results of collisions checks.
            collisions_buffer.buffer.as_entire_buffer_binding(),
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
                    // Entities and coords.
                    uniform_buffer::<YupBuffer>(false),
                    // Terrain to check for collisions.
                    texture_storage_2d(TextureFormat::Rgba8Unorm, StorageTextureAccess::ReadOnly),
                    // Results of collisions checks.
                    storage_buffer::<Vec<u32>>(false),
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
            // n 1D shader dispatches for n Yup's. One each! Their very own compute shader pass.
            // They'll treasure it always. Until the next frame.
            pass.dispatch_workgroups(YUP_COUNT as u32, 1, 1);
        }
        Ok(())
    }

    fn update(&mut self, world: &mut World) {
        let images = world.resource::<RenderAssets<GpuImage>>();
        let collisions_buffer = world.resource::<CollisionsBuffer>();
        let collisions_terrain = world.resource::<CollisionsTerrain>();
        let pipeline = world.resource::<CollisionsPipeline>();
        let render_device = world.resource::<RenderDevice>();
        let render_queue = world.resource::<RenderQueue>();
        let shader_storage_buffers = world.resource::<RenderAssets<GpuShaderStorageBuffer>>();
        let yup_buffer = world.resource::<YupBuffer>();

        let collisions = r!(shader_storage_buffers.get(&collisions_buffer.0));
        let terrain_image = r!(images.get(&collisions_terrain.0));
        let mut yup_uniform = yup_buffer.get_uniform();
        yup_uniform.write_buffer(render_device, render_queue);

        // Update our compute shader bind group to reflect the current state of Yup positioning,
        // and the current terrain.
        let bind_group = render_device.create_bind_group(
            None,
            &pipeline.layout,
            &BindGroupEntries::sequential((
                // Entities and coords.
                yup_uniform.into_binding(),
                // Terrain to check for collisions.
                terrain_image.texture_view.into_binding(),
                // Results of collisions checks.
                collisions.buffer.as_entire_buffer_binding(),
            )),
        );

        let mut collisions_bind_group = world.resource_mut::<CollisionsBufferBindGroup>();
        collisions_bind_group.0 = bind_group;
    }
}

fn update_yup_locations(
    level_transform: Single<&Transform, With<Level>>,
    mut yup_buf: ResMut<YupBuffer>,
    window: Single<&Window>,
    yups: Query<(Entity, &Transform), With<Yup>>,
) {
    // We need to pass
    //  - entity id
    //  - collision-point-x
    //  - collision-point-y
    for (i, (yup, t)) in yups.iter().enumerate() {
        let texture_pos = level_transform
            .compute_matrix()
            .inverse()
            .transform_point3(t.translation);
        yup_buf.yups[i] = Vec4::new(
            // Ordering the values like this just makes reading in the shader simpler (x is x, y ix
            // y, z is the id).
            texture_pos.x + window.width(),
            // Note the y-value inversion to convert from world pos.
            -texture_pos.y + window.height(),
            yup.index() as f32,
            0.0, // Unused padding.
        );
    }
}
