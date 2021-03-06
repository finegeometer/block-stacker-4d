use crate::chunk::CHUNK_SIZE;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

type GL = web_sys::WebGl2RenderingContext;

pub struct GlHandler {
    gl: GL,
    program: web_sys::WebGlProgram,
    vao: web_sys::WebGlVertexArrayObject,
    vertex_buffer: web_sys::WebGlBuffer,
    world_tex: web_sys::WebGlTexture,

    num_triangles: usize,
}

impl Drop for GlHandler {
    fn drop(&mut self) {
        self.gl.delete_program(Some(&self.program));
        self.gl.delete_vertex_array(Some(&self.vao));
        self.gl.delete_buffer(Some(&self.vertex_buffer));
        self.gl.delete_texture(Some(&self.world_tex));
    }
}

pub struct Viewport {
    pub start: [i32; 2],
    pub size: [i32; 2],
}
pub struct Uniforms {
    pub four_camera: nalgebra::Matrix5<f32>,
    pub three_camera: nalgebra::Matrix4<f32>,
}

impl GlHandler {
    pub fn new(gl: GL) -> Self {
        // Multiplicative Blending
        gl.enable(GL::BLEND);
        gl.blend_func(GL::DST_COLOR, GL::ZERO);

        let program = compile_program(&gl);

        let vao = gl.create_vertex_array().unwrap_throw();
        gl.bind_vertex_array(Some(&vao));

        let vertex_buffer = gl.create_buffer().unwrap_throw();

        let attribute_pos = gl.get_attrib_location(&program, "pos") as u32;

        gl.bind_buffer(GL::ARRAY_BUFFER, Some(&vertex_buffer));
        gl.enable_vertex_attrib_array(attribute_pos);
        gl.vertex_attrib_pointer_with_i32(attribute_pos, 4, GL::FLOAT, false, 4 * 4, 0);

        gl.buffer_data_with_array_buffer_view(
            GL::ARRAY_BUFFER,
            &as_f32_array(&[]).into(),
            GL::STATIC_DRAW,
        );

        let world_tex = gl.create_texture().unwrap_throw();
        gl.bind_texture(GL::TEXTURE_2D_ARRAY, Some(&world_tex));
        gl.pixel_storei(GL::UNPACK_ALIGNMENT, 1);
        gl.tex_image_3d_with_opt_u8_array(
            GL::TEXTURE_2D_ARRAY,
            0,                                // level
            GL::R8UI as i32,                  // internal_format
            (CHUNK_SIZE * CHUNK_SIZE) as i32, // width
            (CHUNK_SIZE * CHUNK_SIZE) as i32, // height
            256,                              // depth
            0,                                // border
            GL::RED_INTEGER,                  // format
            GL::UNSIGNED_BYTE,                // type
            Some(&[0; 256 * CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE]),
        )
        .unwrap_throw();

        gl.tex_parameteri(
            GL::TEXTURE_2D_ARRAY,
            GL::TEXTURE_MIN_FILTER,
            GL::NEAREST as i32,
        );
        gl.tex_parameteri(
            GL::TEXTURE_2D_ARRAY,
            GL::TEXTURE_MAG_FILTER,
            GL::NEAREST as i32,
        );
        gl.tex_parameteri(
            GL::TEXTURE_2D_ARRAY,
            GL::TEXTURE_WRAP_S,
            GL::CLAMP_TO_EDGE as i32,
        );
        gl.tex_parameteri(
            GL::TEXTURE_2D_ARRAY,
            GL::TEXTURE_WRAP_T,
            GL::CLAMP_TO_EDGE as i32,
        );

        Self {
            gl,
            program,
            vao,
            vertex_buffer,
            world_tex,

            num_triangles: 0,
        }
    }

    pub fn set_vertex_data(&mut self, data: &[f32]) {
        self.num_triangles = data.len() / 4;

        self.gl
            .bind_buffer(GL::ARRAY_BUFFER, Some(&self.vertex_buffer));
        self.gl.buffer_data_with_array_buffer_view(
            GL::ARRAY_BUFFER,
            &as_f32_array(data).into(),
            GL::STATIC_DRAW,
        );
    }

    pub fn set_texture(&self, which_texture: usize, blocks: &[u8]) {
        self.gl
            .bind_texture(GL::TEXTURE_2D_ARRAY, Some(&self.world_tex));
        self.gl
            .tex_sub_image_3d_with_opt_u8_array(
                GL::TEXTURE_2D_ARRAY,
                0,
                0,
                0,
                which_texture as i32,
                (CHUNK_SIZE * CHUNK_SIZE) as i32,
                (CHUNK_SIZE * CHUNK_SIZE) as i32,
                1,
                GL::RED_INTEGER,
                GL::UNSIGNED_BYTE,
                Some(blocks),
            )
            .unwrap_throw();
    }

    /// Does not bounds-check texture_coordinate.
    pub fn set_texture_pixel(
        &self,
        which_texture: usize,
        texture_coordinate: [usize; 2],
        block: crate::block::BlockName,
    ) {
        self.gl
            .bind_texture(GL::TEXTURE_2D_ARRAY, Some(&self.world_tex));
        self.gl
            .tex_sub_image_3d_with_opt_u8_array(
                GL::TEXTURE_2D_ARRAY,
                0,
                texture_coordinate[0] as i32,
                texture_coordinate[1] as i32,
                which_texture as i32,
                1,
                1,
                1,
                GL::RED_INTEGER,
                GL::UNSIGNED_BYTE,
                Some(&[block as u8]),
            )
            .unwrap_throw();
    }

