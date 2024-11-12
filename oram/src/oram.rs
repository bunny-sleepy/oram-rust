use crate::{block::Block, bucket::Bucket};
use rand::Rng;
use std::collections::HashMap;
#[cfg(feature = "parallel")]
use rayon::prelude::*;

/// Structure for the Path ORAM
pub struct PathORAM {
    pub(crate) tree: Vec<Bucket>,
    pub(crate) position_map: HashMap<u32, u32>, // Maps each block to a position in the tree
    pub stash: HashMap<u32, Block>,      // Stash for blocks that couldn't be evicted
    pub(crate) capacity: usize,                 // Number of blocks per bucket
    pub(crate) tree_height: u32,                // Height of the ORAM tree
}

impl PathORAM {
    pub fn new(num_blocks: u32, capacity: usize) -> Self {
        // Step 1: Calculate the tree height and number of buckets
        let tree_height = (num_blocks as f64).log2().ceil() as u32; // L = ⌈log2(N)⌉
        let num_buckets = (1 << (tree_height + 1)) - 1; // Total buckets for a complete binary tree

        // Step 2: Initialize the ORAM tree with buckets filled with dummy blocks
        let mut tree = Vec::with_capacity(num_buckets as usize);

        #[cfg(feature = "parallel")]
        for _ in 0..num_buckets {
            let mut bucket = Bucket::new(capacity);
            // Fill the bucket with dummy blocks
            for _ in 0..capacity {
                bucket.add_block(Block { block_id: 0, data: 0 });
            }
            tree.push(bucket);
        }

        // Step 3: Initialize the position map with random positions
        let mut position_map = HashMap::new();
        let mut rng = rand::thread_rng();
        #[cfg(feature = "parallel")]
        for block_id in 1..=num_blocks {
            let random_position = rng.gen_range(0..(1 << tree_height));
            position_map.insert(block_id, random_position);
        }

        // Step 4: Initialize PathORAM with the populated tree, position map, and stash
        let mut oram = PathORAM {
            tree,
            position_map,
            stash: HashMap::new(),
            capacity,
            tree_height,
        };

        for block_id in 1..=num_blocks {
            oram.access("write", block_id, Some(0));
        }

        oram
    }

    pub fn access(&mut self, op: &str, block_id: u32, new_data: Option<u32>) -> Option<u32> {
        let mut rng = rand::thread_rng();

        // Step 1: Retrieve the current position of the block and generate a new position
        let old_position = *self.position_map.get(&block_id).unwrap();  // x in the algorithm
        let new_position = rng.gen_range(0..(1 << self.tree_height));   // New position for the block

        // Step 2: Read the path from root to leaf based on the old position
        for level in 0..=self.tree_height {
            let index = self.get_bucket_index(old_position, level);
            println!("Reading bucket at index {}", index);
            for block in self.tree[index].get_all_blocks() {
                if block.block_id != 0 { // Skip dummy blocks
                    self.stash.insert(block.block_id, block);
                }
            }
        }

        println!("Stash size: {}", self.stash.len());

        // Step 3: Retrieve or update the block data in the stash
        let mut data = self.stash.get(&block_id).map(|block| block.data);
        if op == "write" {
            let new_data_value = new_data.unwrap();
            println!("Writing block {} with data {} to position {}", block_id, new_data_value, new_position);
            self.stash.insert(block_id, Block {
                block_id,
                data: new_data_value,
            });
        }

        // Step 4: Update position map and attempt to write path back to the tree
        self.position_map.insert(block_id, new_position); // Update to new position

        for level in (0..=self.tree_height).rev() {
            let index = self.get_bucket_index(old_position, level);

            // Select blocks to write back to the current bucket
            let mut selected_blocks = Vec::with_capacity(self.capacity);
            for block in self.stash.values() {
                if let Some(&pos) = self.position_map.get(&block.block_id) {
                    if self.get_bucket_index(pos, level) == index {
                        selected_blocks.push(block.clone());

                        if selected_blocks.len() >= self.capacity {
                            break;
                        }
                    }
                }
            }

            // Remove selected blocks from the stash
            for block in &selected_blocks {
                self.stash.remove(&block.block_id);
            }

            // Pad with dummy blocks if needed
            while selected_blocks.len() < self.capacity {
                selected_blocks.push(Block { block_id: 0, data: 0 });
            }

            // Replace the bucket's contents with the selected blocks
            self.tree[index].replace_blocks(selected_blocks);
        }

        data
    }

    /// Helper function to calculate the bucket index at a given level for a specific leaf
    fn get_bucket_index(&self, leaf: u32, level: u32) -> usize {
        let path = leaf >> (self.tree_height - level);
        ((1 << level) - 1 + path) as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;

    #[test]
    fn test_basic_write_and_read() {
        let num_blocks = 4;
        let bucket_capacity = 4;
        let mut oram = PathORAM::new(num_blocks, bucket_capacity);

        let block_id = 1;
        let new_data = 42;
        oram.access("write", block_id, Some(new_data));

        let read_data = oram.access("read", block_id, None).unwrap();
        assert_eq!(read_data, new_data);
    }

    #[test]
    fn test_stash_grows_unbounded() {
        let num_blocks = 1 << 5;
        let bucket_capacity = 2;
        let warmup_accesses = 10000;
        let total_accesses = 10000 + warmup_accesses;
    
        // Initialize the ORAM
        let mut oram = PathORAM::new(num_blocks, bucket_capacity);
    
        // Stash statistics: map each stash size to the number of accesses where it was strictly > that size
        let mut stash_sizes: Vec<u32> = vec![0; total_accesses as usize];
    
        // Access the ORAM sequentially and log stash sizes after warmup period
        for access_count in 0..total_accesses {
            let block_id = (access_count % num_blocks) + 1;
            oram.access("read", block_id, None);

            // Start collecting stash size statistics after warmup period
            if access_count >= warmup_accesses {
                let stash_size = oram.stash.len();
                stash_sizes[stash_size] += 1;
            }
        }

        // Calculate the stash size data to write to file
        let mut stash_data: Vec<(i32, u32)> = Vec::new();
        let mut running_sum = stash_sizes.iter().sum::<u32>();
        stash_data.push((-1_i32, total_accesses - warmup_accesses)); // First line: -1, total number of accesses
    
        for i in 0..(stash_sizes.len() as i32) {
            stash_data.push((i, running_sum));
            running_sum -= stash_sizes[i as usize];
        }

        for (i, count) in stash_sizes.iter().enumerate() {
            if count > &0 {
                println!("{},{}", i, count);
            }
        }

        for (i, count) in stash_data {
            if count > 0 {
                println!("{},{}", i, count);
            }
        }
    }
}
