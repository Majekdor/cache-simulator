use std::env;
use crate::cache::cache::{Cache, HitOrMiss};
use crate::HitOrMiss::HIT;
use crate::HitOrMiss::MISS;
use crate::statistics::Statistics;

mod cache;
mod statistics;

const READ: char = 'r';
const WRITE: char = 'w';

fn main() {
    // get command line arguments and fail if insufficient
    let args: Vec<String> = env::args().collect();
    if args.len() != 7 {
        panic!("Expected 6 arguments and got {}", args.len());
    }

    // parse command line arguments
    let args = Args {
        block_size: args[1].parse().unwrap(),
        l1_size: args[2].parse().unwrap(),
        l1_assoc: args[3].parse().unwrap(),
        l2_size: args[4].parse().unwrap(),
        l2_assoc: args[5].parse().unwrap(),
        trace: args[6].clone(),
    };

    // print simulator configuration
    println!("===== Simulator configuration =====");
    println!("BLOCK SIZE:  {}", args.block_size);
    println!("L1_SIZE:     {}", args.l1_size);
    println!("L1_ASSOC:    {}", args.l1_assoc);
    println!("L2_SIZE:     {}", args.l2_size);
    println!("L2_ASSOC:    {}", args.l2_assoc);
    println!("trace_file:  {}", args.trace);

    // initialize statistics
    let mut stats = Statistics::new();

    // create caches
    let mut l1 = Cache::new(args.l1_size, args.l1_assoc, args.block_size);
    let mut l2 = Cache::new(args.l2_size, args.l2_assoc, args.block_size);

    // read every line from trace file
    let contents = std::fs::read_to_string("trace.txt").expect("File not found!");
    for line in contents.lines() {
        // get instruction and address
        let parts: Vec<String> = line.split(" ").map(|s| s.to_string()).collect();
        let rw: char = parts.get(0).unwrap().chars().next().unwrap();
        let address = parts.get(1).unwrap();
        let address_usize: usize = usize::from_str_radix(address, 16).unwrap();
        let address_binary_string = format!("{:032b}", address_usize);

        // if it's not read or write, fail
        if rw != READ && rw != WRITE {
            panic!("Unknown action {}", rw);
        }

        // get the index and tag for l1 cache
        let l1_index: usize = usize::from_str_radix(
            &address_binary_string
                .chars()
                .skip(l1.tag_bits)
                .take(l1.index_bits)
                .collect::<String>(),
            2
        ).unwrap();
        let l1_tag: usize = usize::from_str_radix(
            &address_binary_string
                .chars()
                .take(l1.tag_bits)
                .collect::<String>(),
            2
        ).unwrap();
        //println!("l1_tag={}", l1_tag);

        // get the index and tag for l2 cache
        let l2_index: usize = usize::from_str_radix(
            &address_binary_string
                .chars()
                .skip(l2.tag_bits)
                .take(l2.index_bits)
                .collect::<String>(),
            2
        ).unwrap_or(0);
        let l2_tag: usize = usize::from_str_radix(
            &address_binary_string
                .chars()
                .take(l2.tag_bits)
                .collect::<String>(),
            2
        ).unwrap_or(0);

        // try to read from l1
        let l1_hit_or_miss =
            if rw == READ {
                l1.read(l1_index, l1_tag)
            } else {
                l1.write(l1_index, l1_tag)
            };
        if l1_hit_or_miss == HIT {
            // we hit in l1
            if rw == READ {
                stats.l1_reads += 1;
            } else if rw == WRITE {
                stats.l1_writes += 1;
            }
        } else if l1_hit_or_miss == MISS {
            // we missed in l1
            if rw == READ {
                stats.l1_read_misses += 1;
            } else if rw == WRITE {
                stats.l1_write_misses += 1;
            }

            // check if we need to evict a block before inserting
            if l1.set_is_full(l1_index) {
                let l1_evicted_result = l1.evict_lru_block(l1_index);

                // check if we have an l2
                if l2.cache_size != 0 {
                    // if the block was dirty we need to perform a write back
                    if l1_evicted_result.evicted_block_was_dirty {
                        let evicted_block_address = format!(
                            "{:032b}",
                            l1_evicted_result.evicted_block_address
                        );
                        let l2_write_back_index: usize = usize::from_str_radix(
                            &evicted_block_address
                                .chars()
                                .skip(l2.tag_bits)
                                .take(l2.index_bits)
                                .collect::<String>(),
                            2
                        ).unwrap_or(0);
                        let l2_write_back_tag: usize = usize::from_str_radix(
                            &evicted_block_address
                                .chars()
                                .take(l2.tag_bits)
                                .collect::<String>(),
                            2
                        ).unwrap_or(0);

                        // try to write back to l2
                        let l2_hit_or_miss = l2
                            .write(
                                l2_write_back_index,
                                l2_write_back_tag
                            );
                        if l2_hit_or_miss == MISS {
                            stats.l2_write_misses += 1;

                            // check if we need to evict a block from l2 before installing
                            if l2.set_is_full(l2_write_back_index) {
                                let l2_evicted_result = l2
                                    .evict_lru_block(l2_write_back_index);

                                // write evicted block to main memory if it was dirty
                                if l2_evicted_result.evicted_block_was_dirty {
                                    stats.l2_write_backs += 1;
                                    stats.total_memory_traffic += 1;
                                }
                            }

                            l2.install(
                                l2_write_back_index,
                                l2_write_back_tag,
                                address_usize
                            );
                            stats.total_memory_traffic += 1;
                        }

                        stats.l1_write_backs += 1;
                        stats.l2_writes += 1;
                    }
                } else {
                    // no l2, write back to main memory if dirty
                    if l1_evicted_result.evicted_block_was_dirty {
                        stats.l1_write_backs += 1;
                        stats.total_memory_traffic += 1;
                    }
                }
            }

            // check if we have an l2
            if l2.cache_size != 0 {
                // try to read block from l2
                let l2_hit_or_miss = l2.read(l2_index, l2_tag);
                if l2_hit_or_miss == HIT {
                    stats.l2_reads += 1;

                    // not in l1 but is in l2, install it in l1
                    l1.install(l1_index, l1_tag, address_usize);
                    if rw == READ {
                        stats.l1_reads += 1;
                    } else if rw == WRITE {
                        l1.write(l1_index, l1_tag);
                        stats.l1_writes += 1;
                    }
                } else if l2_hit_or_miss == MISS {
                    stats.l2_read_misses += 1;

                    // check if we need to evict a block before installing
                    if l2.set_is_full(l2_index) {
                        let l2_evicted_result = l2.evict_lru_block(l2_index);

                        // write evicted block back to main memory if it was dirty
                        if l2_evicted_result.evicted_block_was_dirty {
                            stats.l2_write_backs += 1;
                            stats.total_memory_traffic += 1;
                        }
                    }

                    // install in l2
                    l2.install(l2_index, l2_tag, address_usize);
                    stats.total_memory_traffic += 1;
                    stats.l2_reads += 1;

                    // install in l1
                    l1.install(l1_index, l1_tag, address_usize);
                    if rw == READ {
                        stats.l1_reads += 1;
                    } else {
                        l1.write(l1_index, l1_tag);
                        stats.l1_writes += 1;
                    }
                }
            } else {
                // install block from main memory
                l1.install(l1_index, l1_tag, address_usize);
                if rw == WRITE {
                    l1.write(l1_index, l1_tag);
                }
                stats.total_memory_traffic += 1;
                if rw == READ {
                    stats.l1_reads += 1;
                } else if rw == WRITE {
                    stats.l1_writes += 1;
                }
            }
        }
    }

    // print results

    println!("===== L1 contents =====");
    l1.print_cache_info();

    if l2.cache_size != 0 {
        println!("===== L2 contents =====");
        l2.print_cache_info();
    }

    stats.print_stats();
}

/// Command line arguments needed to run the simulator.
struct Args {
    block_size: usize,
    l1_size: usize,
    l1_assoc: usize,
    l2_size: usize,
    l2_assoc: usize,
    trace: String,
}
