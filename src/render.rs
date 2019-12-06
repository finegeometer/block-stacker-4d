mod gl_handler;

const WORLD_SIZE: usize = 8;

pub use gl_handler::{Uniforms, Viewport};

pub struct Renderer {
    gl_handler: gl_handler::GlHandler,
    mesh_dirty: bool,
}

pub enum Msg {
    BlockChanged([isize; 4]),
}

impl Renderer {
    pub fn new(gl: web_sys::WebGl2RenderingContext) -> Self {
        Self {
            gl_handler: gl_handler::GlHandler::new(gl),
            mesh_dirty: false,
        }
    }

    pub fn update(&mut self, world: &super::world::World, msg: Msg) {
        match msg {
            Msg::BlockChanged(block) => {
                if block.iter().any(|&x| x < 0 || x >= WORLD_SIZE as isize) {
                    return;
                }
                self.gl_handler.set_texture_pixel(
                    [
                        block[0] as usize + WORLD_SIZE * block[2] as usize,
                        block[1] as usize + WORLD_SIZE * block[3] as usize,
                    ],
                    &world[block],
                )
            }
        }

        self.mesh_dirty = true;
    }

    pub fn clear_canvas(&self) {
        self.gl_handler.clear_canvas();
    }

    pub fn render(&mut self, world: &super::world::World, viewport: Viewport, uniforms: Uniforms) {
        if self.mesh_dirty {
            self.redo_mesh(world);
        }
        self.gl_handler.render(viewport, uniforms);
    }

    pub fn redo_mesh(&mut self, world: &super::world::World) {
        let mut vertex_data: Vec<f32> = Vec::new();

        fn all_or_nothing(blocks: &[&crate::block::Block]) -> bool {
            blocks.iter().all(|b| **b == crate::block::BlockName::Air)
                || blocks.iter().all(|b| **b != crate::block::BlockName::Air)
        }

        for i in 0..WORLD_SIZE as isize {
            for j in 0..WORLD_SIZE as isize {
                for k in 0..=WORLD_SIZE as isize {
                    for l in 0..=WORLD_SIZE as isize {
                        if !all_or_nothing(&[
                            &world[[i, j, k, l]],
                            &world[[i, j, k - 1, l]],
                            &world[[i, j, k, l - 1]],
                            &world[[i, j, k - 1, l - 1]],
                        ]) {
                            #[rustfmt::skip]
                                vertex_data.extend_from_slice(&[
                                    i as f32     , j as f32     , k as f32, l as f32,
                                    i as f32 + 1., j as f32     , k as f32, l as f32,
                                    i as f32 + 1., j as f32 + 1., k as f32, l as f32,
                                    i as f32 + 1., j as f32 + 1., k as f32, l as f32,
                                    i as f32     , j as f32 + 1., k as f32, l as f32,
                                    i as f32     , j as f32     , k as f32, l as f32,
                                ]);
                        }

                        if !all_or_nothing(&[
                            &world[[i, k, j, l]],
                            &world[[i, k - 1, j, l]],
                            &world[[i, k, j, l - 1]],
                            &world[[i, k - 1, j, l - 1]],
                        ]) {
                            #[rustfmt::skip]
                                vertex_data.extend_from_slice(&[
                                    i as f32     , k as f32, j as f32     , l as f32,
                                    i as f32 + 1., k as f32, j as f32     , l as f32,
                                    i as f32 + 1., k as f32, j as f32 + 1., l as f32,
                                    i as f32 + 1., k as f32, j as f32 + 1., l as f32,
                                    i as f32     , k as f32, j as f32 + 1., l as f32,
                                    i as f32     , k as f32, j as f32     , l as f32,
                                ]);
                        }

                        if !all_or_nothing(&[
                            &world[[k, i, j, l]],
                            &world[[k - 1, i, j, l]],
                            &world[[k, i, j, l - 1]],
                            &world[[k - 1, i, j, l - 1]],
                        ]) {
                            #[rustfmt::skip]
                                vertex_data.extend_from_slice(&[
                                    k as f32, i as f32     , j as f32     , l as f32,
                                    k as f32, i as f32 + 1., j as f32     , l as f32,
                                    k as f32, i as f32 + 1., j as f32 + 1., l as f32,
                                    k as f32, i as f32 + 1., j as f32 + 1., l as f32,
                                    k as f32, i as f32     , j as f32 + 1., l as f32,
                                    k as f32, i as f32     , j as f32     , l as f32,
                                ]);
                        }

                        if !all_or_nothing(&[
                            &world[[i, k, l, j]],
                            &world[[i, k - 1, l, j]],
                            &world[[i, k, l - 1, j]],
                            &world[[i, k - 1, l - 1, j]],
                        ]) {
                            #[rustfmt::skip]
                                vertex_data.extend_from_slice(&[
                                    i as f32     , k as f32, l as f32, j as f32     ,
                                    i as f32 + 1., k as f32, l as f32, j as f32     ,
                                    i as f32 + 1., k as f32, l as f32, j as f32 + 1.,
                                    i as f32 + 1., k as f32, l as f32, j as f32 + 1.,
                                    i as f32     , k as f32, l as f32, j as f32 + 1.,
                                    i as f32     , k as f32, l as f32, j as f32     ,
                                ]);
                        }

                        if !all_or_nothing(&[
                            &world[[k, i, l, j]],
                            &world[[k - 1, i, l, j]],
                            &world[[k, i, l - 1, j]],
                            &world[[k - 1, i, l - 1, j]],
                        ]) {
                            #[rustfmt::skip]
                                vertex_data.extend_from_slice(&[
                                    k as f32, i as f32     , l as f32, j as f32     ,
                                    k as f32, i as f32 + 1., l as f32, j as f32     ,
                                    k as f32, i as f32 + 1., l as f32, j as f32 + 1.,
                                    k as f32, i as f32 + 1., l as f32, j as f32 + 1.,
                                    k as f32, i as f32     , l as f32, j as f32 + 1.,
                                    k as f32, i as f32     , l as f32, j as f32     ,
                                ]);
                        }

                        if !all_or_nothing(&[
                            &world[[k, l, i, j]],
                            &world[[k - 1, l, i, j]],
                            &world[[k, l - 1, i, j]],
                            &world[[k - 1, l - 1, i, j]],
                        ]) {
                            #[rustfmt::skip]
                                vertex_data.extend_from_slice(&[
                                    k as f32, l as f32, i as f32     , j as f32     ,
                                    k as f32, l as f32, i as f32 + 1., j as f32     ,
                                    k as f32, l as f32, i as f32 + 1., j as f32 + 1.,
                                    k as f32, l as f32, i as f32 + 1., j as f32 + 1.,
                                    k as f32, l as f32, i as f32     , j as f32 + 1.,
                                    k as f32, l as f32, i as f32     , j as f32     ,
                                ]);
                        }
                    }
                }
            }
        }

        self.gl_handler.set_vertex_data(&vertex_data);
        self.mesh_dirty = false;
    }
}
