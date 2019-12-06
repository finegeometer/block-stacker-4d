use super::block::{Block, BlockName};

const WORLD_SIZE: usize = 8;

pub struct World {
    world: Vec<Block>,

    // This should be removable when I implement infinite worlds.
    fake_block_for_out_of_bounds: Block,
}

impl World {
    pub fn new() -> Self {
        let mut world = Vec::with_capacity(WORLD_SIZE * WORLD_SIZE * WORLD_SIZE * WORLD_SIZE);
        for _ in 0..WORLD_SIZE * WORLD_SIZE * WORLD_SIZE * WORLD_SIZE {
            world.push(Block::new(BlockName::Air));
        }
        Self {
            world,
            fake_block_for_out_of_bounds: Block::new(BlockName::Air),
        }
    }
}

impl std::ops::Index<[isize; 4]> for World {
    type Output = Block;
    fn index(&self, block: [isize; 4]) -> &Block {
        if block.iter().all(|x| *x >= 0 && (*x as usize) < WORLD_SIZE) {
            &self.world[((block[0] as usize * WORLD_SIZE + block[1] as usize) * WORLD_SIZE
                + block[2] as usize)
                * WORLD_SIZE
                + block[3] as usize]
        } else {
            const AIR: Block = Block::new(BlockName::Air);
            &AIR
        }
    }
}

impl std::ops::IndexMut<[isize; 4]> for World {
    fn index_mut(&mut self, block: [isize; 4]) -> &mut Block {
        if block.iter().all(|x| *x >= 0 && (*x as usize) < WORLD_SIZE) {
            &mut self.world[((block[0] as usize * WORLD_SIZE + block[1] as usize) * WORLD_SIZE
                + block[2] as usize)
                * WORLD_SIZE
                + block[3] as usize]
        } else {
            self.fake_block_for_out_of_bounds = Block::new(BlockName::Air);
            &mut self.fake_block_for_out_of_bounds
        }
    }
}
