#[derive(Clone)]
pub struct Block {
    pub address: usize,
    pub tag: usize,
    pub lru: usize,
    pub valid: bool,
    pub dirty: bool,
}

impl Block {
    pub fn new() -> Self {
        Block {
            address: 0,
            tag: 0,
            lru: 0,
            valid: false,
            dirty: false
        }
    }
}
