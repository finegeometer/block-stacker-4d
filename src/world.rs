use super::block::{Block, BlockName};

use std::cell::RefCell;
use std::collections::HashMap;

const CHUNK_SIZE: usize = 8;

struct Chunk(Vec<Block>);

impl Chunk {
    fn new() -> Self {
        let mut chunk = Vec::with_capacity(CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE);
        for _ in 0..CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE {
            chunk.push(Block::new(BlockName::Air));
        }
        Self(chunk)
    }
}

pub struct World {
    chunks: RefCell<HashMap<[isize; 4], Chunk>>,
}

impl World {
    pub fn new() -> Self {
        Self {
            chunks: RefCell::new(HashMap::new()),
        }
    }

    pub fn get(&self, coords: [isize; 4]) -> BlockName {
        let (which_chunk, rel_pos) = chunk_position(coords);

        #[rustfmt::skip]
        let chunk_index =
            rel_pos[0] as usize + CHUNK_SIZE * (
            rel_pos[1] as usize + CHUNK_SIZE * (
            rel_pos[2] as usize + CHUNK_SIZE * (
            rel_pos[3] as usize)));

        *self
            .chunks
            .borrow_mut()
            .entry(which_chunk)
            .or_insert_with(|| Chunk::new())
            .0[chunk_index]
    }

    pub fn get_mut(&mut self, coords: [isize; 4]) -> &mut Block {
        let (which_chunk, rel_pos) = chunk_position(coords);

        #[rustfmt::skip]
        let chunk_index =
            rel_pos[0] as usize + CHUNK_SIZE * (
            rel_pos[1] as usize + CHUNK_SIZE * (
            rel_pos[2] as usize + CHUNK_SIZE * (
            rel_pos[3] as usize)));

        &mut self
            .chunks
            .get_mut()
            .entry(which_chunk)
            .or_insert_with(|| Chunk::new())
            .0[chunk_index]
    }
}

fn chunk_position(coords: [isize; 4]) -> ([isize; 4], [isize; 4]) {
    let mut chunk = [0; 4];
    let mut rel_pos = [0; 4];
    for i in 0..4 {
        chunk[i] = coords[i].div_euclid(CHUNK_SIZE as isize);
        rel_pos[i] = coords[i].rem_euclid(CHUNK_SIZE as isize);
    }
    (chunk, rel_pos)
}
