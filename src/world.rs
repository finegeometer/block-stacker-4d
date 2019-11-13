mod renderer;
pub use renderer::View;

const WORLD_SIZE: usize = 8;

type Block = Option<[u8; 3]>;

pub struct World {
    //
    renderer: renderer::Renderer,

    world: [[[[Block; WORLD_SIZE]; WORLD_SIZE]; WORLD_SIZE]; WORLD_SIZE],

    mesh_dirty: bool,
}

impl World {
    pub fn new(gl: web_sys::WebGl2RenderingContext) -> Self {
        Self {
            renderer: renderer::Renderer::new(gl),
            world: [[[[None; WORLD_SIZE]; WORLD_SIZE]; WORLD_SIZE]; WORLD_SIZE],
            mesh_dirty: false,
        }
    }

    pub fn set_block(&mut self, coords: [i32; 4], block: Block) {
        self.renderer.set_world_pixel(
            coords,
            if let Some([x, y, z]) = block {
                [x, y, z, 0xFF]
            } else {
                [0xFF, 0xFF, 0xFF, 0x00]
            },
        );

        (|| -> Option<_> {
            use std::convert::TryFrom;
            let reference: &mut Block = self
                .world
                .get_mut(usize::try_from(coords[0]).ok()?)?
                .get_mut(usize::try_from(coords[1]).ok()?)?
                .get_mut(usize::try_from(coords[2]).ok()?)?
                .get_mut(usize::try_from(coords[3]).ok()?)?;

            if reference.is_some() != block.is_some() {
                self.mesh_dirty = true;
            }

            *reference = block;

            Some(())
        })();
    }

    pub fn get_block(&self, coords: [i32; 4]) -> Block {
        use std::convert::TryFrom;
        *self
            .world
            .get(usize::try_from(coords[0]).ok()?)?
            .get(usize::try_from(coords[1]).ok()?)?
            .get(usize::try_from(coords[2]).ok()?)?
            .get(usize::try_from(coords[3]).ok()?)?
    }

    pub fn render(&mut self, four_camera: nalgebra::Matrix5<f32>, views: Vec<View>) {
        if self.mesh_dirty {
            let mut vertex_data: Vec<f32> = Vec::new();

            fn all_or_nothing(blocks: &[Block]) -> bool {
                blocks.iter().all(Option::is_some) || blocks.iter().all(Option::is_none)
            }

            for i in 0..WORLD_SIZE as i32 {
                for j in 0..WORLD_SIZE as i32 {
                    for k in 0..=WORLD_SIZE as i32 {
                        for l in 0..=WORLD_SIZE as i32 {
                            if !all_or_nothing(&[
                                self.get_block([i, j, k, l]),
                                self.get_block([i, j, k - 1, l]),
                                self.get_block([i, j, k, l - 1]),
                                self.get_block([i, j, k - 1, l - 1]),
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
                                self.get_block([i, k, j, l]),
                                self.get_block([i, k - 1, j, l]),
                                self.get_block([i, k, j, l - 1]),
                                self.get_block([i, k - 1, j, l - 1]),
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
                                self.get_block([k, i, j, l]),
                                self.get_block([k - 1, i, j, l]),
                                self.get_block([k, i, j, l - 1]),
                                self.get_block([k - 1, i, j, l - 1]),
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
                                self.get_block([i, k, l, j]),
                                self.get_block([i, k - 1, l, j]),
                                self.get_block([i, k, l - 1, j]),
                                self.get_block([i, k - 1, l - 1, j]),
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
                                self.get_block([k, i, l, j]),
                                self.get_block([k - 1, i, l, j]),
                                self.get_block([k, i, l - 1, j]),
                                self.get_block([k - 1, i, l - 1, j]),
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
                                self.get_block([k, l, i, j]),
                                self.get_block([k - 1, l, i, j]),
                                self.get_block([k, l - 1, i, j]),
                                self.get_block([k - 1, l - 1, i, j]),
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

            self.renderer.set_vertex_data(&vertex_data);
            self.mesh_dirty = false;
        }
        self.renderer.render(four_camera, views);
    }

    // Note: A translation of this to GLSL is in the fragment shader.
    pub fn raycast(
        &self,
        pos: nalgebra::Vector4<f32>,
        dir: nalgebra::Vector4<f32>,
        ray_distance: f32,
    ) -> (Option<[i32; 4]>, Option<[i32; 4]>) {
        let t_max = ray_distance / dir.norm();

        let t_steps = {
            let mut out: [f32; 4] = dir.into();
            for thing in out.iter_mut() {
                *thing = thing.recip().abs();
            }
            out
        };

        let mut next_ts = {
            let mut out = t_steps;
            for i in 0..4 {
                let tmp = -pos[i] * dir[i].signum();
                out[i] *= tmp - tmp.floor();
            }
            out
        };

        let block_steps = {
            let mut out = [0; 4];
            for i in 0..4 {
                out[i] = dir[i].signum() as i32;
            }
            out
        };

        let mut current_block = None;
        let mut next_block = {
            let mut out = [0; 4];
            for i in 0..4 {
                out[i] = pos[i].floor() as i32;
            }
            out
        };

        let mut t = 0.0;

        while t < t_max {
            if self.get_block(next_block).is_some() {
                return (current_block, Some(next_block));
            }
            current_block = Some(next_block);

            let i: usize = next_ts
                .iter()
                .filter(|x| x.is_finite())
                .enumerate()
                .min_by(|(_, t1), (_, t2)| t1.partial_cmp(t2).expect("NAN"))
                .unwrap()
                .0;

            next_block[i] += block_steps[i];
            t = next_ts[i];
            next_ts[i] += t_steps[i];
        }

        (None, None)
    }
}
