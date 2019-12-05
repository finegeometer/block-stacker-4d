#version 300 es

in vec4 pos;

// World -> 3-screen
// x y z w hmg -> x y z hmg
uniform mat4 four_camera_a;
uniform vec4 four_camera_b;

// 3-screen -> 2-screen
// x y z hmg -> x y dpth hmg
uniform mat4 three_camera;

out vec4 vpos;
out vec4 v_three_screen_pos;

void main() {
    vpos = pos;
    v_three_screen_pos = four_camera_a * pos + four_camera_b;
    gl_Position = three_camera * v_three_screen_pos;
}
