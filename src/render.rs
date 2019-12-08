mod gl_handler;

use crate::chunk::CHUNK_SIZE;

// Changing this value requires changing other parts of the code.
const RENDER_CHUNKS: usize = 4;
const RENDER_DISTANCE: usize = ((RENDER_CHUNKS - 1) * CHUNK_SIZE) / 2;

pub use gl_handler::{Uniforms, Viewport};

pub struct Renderer {
    gl_handler: gl_handler::GlHandler,
    mesh_dirty: bool,

    corner_of_loaded_region: [isize; 4],
}

pub enum Msg {
    BlockChanged([isize; 4]),
    PlayerMoved([f32; 4]),
}

impl Renderer {
    pub fn new(gl: web_sys::WebGl2RenderingContext) -> Self {
        Self {
            gl_handler: gl_handler::GlHandler::new(gl),
            mesh_dirty: false,

            corner_of_loaded_region: [9999, 9999, 9999, 9999],
        }
    }

    pub fn update(&mut self, world: &crate::world::World, msg: Msg) {
        match msg {
            Msg::BlockChanged(block) => {
                self.set_block(block, world.get(block));
            }
            Msg::PlayerMoved(pos) => {
                let mut chunk = [0; 4];
                for i in 0..4 {
                    chunk[i] =
                        (pos[i] as isize - RENDER_CHUNKS as isize).div_euclid(CHUNK_SIZE as isize);
                }
                self.change_loaded_region(world, chunk);
            }
        }
    }

    pub fn clear_canvas(&self) {
        self.gl_handler.clear_canvas();
    }

    pub fn render(&mut self, world: &crate::world::World, viewport: Viewport, uniforms: Uniforms) {
        if self.mesh_dirty {
            self.redo_mesh(world);
        }
        self.gl_handler.render(viewport, uniforms);
    }

    pub fn redo_mesh(&mut self, world: &crate::world::World) {
        let mut vertex_data: Vec<f32> = Vec::new();

        fn all_or_nothing(blocks: &[crate::block::BlockName]) -> bool {
            blocks.iter().all(|b| *b == crate::block::BlockName::Air)
                || blocks.iter().all(|b| *b != crate::block::BlockName::Air)
        }

        for x in self.corner_of_loaded_region[0] * CHUNK_SIZE as isize
            ..(self.corner_of_loaded_region[0] + RENDER_CHUNKS as isize) * CHUNK_SIZE as isize
        {
            for y in self.corner_of_loaded_region[1] * CHUNK_SIZE as isize
                ..(self.corner_of_loaded_region[1] + RENDER_CHUNKS as isize) * CHUNK_SIZE as isize
            {
                for z in self.corner_of_loaded_region[2] * CHUNK_SIZE as isize
                    ..(self.corner_of_loaded_region[2] + RENDER_CHUNKS as isize)
                        * CHUNK_SIZE as isize
                {
                    for w in self.corner_of_loaded_region[3] * CHUNK_SIZE as isize
                        ..(self.corner_of_loaded_region[3] + RENDER_CHUNKS as isize)
                            * CHUNK_SIZE as isize
                    {
                        if !all_or_nothing(&[
                            world.get([x, y, z, w]),
                            world.get([x, y, z, w - 1]),
                            world.get([x, y, z - 1, w]),
                            world.get([x, y, z - 1, w - 1]),
                        ]) {
                            #[rustfmt::skip]
                            vertex_data.extend_from_slice(&[
                                x as f32     , y as f32     , z as f32, w as f32,
                                x as f32 + 1., y as f32     , z as f32, w as f32,
                                x as f32 + 1., y as f32 + 1., z as f32, w as f32,
                                x as f32 + 1., y as f32 + 1., z as f32, w as f32,
                                x as f32     , y as f32 + 1., z as f32, w as f32,
                                x as f32     , y as f32     , z as f32, w as f32,
                            ]);
                        }

                        if !all_or_nothing(&[
                            world.get([x, y, z, w]),
                            world.get([x, y, z, w - 1]),
                            world.get([x, y - 1, z, w]),
                            world.get([x, y - 1, z, w - 1]),
                        ]) {
                            #[rustfmt::skip]
                            vertex_data.extend_from_slice(&[
                                x as f32     , y as f32, z as f32     , w as f32,
                                x as f32 + 1., y as f32, z as f32     , w as f32,
                                x as f32 + 1., y as f32, z as f32 + 1., w as f32,
                                x as f32 + 1., y as f32, z as f32 + 1., w as f32,
                                x as f32     , y as f32, z as f32 + 1., w as f32,
                                x as f32     , y as f32, z as f32     , w as f32,
                            ]);
                        }

                        if !all_or_nothing(&[
                            world.get([x, y, z, w]),
                            world.get([x, y, z, w - 1]),
                            world.get([x - 1, y, z, w]),
                            world.get([x - 1, y, z, w - 1]),
                        ]) {
                            #[rustfmt::skip]
                            vertex_data.extend_from_slice(&[
                                x as f32, y as f32     , z as f32     , w as f32,
                                x as f32, y as f32 + 1., z as f32     , w as f32,
                                x as f32, y as f32 + 1., z as f32 + 1., w as f32,
                                x as f32, y as f32 + 1., z as f32 + 1., w as f32,
                                x as f32, y as f32     , z as f32 + 1., w as f32,
                                x as f32, y as f32     , z as f32     , w as f32,
                            ]);
                        }

                        if !all_or_nothing(&[
                            world.get([x, y, z, w]),
                            world.get([x, y, z - 1, w]),
                            world.get([x, y - 1, z, w]),
                            world.get([x, y - 1, z - 1, w]),
                        ]) {
                            #[rustfmt::skip]
                            vertex_data.extend_from_slice(&[
                                x as f32     , y as f32 , z as f32, w as f32     ,
                                x as f32 + 1., y as f32 , z as f32, w as f32     ,
                                x as f32 + 1., y as f32 , z as f32, w as f32 + 1.,
                                x as f32 + 1., y as f32 , z as f32, w as f32 + 1.,
                                x as f32     , y as f32 , z as f32, w as f32 + 1.,
                                x as f32     , y as f32 , z as f32, w as f32     ,
                            ]);
                        }

                        if !all_or_nothing(&[
                            world.get([x, y, z, w]),
                            world.get([x, y, z - 1, w]),
                            world.get([x - 1, y, z, w]),
                            world.get([x - 1, y, z - 1, w]),
                        ]) {
                            #[rustfmt::skip]
                            vertex_data.extend_from_slice(&[
                                x as f32, y as f32     , z as f32, w as f32     ,
                                x as f32, y as f32 + 1., z as f32, w as f32     ,
                                x as f32, y as f32 + 1., z as f32, w as f32 + 1.,
                                x as f32, y as f32 + 1., z as f32, w as f32 + 1.,
                                x as f32, y as f32     , z as f32, w as f32 + 1.,
                                x as f32, y as f32     , z as f32, w as f32     ,
                            ]);
                        }

                        if !all_or_nothing(&[
                            world.get([x, y, z, w]),
                            world.get([x, y - 1, z, w]),
                            world.get([x - 1, y, z, w]),
                            world.get([x - 1, y - 1, z, w]),
                        ]) {
                            #[rustfmt::skip]
                            vertex_data.extend_from_slice(&[
                                x as f32, y as f32, z as f32     , w as f32     ,
                                x as f32, y as f32, z as f32 + 1., w as f32     ,
                                x as f32, y as f32, z as f32 + 1., w as f32 + 1.,
                                x as f32, y as f32, z as f32 + 1., w as f32 + 1.,
                                x as f32, y as f32, z as f32     , w as f32 + 1.,
                                x as f32, y as f32, z as f32     , w as f32     ,
                            ]);
                        }
                    }
                }
            }
        }

        self.gl_handler.set_vertex_data(&vertex_data);
        self.mesh_dirty = false;
    }

