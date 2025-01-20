#import bevy_sprite::mesh2d_vertex_output::VertexOutput

@group(2) @binding(0) var<uniform> cursor_position: vec2<f32>;
@group(2) @binding(1) var<uniform> level_viewport: vec2<f32>;
@group(2) @binding(2) var terrain_texture: texture_2d<f32>;
@group(2) @binding(3) var terrain_texture_sampler: sampler;
@group(2) @binding(4) var mask_texture: texture_2d<f32>;
@group(2) @binding(5) var mask_texture_sampler: sampler;

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    var terrain_color = textureSample(terrain_texture, terrain_texture_sampler, mesh.uv);

    // For centre of screen:
    // level_viewport here is 640, 360 (centred)
    // mesh.position.xy is 1280, 720 because it's the middle of a 2k screen
    // cursor_position will be 640, 360 (centre of screen)
    // adding level_viewport + mesh.position.xy == 1920, 1080 which makes little sense
    // instead, our target value should be the centre of 2k which is 1280, 720
    // it also needs to work when viewport is 0, 0 or 1920, 720, the min and max possible for viewport
    // let adjusted_position = mesh.position.xy + level_viewport;
    // let diff = adjusted_position - cursor_position;

    let diff = mesh.uv - cursor_position;

    if all(abs(diff) < vec2<f32>(0.01, 0.01)) {
        // TODO: should this be different to adjust for actual mesh position?
        let mask_uv = (diff + vec2<f32>(1., 1.)) / 2.;

        var mask_color = textureSample(mask_texture, mask_texture_sampler, mask_uv);
        if terrain_color.a != 0. {
            terrain_color.a = mask_color.a;
        }
    }

    return terrain_color;
}
