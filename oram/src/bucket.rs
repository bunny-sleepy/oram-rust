use crate::block::Block;
use std::collections::HashMap;

/// A bucket within the ORAM tree structure that stores blocks by their IDs.
#[derive(Clone, Debug)]
pub struct Bucket {
    pub blocks: HashMap<u32, Block>, // Maps block IDs to blocks for efficient access
    pub capacity: usize,              // Maximum number of blocks per bucket
}

impl Bucket {
    /// Creates a new bucket with a specified capacity.
    pub fn new(capacity: usize) -> Self {
        Bucket {
            blocks: HashMap::new(),
            capacity,
        }
    }

    /// Adds a block to the bucket, respecting the capacity limit.
    /// If the bucket is full, an existing block is removed to make space.
    pub fn add_block(&mut self, block: Block) {
        if self.blocks.len() >= self.capacity {
            // Remove a random block to maintain the bucket capacity
            let first_key = *self.blocks.keys().next().unwrap();
            self.blocks.remove(&first_key);
        }
        self.blocks.insert(block.block_id, block);
    }

    /// Retrieves a block by its ID, if it exists.
    pub fn get_block(&self, block_id: u32) -> Option<&Block> {
        self.blocks.get(&block_id)
    }

    /// Returns all blocks in the bucket as a vector (useful for reading paths).
    pub fn get_all_blocks(&self) -> Vec<Block> {
        self.blocks.values().cloned().collect()
    }

    /// Clears the bucket and refills it with selected blocks up to its capacity.
    pub fn replace_blocks(&mut self, new_blocks: Vec<Block>) {
        self.blocks.clear();
        for block in new_blocks.into_iter().take(self.capacity) {
            self.blocks.insert(block.block_id, block);
        }
    }
}
