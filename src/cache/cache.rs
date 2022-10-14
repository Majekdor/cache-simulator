use fast_math::log2;
use crate::cache::block::Block;

pub struct Cache {
    pub cache_size: usize,
    pub assoc: usize,
    pub block_size: usize,
    pub sets: usize,
    pub index_bits: usize,
    pub block_offset_bits: usize,
    pub tag_bits: usize,
    pub cache: Vec<Vec<Block>>,
}

#[derive(PartialEq)]
pub enum HitOrMiss {
    HIT,
    MISS,
}

pub struct EvictionResult {
    pub evicted_block_address: usize,
    pub evicted_block_was_dirty: bool,
}

impl Cache {
    /// Creates a new cache with the given constraints.
    ///
    /// ## Arguments
    /// * `cache_size` - The total size of the cache.
    /// * `assoc` - The associativity of the cache.
    /// * `block_size` - The size of the blocks in the cache.
    ///
    /// ## Example
    /// ```rust
    /// let mut l1 = Cache::new(1024, 2, 32);
    /// ```
    pub fn new(
        cache_size: usize,
        assoc: usize,
        block_size: usize,
    ) -> Self {
        if cache_size == 0 {
            return Self {
                cache_size: 0,
                assoc: 0,
                block_size: 0,
                sets: 0,
                index_bits: 0,
                block_offset_bits: 0,
                tag_bits: 0,
                cache: vec![]
            }
        }
        let sets = cache_size / (assoc * block_size);
        let index_bits = log2(sets as f32) as usize;
        let block_offset_bits = log2(block_size as f32) as usize;
        let tag_bits = 32 - index_bits - block_offset_bits;

        // resize the cache
        let mut cache: Vec<Vec<Block>> = vec![];
        cache.resize(sets, Vec::new());
        for i in 0..sets {
            cache[i].resize(assoc, Block::new());
            // set the lru values to all be different
            for j in 0..assoc {
                cache[i][j].lru = j;
            }
        }

        Self {
            cache_size,
            assoc,
            block_size,
            sets,
            index_bits,
            block_offset_bits,
            tag_bits,
            cache,
        }
    }

    /// Print out information for the entire cache.
    ///
    /// ## Example
    /// ```
    /// set    1:   824721 D  948241
    /// set    2:   824721 D  948241
    /// ```
    pub fn print_cache_info(&self) {
        for i in 0..self.sets {
            print!("set    ");
            if i < 100 {
                print!(" ");
            }
            if i < 10 {
                print!(" ");
            }
            print!("{}: ", i);

            let mut set: Vec<Block> = self.cache[i].clone();
            set.sort_by(|a, b| a.lru.partial_cmp(&b.lru).unwrap());

            for j in 0..self.assoc {
                print!("  ");
                print!("{number:>6x}", number=set[j].tag);
                if set[j].dirty {
                    print!(" D");
                } else {
                    print!("  ");
                }
            }
            println!();
        }
    }

    /// Try to read from the cache given the index and tag of the block.
    ///
    /// ## Arguments
    /// * `index` - The index of the desired block.
    /// * `tag` - The tag of the desired block.
    ///
    /// Returns whether the block was in the cache (hit) or not (miss).
    pub fn read(&mut self, index: usize, tag: usize) -> HitOrMiss {
        for i in 0..self.assoc {
            // L1 Hit if tags are equal and location is valid
            if self.cache[index][i].tag == tag && self.cache[index][i].valid {
                self.update_lru(index, tag);
                return HitOrMiss::HIT;
            }
        }
        return HitOrMiss::MISS;
    }

    /// Try to write to the cache given the index and tag of the block.
    ///
    /// ## Arguments
    /// * `index` - The index of the desired block.
    /// * `tag` - The tag of the desired block.
    ///
    /// Returns whether the block was written to in the cache (hit) or not (miss).
    pub fn write(&mut self, index: usize, tag: usize) -> HitOrMiss {
        let mut written = false;
        for i in 0..self.assoc {
            if self.cache[index][i].tag == tag {
                self.cache[index][i].valid = true;
                self.update_lru(index, tag);
                self.cache[index][i].dirty = true;
                written = true;
                break;
            }
        }
        return if written { HitOrMiss::HIT } else { HitOrMiss::MISS };
    }

    /// Install a block in the cache given the index, tag and address of the block.
    ///
    /// ## Arguments
    /// * `index` - The index of the block to install.
    /// * `tag` - The tag of the block to install.
    /// * `address` - The address of the block to install.
    ///
    /// ## Throws
    /// This function will panic if there is no room to install in the set.
    /// That should be handled before installing.
    pub fn install(&mut self, index: usize, tag: usize, address: usize) {
        let mut installed = false;
        for i in 0..self.assoc {
            // Found an invalid block, install
            if !self.cache[index][i].valid {
                self.cache[index][i].address = address;
                self.cache[index][i].tag = tag;
                self.cache[index][i].valid = true;
                self.update_lru(index, tag);
                installed = true;
                break;
            }
        }

        if !installed {
            panic!("Tried to install where there was no free space.");
        }
    }

    /// Update the recency values of all blocks in a set.
    /// This is called after reading, writing, and installing.
    ///
    /// ## Arguments
    /// * `index` - The index (or set) to update.
    /// * `tag` - The tag of the block that was just accessed.
    pub fn update_lru(&mut self, index: usize, tag: usize) {
        let mut new_mru_way: usize = 0;
        for i in 0..self.assoc {
            if self.cache[index][i].tag == tag {
                new_mru_way = i;
            }
        }

        for i in 0..self.assoc {
            if i != new_mru_way && self.cache[index][i].lru < self.cache[index][new_mru_way].lru {
                self.cache[index][i].lru += 1;
            }
        }
        self.cache[index][new_mru_way].lru = 0;
    }

    /// Check whether a set is full.
    ///
    /// ## Arguments
    /// * `index` - The index (or set) to check.
    ///
    /// Returns whether the set is full.
    pub fn set_is_full(&self, index:usize) -> bool {
        for i in 0..self.assoc {
            if !self.cache[index][i].valid {
                return false;
            }
        }
        return true;
    }

    /// Evict the block that was accessed least recently.
    ///
    /// ## Arguments
    /// * `index` - The index (or set) to evict a block from.
    ///
    /// Returns an eviction result, containing the evicted block's address and whether
    /// the block was dirty (meaning it needs to be written back).
    pub fn evict_lru_block(&mut self, index: usize) -> EvictionResult {
        let mut block_to_evict_index: usize = 0;
        let mut lru_value: usize = 0;
        // find least recently used
        for i in 0..self.assoc {
            if self.cache[index][i].lru > lru_value {
                lru_value = self.cache[index][i].lru;
                block_to_evict_index = i;
            }
        }
        // set the valid bit false so we know we can write to it
        // TODO: Not sure we're supposed to do this, but it should work for my impl
        self.cache[index][block_to_evict_index].valid = false;
        let was_dirty = self.cache[index][block_to_evict_index].dirty;
        self.cache[index][block_to_evict_index].dirty = false;
        // return the evicted block tag
        return EvictionResult {
            evicted_block_address: self.cache[index][block_to_evict_index].address,
            evicted_block_was_dirty: was_dirty,
        };
    }
}