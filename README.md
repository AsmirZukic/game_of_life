# Rust Game of Life - High Performance Demo

A highly optimized Implementation of Conway's Game of Life in Rust, demonstrating advanced optimization techniques including SIMD, Bit-Packing, and Parallelism.

## Features

### Multiple Algorithms
This project implements several evolution strategies to demonstrate performance scaling:
1.  **Original**: Naive `Vec<Cell>` implementation (Baseline).
2.  **BitGrid**: 1-bit per cell storage (8x memory reduction).
3.  **SIMD**: AVX2-optimized bitwise operations processing 64 cells per step.
4.  **SIMD + Parallel**: The **Fastest** implementation. Combines SIMD with multi-threading (Rayon) for massive throughput.
5.  **Temporal Blocking**: Cache-aware tiling algorithm that processes 4 generations at once to reduce memory bandwidth.

### Rule Sets
Supports configurable cellular automata rules:
- **Conway's Game of Life** (B3/S23)
- **HighLife** (B36/S23)
- **Seeds** (B2/S)

### Interactive UI
- Zoomable/Pannable infinite grid
- Pattern placement (Gliders, Spaceships, Guns)
- Real-time performance metrics
- Configurable simulation speed

## Performance Results

Benchmarks run on the test environment specified below with `target-cpu=native`.

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

## Test Environment

These results were achieved under specific constraints that I chose for this project.

- **Virtualization**: Proxmox VM
- **OS**: OpenSUSE Leap (Kernel 6.12.0-160000.9-default)
- **Host CPU**: Ryzen 5 4650G
- **VM Evaluation**: 4 Cores allocated, 8GB RAM

### Performance Note
There is likely significant performance left on the table. A more modern CPU (e.g., AVX-512 support) and faster DDR5 RAM would likely yield even higher throughput than what was observed in this constrained VM environment.

If you try this out on different hardware, I would be interested to hear your results!

## Build & Run

### Standard Build
```bash
cargo run --release
```

### Optimized Build (Recommended)
Unlocks AVX2/BMI2 instructions for maximum speed.
```bash
RUSTFLAGS="-C target-cpu=native" cargo run --release
```

## How to Play
- **Left Click**: Draw cells
- **Right Click**: Erase cells
- **Scroll Wheel**: Zoom
- **Middle Click**: Pan
- **Space**: Pause/Resume
- **C**: Clear Grid
- **R**: Randomize Grid
