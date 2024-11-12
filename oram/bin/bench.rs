use std::fs::File;
use std::io::{self, Write};
use oram::oram::PathORAM;
use std::env;

fn run_oram_benchmark(num_blocks: u32, bucket_capacity: usize, block_size: usize) -> io::Result<()> {
    let warmup_accesses = 1_000_000;
    let total_accesses = 1_000_000 + warmup_accesses;

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

    // Write the stash size data to a text file
    let mut file = File::create(format!("stash_data_N{}_Z{}_B{}.txt", num_blocks, bucket_capacity, block_size))?;
    for (i, count) in stash_data {
        if count > 0 {
            writeln!(file, "{},{}", i, count)?;
        }
    }

    Ok(())
}


fn main() -> io::Result<()> {
    // Parse command-line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <bucket_capacity>", args[0]);
        std::process::exit(1);
    }

    // Parse the bucket capacity from the command line
    let bucket_capacity: usize = args[1].parse().expect("Invalid bucket capacity");

    println!("Running ORAM benchmark with bucket capacity {}", bucket_capacity);

    // Run benchmarks for each configuration with the specified bucket capacity
    run_oram_benchmark(1 << 20, bucket_capacity, 32)?;
    println!("Finished benchmark with capacity {}", bucket_capacity);

    Ok(())
}