    fn is_chunk_loaded(&self, chunk: [isize; 4]) -> bool {
        for i in 0..4 {
            if !(0..CHUNK_SIZE as isize).contains(&(chunk[i] - self.corner_of_loaded_region[i])) {
                return false;
            }
        }
        true
    }

    fn load_chunk(&mut self, world: &crate::world::World, chunk: [isize; 4]) {
        if !self.is_chunk_loaded(chunk) {
            world.pass_chunk_as_slice(chunk, |slice| {
                self.gl_handler.set_texture(chunk_texture_loc(chunk), slice)
            });
        }
    }

    fn change_loaded_region(&mut self, world: &crate::world::World, new_corner: [isize; 4]) {
        if new_corner == self.corner_of_loaded_region {
            return;
        }

        for x in 0..RENDER_CHUNKS as isize {
            for y in 0..RENDER_CHUNKS as isize {
                for z in 0..RENDER_CHUNKS as isize {
                    for w in 0..RENDER_CHUNKS as isize {
                        self.load_chunk(
                            world,
                            [
                                new_corner[0] + x,
                                new_corner[1] + y,
                                new_corner[2] + z,
                                new_corner[3] + w,
                            ],
                        );
                    }
                }
            }
        }

        self.corner_of_loaded_region = new_corner;

        self.mesh_dirty = true;
    }

    fn set_block(&mut self, coords: [isize; 4], block: crate::block::BlockName) {
        let (which_chunk, rel_pos) = crate::chunk::chunk_position(coords);
        if self.is_chunk_loaded(which_chunk) {
            self.gl_handler.set_texture_pixel(
                chunk_texture_loc(which_chunk),
                [
                    rel_pos[0] as usize + CHUNK_SIZE * rel_pos[1] as usize,
                    rel_pos[2] as usize + CHUNK_SIZE * rel_pos[3] as usize,
                ],
                block,
            );

            self.mesh_dirty = true;
        }
    }
}

#[rustfmt::skip]
fn chunk_texture_loc(chunk: [isize; 4]) -> usize {
    (chunk[0] as usize & 3) + RENDER_CHUNKS * (
    (chunk[1] as usize & 3) + RENDER_CHUNKS * (
    (chunk[2] as usize & 3) + RENDER_CHUNKS * (
    (chunk[3] as usize & 3))))
}
