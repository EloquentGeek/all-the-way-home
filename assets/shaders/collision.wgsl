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

// This buffer contains Vec4-aligned bytes:
// (
//    x:       current x coord for collision point of Yup
//    y:       current y coord for collision point of Yup
//    z:       entity id
//    _:       unused padding
// )
@group(0) @binding(0) var<uniform> yups: array<vec4<f32>, 100>;

// And this buffer contains the current level image to check for alpha. Alpha > 0.0 is collide-able.
@group(0) @binding(1) var texture: texture_storage_2d<rgba8unorm, read>;

// Finally, this buffer allows us to write bit-packed data back to the CPU
@group(0) @binding(2) var<storage, read_write> collisions: array<u32>;

@compute @workgroup_size(1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    // Here, the global_id being passed is an index 0..99 because we've asked for 100 workgroups to
    // be dispatched (one for each Yup). To obtain the actual storage buffer index, we multiply by 3
    // because there are three elements in each Yup's data.
    let yup = yups[global_id.x];
    if yup.z == 0.0f {
        // We can safely ignore, no entity id.
        return;
    }

    let pixel = textureLoad(texture, vec2<u32>(floor(yup.xy)));
    if pixel.a > 0.0f {
        // We collided! Set the collision bit. This is a little fussy, could just set to 1! However
        // if we need to encode more info later, this might be handy.
        // collisions[global_id.x] = (0 << 1u) | 1u;
        collisions[global_id.x] = 1u;
    } else {
        // No collision, clear the bit.
        // yups[global_id.x] = payload << 1u;
        collisions[global_id.x] = 0u;
    }
}
