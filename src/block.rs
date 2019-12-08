// Intentionally does not implement Copy, as that would permit duplication glitches.
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum BlockName {
    Air = 0,
    Stone = 1,
    Grass = 2,
}

#[must_use = "Every time you drop this, an actual in-game block gets destroyed. If this is what you want, drop this explicitly."]
pub struct Block(BlockName);

impl std::ops::Deref for Block {
    type Target = BlockName;
    fn deref(&self) -> &BlockName {
        &self.0
    }
}

impl PartialEq<BlockName> for Block {
    fn eq(&self, other: &BlockName) -> bool {
        self as &BlockName == other
    }
}

impl Block {
    pub const fn create(block: BlockName) -> Self {
        Self(block)
    }
}
