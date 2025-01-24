#import bevy_sprite::mesh2d_vertex_output::VertexOutput

const MESH_DIMENSIONS: vec2<f32> = vec2<f32>(2560., 1440.);

@group(2) @binding(0) var<uniform> cursor_position: vec2<f32>;
@group(2) @binding(1) var terrain_texture: texture_2d<f32>;
@group(2) @binding(2) var terrain_texture_sampler: sampler;
@group(2) @binding(3) var mask_texture: texture_2d<f32>;
@group(2) @binding(4) var mask_texture_sampler: sampler;

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    var terrain_color = textureSample(terrain_texture, terrain_texture_sampler, mesh.uv);
    if terrain_color.a == 0. {
        // Exit early, we don't need to do anything else. Note that this might not always be true,
        // if we end up introducing mechanics by which empty space can be "filled in". For example,
        // could we implement builders this way?
        return terrain_color;
    }

    // Normalise pos to dimensions of underlying mesh (NOT window dimensions).
    let npos = cursor_position / MESH_DIMENSIONS;
    let diff = mesh.uv - npos;
    // Scaling the diff by aspect ratio avoids the "squashed circle" problem.
    let scaled_diff = vec2<f32>(diff.x * (MESH_DIMENSIONS.x / MESH_DIMENSIONS.y), diff.y);
    let distance = length(scaled_diff);

    // Compare with the radius of the 20px mask texture, converted to UV via the smaller of the mesh
    // dimensions.
    if distance < (10. / MESH_DIMENSIONS.y) {
        // Convert NDC value to UV for mask texture sampling.
        let mask_uv = (diff + vec2<f32>(1.0)) / 2.0;
        var mask_color = textureSample(mask_texture, mask_texture_sampler, mask_uv);
        terrain_color.a = mask_color.a;
    }

    return terrain_color;
}
