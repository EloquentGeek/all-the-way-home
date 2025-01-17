#import bevy_sprite::mesh2d_vertex_output::VertexOutput

@group(2) @binding(0) var<uniform> cursor_position: vec2<f32>;
@group(2) @binding(1) var terrain_texture: texture_2d<f32>;
@group(2) @binding(2) var terrain_texture_sampler: sampler;
@group(2) @binding(3) var mask_texture: texture_2d<f32>;
@group(2) @binding(4) var mask_texture_sampler: sampler;

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    var terrain_color = textureSample(terrain_texture, terrain_texture_sampler, mesh.uv);

    let diff = mesh.position.xy - cursor_position;
    if all(abs(diff) < vec2<f32>(36.0, 36.0)) {
        // Convert the difference to UV coordinates for the mask (0 to 1 range)
        // Add 0.5 to center the mask (moving from -36..36 to 0..1 range)
        let mask_uv = (diff + vec2<f32>(36.0)) / 72.0;

        var mask_color = textureSample(mask_texture, mask_texture_sampler, mask_uv);
        if terrain_color.a != 0. {
            terrain_color.a = mask_color.a;
        }
    }

    return terrain_color;
}