    pub fn clear_canvas(&self) {
        self.gl.clear_color(1., 1., 1., 1.);
        self.gl.clear(GL::COLOR_BUFFER_BIT);
    }

    pub fn render(&self, viewport: Viewport, uniforms: Uniforms) {
        // x y z dpth hmg -> x y z w hmg
        let four_camera_inverse = uniforms.four_camera.try_inverse().unwrap_throw();
        let four_camera_pos = four_camera_inverse * nalgebra::Vector5::new(0., 0., 0., -1., 0.);
        let four_camera_pos = four_camera_pos.remove_row(4) * four_camera_pos[4];

        // x y z w hmg -> x y z hmg
        let four_camera_no_depth = uniforms.four_camera.remove_row(3);
        let four_camera_no_depth = four_camera_no_depth.as_slice();

        self.gl.use_program(Some(&self.program));
        self.gl.bind_vertex_array(Some(&self.vao));

        self.gl.uniform4f(
            self.gl
                .get_uniform_location(&self.program, "four_camera_pos")
                .as_ref(),
            four_camera_pos[0],
            four_camera_pos[1],
            four_camera_pos[2],
            four_camera_pos[3],
        );

        self.gl.uniform_matrix4fv_with_f32_array(
            self.gl
                .get_uniform_location(&self.program, "four_camera_a")
                .as_ref(),
            false,
            &four_camera_no_depth[0..16],
        );

        self.gl.uniform4f(
            self.gl
                .get_uniform_location(&self.program, "four_camera_b")
                .as_ref(),
            four_camera_no_depth[16],
            four_camera_no_depth[17],
            four_camera_no_depth[18],
            four_camera_no_depth[19],
        );

        self.gl
            .bind_texture(GL::TEXTURE_2D_ARRAY, Some(&self.world_tex));
        self.gl.uniform1i(
            self.gl
                .get_uniform_location(&self.program, "world")
                .as_ref(),
            0,
        );

        self.gl.uniform_matrix4fv_with_f32_array(
            self.gl
                .get_uniform_location(&self.program, "three_camera")
                .as_ref(),
            false,
            &uniforms.three_camera.as_slice(),
        );

        // x y dpth hmg -> x y z hmg
        let three_camera_inverse = uniforms.three_camera.try_inverse().unwrap_throw();
        let tiny_three_camera_fleeing_step_in_three_screen_coordinates =
            (three_camera_inverse * nalgebra::Vector4::new(0., 0., 1., 0.)).normalize() * 1e-5;
        let tiny_three_camera_fleeing_step_in_world_coordinates = four_camera_inverse
            * tiny_three_camera_fleeing_step_in_three_screen_coordinates.insert_row(3, 0.);

        self.gl.uniform4f(
            self.gl
                .get_uniform_location(
                    &self.program,
                    "tiny_three_camera_fleeing_step_in_world_coordinates_a",
                )
                .as_ref(),
            tiny_three_camera_fleeing_step_in_world_coordinates[0],
            tiny_three_camera_fleeing_step_in_world_coordinates[1],
            tiny_three_camera_fleeing_step_in_world_coordinates[2],
            tiny_three_camera_fleeing_step_in_world_coordinates[3],
        );

        self.gl.uniform1f(
            self.gl
                .get_uniform_location(
                    &self.program,
                    "tiny_three_camera_fleeing_step_in_world_coordinates_b",
                )
                .as_ref(),
            tiny_three_camera_fleeing_step_in_world_coordinates[4],
        );

        self.gl.viewport(
            viewport.start[0],
            viewport.start[1],
            viewport.size[0],
            viewport.size[1],
        );
        self.gl
            .draw_arrays(GL::TRIANGLES, 0, self.num_triangles as i32);
    }
}

fn compile_program(gl: &GL) -> web_sys::WebGlProgram {
    let vertex_shader = gl.create_shader(GL::VERTEX_SHADER).unwrap_throw();
    gl.shader_source(&vertex_shader, include_str!("shaders/vertex.glsl"));
    gl.compile_shader(&vertex_shader);

    web_sys::console::log_1(&gl.get_shader_info_log(&vertex_shader).unwrap_throw().into());

    let fragment_shader = gl.create_shader(GL::FRAGMENT_SHADER).unwrap_throw();
    gl.shader_source(
        &fragment_shader,
        &format!(
            include_str!("shaders/fragment.glsl"),
            CHUNK_SIZE,
            super::RENDER_CHUNKS,
            super::RENDER_DISTANCE,
        ),
    );
    gl.compile_shader(&fragment_shader);

    web_sys::console::log_1(
        &gl.get_shader_info_log(&fragment_shader)
            .unwrap_throw()
            .into(),
    );

    let program = gl.create_program().unwrap_throw();
    gl.attach_shader(&program, &vertex_shader);
    gl.attach_shader(&program, &fragment_shader);
    gl.link_program(&program);

    gl.delete_shader(Some(&vertex_shader));
    gl.delete_shader(Some(&fragment_shader));

    program
}

fn as_f32_array(v: &[f32]) -> js_sys::Float32Array {
    let memory_buffer = wasm_bindgen::memory()
        .dyn_into::<js_sys::WebAssembly::Memory>()
        .unwrap_throw()
        .buffer();

    let location = v.as_ptr() as u32 / 4;

    js_sys::Float32Array::new(&memory_buffer).subarray(location, location + v.len() as u32)
}
