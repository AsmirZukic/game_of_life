//! Performance benchmark comparing all implementations

use std::time::Instant;
use game_of_life::domain::{Grid, BitGrid, simd_life, temporal_blocking, ConwayRule};

fn benchmark_original_grid(size: usize, iterations: u32) -> f64 {
    let rule = ConwayRule;
    let mut grid = Grid::new(size, size).randomize();
    
    let start = Instant::now();
    for _ in 0..iterations {
        grid = grid.evolve_parallel(&rule);
    }
    start.elapsed().as_secs_f64() * 1000.0 / iterations as f64
}

fn benchmark_bit_grid_naive(size: usize, iterations: u32) -> f64 {
    let rule = ConwayRule;
    let mut grid = BitGrid::new(size, size);
    grid.randomize();
    
    let start = Instant::now();
    for _ in 0..iterations {
        grid = grid.evolve_parallel(&rule);
    }
    start.elapsed().as_secs_f64() * 1000.0 / iterations as f64
}

fn benchmark_bit_grid_simd(size: usize, iterations: u32) -> f64 {
    let rule = ConwayRule;
    let mut grid = BitGrid::new(size, size);
    grid.randomize();
    
    let start = Instant::now();
    for _ in 0..iterations {
        grid = simd_life::evolve_simd(&grid, &rule);
    }
    start.elapsed().as_secs_f64() * 1000.0 / iterations as f64
}

fn benchmark_bit_grid_simd_parallel(size: usize, iterations: u32) -> f64 {
    let rule = ConwayRule;
    let mut grid = BitGrid::new(size, size);
    grid.randomize();
    
    let start = Instant::now();
    for _ in 0..iterations {
        grid = simd_life::evolve_simd_parallel(&grid, &rule);
    }
    start.elapsed().as_secs_f64() * 1000.0 / iterations as f64
}

fn benchmark_temporal_blocking(size: usize, iterations: u32) -> f64 {
    let rule = ConwayRule;
    let mut grid = BitGrid::new(size, size);
    grid.randomize();
    
    let start = Instant::now();
    // Note: temporal blocking does 4 generations per call
    for _ in 0..(iterations / 4).max(1) {
        grid = temporal_blocking::evolve_temporal_blocking_parallel(&grid, &rule, 4);
    }
    // Adjust for 4 generations per call
    start.elapsed().as_secs_f64() * 1000.0 / (iterations as f64)
}

fn main() {
    println!("=== Game of Life Performance Benchmark ===\n");
    
    let sizes = [100, 500, 1000, 2000, 5000, 10000, 20000];
    let iterations = 20; // More iterations for better accuracy
    
    println!("{:>10} {:>12} {:>12} {:>12} {:>12} {:>12} {:>10}", 
        "Size", "Original", "BitGrid", "SIMD", "SIMD+Par", "TempBlock", "Speedup");
    println!("{:-<90}", "");
    
    for size in sizes {
        // Skip slow algorithms for huge grids to save time
        let original_ms = if size <= 5000 { benchmark_original_grid(size, iterations) } else { 0.0 };
        let naive_ms = if size <= 5000 { benchmark_bit_grid_naive(size, iterations) } else { 0.0 };
        
        let simd_ms = benchmark_bit_grid_simd(size, iterations);
        let simd_par_ms = benchmark_bit_grid_simd_parallel(size, iterations);
        let temp_block_ms = benchmark_temporal_blocking(size, iterations);
        
        let fastest = simd_par_ms.min(temp_block_ms);
        
        let speedup_str = if original_ms > 0.0 {
            format!("{:>9.1}x", original_ms / fastest)
        } else {
            format!("{:>10}", "-")
        };
        
        let orig_str = if original_ms > 0.0 { format!("{:>12.2}", original_ms) } else { format!("{:>12}", "-") };
        let naive_str = if naive_ms > 0.0 { format!("{:>12.2}", naive_ms) } else { format!("{:>12}", "-") };

        println!(
            "{:>10} {} {} {:>12.2} {:>12.2} {:>12.2} {}",
            format!("{}x{}", size, size),
            orig_str,
            naive_str,
            simd_ms,
            simd_par_ms,
            temp_block_ms,
            speedup_str
        );
    }
    
    println!("\n=== Memory Usage (10000x10000) ===\n");
    
    let size = 10000;
    let original_mem = size * size; // 1 byte per Cell enum
    let bit_grid = BitGrid::new(size, size);
    let bit_mem = bit_grid.memory_bytes();
    
    println!("Original Grid: {:>10} bytes ({:.1} MB)", original_mem, original_mem as f64 / 1_000_000.0);
    println!("BitGrid:       {:>10} bytes ({:.1} MB)", bit_mem, bit_mem as f64 / 1_000_000.0);
    println!("Reduction:     {:>10.1}x", original_mem as f64 / bit_mem as f64);
    
    println!("\n=== Throughput at 10000x10000 ===\n");
    
    let cells = 10000 * 10000;
    let extra_iters = 20; 
    let simd_par_ms = benchmark_bit_grid_simd_parallel(10000, extra_iters);
    let temp_block_ms = benchmark_temporal_blocking(10000, extra_iters);
    
    println!("SIMD+Parallel:    {:.2} ms/gen, {:.1}M cells/sec", 
        simd_par_ms, (cells as f64) / (simd_par_ms / 1000.0) / 1_000_000.0);
    println!("TempBlock+Par:    {:.2} ms/gen, {:.1}M cells/sec", 
        temp_block_ms, (cells as f64) / (temp_block_ms / 1000.0) / 1_000_000.0);
}
