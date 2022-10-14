pub struct Statistics {
    pub l1_reads: usize,
    pub l1_read_misses: usize,
    pub l1_writes: usize,
    pub l1_write_misses: usize,
    pub l1_write_backs: usize,

    pub l2_reads: usize,
    pub l2_read_misses: usize,
    pub l2_writes: usize,
    pub l2_write_misses: usize,
    pub l2_write_backs: usize,

    pub total_memory_traffic: usize,

    pub l1_prefetches: usize,
    pub l2_prefetches: usize,
    pub l2_reads_from_l1_prefetch: usize,
    pub l2_read_misses_from_l1_prefetch: usize,
}

impl Statistics {
    pub fn new() -> Self {
        Self {
            l1_reads: 0,
            l1_read_misses: 0,
            l1_writes: 0,
            l1_write_misses: 0,
            l1_write_backs: 0,
            l2_reads: 0,
            l2_read_misses: 0,
            l2_writes: 0,
            l2_write_misses: 0,
            l2_write_backs: 0,
            total_memory_traffic: 0,
            l1_prefetches: 0,
            l2_prefetches: 0,
            l2_reads_from_l1_prefetch: 0,
            l2_read_misses_from_l1_prefetch: 0
        }
    }

    pub fn print_stats(self) {
        let l1_miss_rate: f32 = ((self.l1_read_misses + self.l1_write_misses) as f32) /
            ((self.l1_reads + self.l1_writes) as f32);
        let mut l2_miss_rate: f32 = (self.l2_read_misses as f32) / (self.l2_reads as f32);
        if l2_miss_rate.is_nan() {
            l2_miss_rate = 0.0;
        }
        println!("===== Measurements =====");
        println!("a. L1 reads:                   {}", self.l1_reads);
        println!("b. L1 read misses:             {}", self.l1_read_misses);
        println!("c. L1 writes:                  {}", self.l1_writes);
        println!("d. L1 write misses:            {}", self.l1_write_misses);
        println!("e. L1 miss rate:               {:.4}", l1_miss_rate);
        println!("f. L1 writebacks:              {}", self.l1_write_backs);
        println!("g. L1 prefetches:              {}", self.l1_prefetches);
        println!("h. L2 reads (demand):          {}", self.l2_reads);
        println!("i. L2 read misses (demand):    {}", self.l2_read_misses);
        println!("j. L2 reads (prefetch):        {}", self.l2_reads_from_l1_prefetch);
        println!("k. L2 read misses (prefetch):  {}", self.l2_read_misses_from_l1_prefetch);
        println!("l. L2 writes:                  {}", self.l2_writes);
        println!("m. L2 write misses:            {}", self.l2_write_misses);
        println!("n. L2 miss rate:               {:.4}", l2_miss_rate);
        println!("o. L2 writebacks:              {}", self.l2_write_backs);
        println!("p. L2 prefetches:              {}", self.l2_prefetches);
        println!("q. memory traffic:             {}", self.total_memory_traffic);
    }
}
