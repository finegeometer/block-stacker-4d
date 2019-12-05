#[derive(Copy, Clone)]
pub enum Block {
    Air,
    Solid { color: [u8; 3] },
}

impl Block {
    pub fn is_transparent(&self) -> bool {
        match self {
            Self::Air => true,
            _ => false,
        }
    }

    pub fn color(&self) -> [u8; 4] {
        match self {
            Self::Air => [0, 0, 0, 0],
            Self::Solid { color: [r, g, b] } => [*r, *g, *b, 0xFF],
        }
    }
}
