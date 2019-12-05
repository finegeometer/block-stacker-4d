use super::block::Block;

const WORLD_SIZE: usize = 8;

pub struct World {
    world: [[[[Block; WORLD_SIZE]; WORLD_SIZE]; WORLD_SIZE]; WORLD_SIZE],

    // This should be removable when I implement infinite worlds.
    fake_block_for_out_of_bounds: Block,
}

impl World {
    pub fn new() -> Self {
        Self {
            world: [[[[Block::Air; WORLD_SIZE]; WORLD_SIZE]; WORLD_SIZE]; WORLD_SIZE],
            fake_block_for_out_of_bounds: Block::Air,
        }
    }
}

impl std::ops::Index<[isize; 4]> for World {
    type Output = Block;
    fn index(&self, [x, y, z, w]: [isize; 4]) -> &Block {
        self.world
            .get(x as usize)
            .and_then(|thing| thing.get(y as usize))
            .and_then(|thing| thing.get(z as usize))
            .and_then(|thing| thing.get(w as usize))
            .unwrap_or(&Block::Air)
    }
}

impl std::ops::IndexMut<[isize; 4]> for World {
    fn index_mut(&mut self, [x, y, z, w]: [isize; 4]) -> &mut Block {
        self.fake_block_for_out_of_bounds = Block::Air;
        self.world
            .get_mut(x as usize)
            .and_then(|thing| thing.get_mut(y as usize))
            .and_then(|thing| thing.get_mut(z as usize))
            .and_then(|thing| thing.get_mut(w as usize))
            .unwrap_or(&mut self.fake_block_for_out_of_bounds)
    }
}
