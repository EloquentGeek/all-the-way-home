// Collision detection

// What do we need back from the compute shader?
//
// 1. Ground collision: we need to know when to stop falling.
//   - if falling and collision occurs, stop falling
// 2. Forward collision: we need to know when to turn around.
//   - if walking and collision occrs, turn around
//
// Given the above, we may not need to know much about the collision itself, only that it has
// occurred. We can do any finer detail collision (e.g. between blocker and non-blocker characters)
// on the CPU side with no dramas.
//
// We just need to return an identifier, and an on-off bit determining if the preceding identifier
// has collided with something.

@group(0) @binding(0) var<storage, read_write> collisions: array<u32>;
@group(0) @binding(1) var texture: texture_storage_2d<rgba8unorm, read>;

@compute @workgroup_size(1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    // We use the global_id to index the array to make sure we don't
    // access data used in another workgroup.
    collisions[global_id.x] += 1u;
    // textureStore(texture, vec2<i32>(i32(global_id.x), 0), vec4<u32>(data[global_id.x], 0, 0, 0));
}
