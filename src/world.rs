use crate::block::{Block, BlockName};
use crate::chunk::Chunk;

use std::cell::RefCell;
use std::collections::HashMap;

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
        let (which_chunk, rel_pos) = crate::chunk::chunk_position(coords);

        *self
            .chunks
            .borrow_mut()
            .entry(which_chunk)
            .or_insert_with(|| Chunk::new(which_chunk))[rel_pos]
    }

    pub fn get_mut(&mut self, coords: [isize; 4]) -> &mut Block {
        let (which_chunk, rel_pos) = crate::chunk::chunk_position(coords);

        &mut self
            .chunks
            .get_mut()
            .entry(which_chunk)
            .or_insert_with(|| Chunk::new(which_chunk))[rel_pos]
    }

    pub fn pass_chunk_as_slice(&self, which_chunk: [isize; 4], f: impl FnOnce(&[u8])) {
        self.chunks
            .borrow_mut()
            .entry(which_chunk)
            .or_insert_with(|| Chunk::new(which_chunk))
            .pass_as_slice(f)
    }
}
