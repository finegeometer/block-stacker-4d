#version 300 es
precision mediump float;
precision mediump usampler2D;

in vec4 vpos;
in vec4 v_three_screen_pos;

out vec4 color;

uniform vec4 four_camera_pos;

// A tiny (length 1e-6 or something) vector, in world coordinates,
// that when transformed by the four-camera, turns into a vector pointing directly away from the three-camera.
uniform vec4 tiny_three_camera_fleeing_step_in_world_coordinates_a;
uniform float tiny_three_camera_fleeing_step_in_world_coordinates_b;

// Because there isn't a sampler4D
uniform usampler2D world;

int world_size = 8;
float render_distance = 20.0;


vec4 block_color(uint id) {
    if (id == uint(0)) {
        return vec4(1., 1., 1., 0.);
    } else if (id == uint(1)) {
        return vec4(0.5, 0.5, 0.5, 1.);
    } else if (id == uint(2)) {
        return vec4(0.0, 0.8, 0.0, 1.);
    } else {
        return vec4(1.0, 0.0, 1.0, 1.0);
    }
}

uint get_block(ivec4 pos) {
    if (pos.x < 0 || pos.y < 0 || pos.z < 0 || pos.w < 0 || pos.x >= world_size || pos.y >= world_size || pos.z >= world_size || pos.w >= world_size) {
        return uint(0);
    } else {
        return texelFetch(world, ivec2(pos.x + world_size*pos.z, pos.y + world_size*pos.w), 0).r;
    }
}


// Raytrace along the ray from `start` through `end`.
// If the ray intersects the scene at `mix(start, end, parameter)`:
//     Set t = parameter.
//     Set col = the color of the surface at the intersection.
//     Return true;
// If there is no intersection, return false.

// Note: A translation of this into Rust is in `world.rs`.
bool intersect_scene(vec4 start, vec4 end, out float t, out vec4 col) {

    float t_max = render_distance / length(end - start);

    vec4 t_steps = vec4(1.0) / abs(end - start);
    vec4 next_ts = fract(-start * sign(end - start)) * t_steps;

    ivec4 block_steps = ivec4(sign(end - start));
    ivec4 current_block = ivec4(floor(start));

    t = 0.0;
    while (t < t_max && get_block(current_block) == uint(0)) {
        if (min(next_ts.x, next_ts.y) < min(next_ts.z, next_ts.w)) {
            if (next_ts.x < next_ts.y) {
                t = next_ts.x;
                next_ts.x += t_steps.x;
                current_block.x += block_steps.x;
            } else {
                t = next_ts.y;
                next_ts.y += t_steps.y;
                current_block.y += block_steps.y;
            }
        } else {
            if (next_ts.z < next_ts.w) {
                t = next_ts.z;
                next_ts.z += t_steps.z;
                current_block.z += block_steps.z;
            } else {
                t = next_ts.w;
                next_ts.w += t_steps.w;
                current_block.w += block_steps.w;
            }
        }
    }

    col = block_color(get_block(current_block));

    return t < t_max;

}


void main() {
    // limit render distance to infinity
    if (v_three_screen_pos.w < 0.0) {
        discard;
    }

    // further limit it to render_distance
    if (dot(vpos-four_camera_pos, vpos-four_camera_pos) >= render_distance * render_distance) {
        discard;
    }

    vec4 adjusted_pos = (vpos + tiny_three_camera_fleeing_step_in_world_coordinates_a) / (1.0 + tiny_three_camera_fleeing_step_in_world_coordinates_b);

    float t;
    if (intersect_scene(four_camera_pos, adjusted_pos, t, color)) {
        if (t < 0.993) {
            // Occluded
            discard;
        }
        color = mix(color, vec4(1.0), 0.8);
    } else {
        // Sky
        color = vec4(0.8, 0.9, 1.0, 1.0);
    }
}
