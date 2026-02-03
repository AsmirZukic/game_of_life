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

| Grid Size | Algorithm | Time/Gen | Throughput | speedup |
|-----------|-----------|----------|------------|---------|
| 100x100 | Original | 0.70 ms | - | 1x |
| | **SIMD+Par** | **0.06 ms** | - | **11.7x** |
| 5000x5000 | Original | 781.48 ms | 32M cells/s | 1x |
| (25M cells)| **SIMD+Par** | **11.95 ms** | **2.1B cells/s** | **65.4x** |
| 20000x20000| **SIMD+Par** | **184.99 ms** | **2.16B cells/s** | |
| (400M cells)| (Native) | | | |

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
