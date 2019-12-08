use crate::block::{Block, BlockName};

// Changing this constant requires changing the fragment shader.
pub const CHUNK_SIZE: usize = 8;

pub struct Chunk(Vec<Block>);

impl Chunk {
    pub fn new(which_chunk: [isize; 4]) -> Self {
        let mut chunk = Vec::with_capacity(CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE);
        for w in 0..CHUNK_SIZE as isize {
            for z in 0..CHUNK_SIZE as isize {
                for y in 0..CHUNK_SIZE as isize {
                    for x in 0..CHUNK_SIZE as isize {
                        chunk.push(generate_block([
                            which_chunk[0] * CHUNK_SIZE as isize + x,
                            which_chunk[1] * CHUNK_SIZE as isize + y,
                            which_chunk[2] * CHUNK_SIZE as isize + z,
                            which_chunk[3] * CHUNK_SIZE as isize + w,
                        ]))
                    }
                }
            }
        }
        Self(chunk)
    }

    pub fn pass_as_slice(&self, f: impl FnOnce(&[u8])) {
        f(&self
            .0
            .iter()
            .map(|b| *(b as &BlockName) as u8)
            .collect::<Vec<u8>>())
    }
}

impl std::ops::Index<[isize; 4]> for Chunk {
    type Output = Block;
    fn index(&self, [x, y, z, w]: [isize; 4]) -> &Block {
        #[rustfmt::skip]
        let chunk_index =
            x as usize + CHUNK_SIZE * (
            y as usize + CHUNK_SIZE * (
            z as usize + CHUNK_SIZE * (
            w as usize)));
        &self.0[chunk_index]
    }
}

impl std::ops::IndexMut<[isize; 4]> for Chunk {
    fn index_mut(&mut self, [x, y, z, w]: [isize; 4]) -> &mut Block {
        #[rustfmt::skip]
        let chunk_index =
            x as usize + CHUNK_SIZE * (
            y as usize + CHUNK_SIZE * (
            z as usize + CHUNK_SIZE * (
            w as usize)));
        &mut self.0[chunk_index]
    }
}

pub fn chunk_position(coords: [isize; 4]) -> ([isize; 4], [isize; 4]) {
    let mut chunk = [0; 4];
    let mut rel_pos = [0; 4];
    for i in 0..4 {
        chunk[i] = coords[i].div_euclid(CHUNK_SIZE as isize);
        rel_pos[i] = coords[i].rem_euclid(CHUNK_SIZE as isize);
    }
    (chunk, rel_pos)
}

fn generate_block(block: [isize; 4]) -> Block {
    if block[0] == 0 && block[1] == 0 {
        Block::create(BlockName::Grass)
    } else if block[2] == 0 && block[3] == 0 {
        Block::create(BlockName::Stone)
    } else {
        Block::create(BlockName::Air)
    }
}
