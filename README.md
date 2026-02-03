# Game of Life in Rust

This project started as a simple exercise to build a Cellular Automata engine in Rust. I wanted to understand how these simulations work and build a nice interactive playground for them.

Once I had the basics working, I started adding features I wanted to play with, like different rule sets and a library of patterns because drawing gliders by hand gets tedious fast.

Eventually, it turned into a performance challenge. I wanted to see just how many cells I could simulate per second on my machine by implementing known optimization techniques like bit-packing and SIMD.

## Features

It works as a fully interactive simulator that supports multiple rule sets including Conway's Life, HighLife, and Seeds. It features a built-in pattern library containing various spaceships and guns. The grid is completely interactive, allowing you to zoom, pan, and draw or erase cells with your mouse while monitoring real-time performance metrics.

## The Optimization Experiment

After building the core engine, I found the performance on large grids was pretty bad (~0.7ms for a tiny 100x100 grid). So I went down the rabbit hole of standard optimizations.

First came memory compression, where I switched from storing bytes to bits (`BitGrid`), reducing memory usage by 8x. Then I added parallelism using `rayon` to split the workload across CPU cores. Since the data was now just bits, I could use AVX/SSE instructions to process 64 cells at once using hardware bitwise operations. Finally, I explored temporal blocking strategies to keep data in the CPU cache longer.

It turns out that for my specific hardware (Ryzen 5 4650G), the **SIMD + Parallel** approach combined with the `target-cpu=native` compiler flag gives the best results.

## Benchmark Results

I ran these on a Proxmox VM (4 cores, 8GB RAM). The speedup from the naive implementation to the final optimized version is about **65x**. We're pushing over **2 Billion cells per second**.

```text
      Size     Original      BitGrid         SIMD     SIMD+Par    TempBlock    Speedup
------------------------------------------------------------------------------------------
   100x100         0.70         0.59         0.02         0.06         0.05      13.6x
   500x500         8.73         7.92         0.44         0.16         0.25      53.9x
 1000x1000        32.40        32.35         1.83         0.57         0.92      57.1x
 2000x2000       125.68       124.96         7.17         2.15         3.69      58.5x
 5000x5000       781.48       772.04        45.15        11.95        16.31      65.4x
10000x10000            -            -       179.90        47.04        62.75          -
20000x20000            -            -       700.87       184.99       241.87          -
```

```text
=== Memory Usage (10000x10000) ===

Original Grid:  100000000 bytes (100.0 MB)
BitGrid:         12560000 bytes (12.6 MB)
Reduction:            8.0x
```

```text
=== Throughput at 10000x10000 ===

SIMD+Parallel:    47.66 ms/gen, 2098.0M cells/sec
TempBlock+Par:    61.76 ms/gen, 1619.1M cells/sec
```

## Build & Run

To run this project, you will need the Rust compiler installed. If you haven't installed it yet, `rustup` is the recommended way to do so.

```bash
# Build the project (release mode is highly recommended for performance)
cargo build --release

# Run the simulation
cargo run --release
```

To achieve the maximum performance shown in the benchmarks, you should compile with CPU-specific optimizations enabled. This allows the compiler to use AVX2 and BMI2 instructions available on your processor:

```bash
RUSTFLAGS="-C target-cpu=native" cargo run --release
```

## Disclaimer

Parts of this code and documentation were generated with the assistance of an AI.
