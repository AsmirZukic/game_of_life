# Rust Game of Life - High Performance Demo

A highly optimized Implementation of Conway's Game of Life in Rust, demonstrating advanced optimization techniques including SIMD, Bit-Packing, and Parallelism.

## 🚀 Features

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

## ⚡ Performance Results

Benchmarks run on **Ryzen 5 4650G** (6C/12T) with `target-cpu=native`.

| Grid Size | Algorithm | Time/Gen | Throughput | speedup |
|-----------|-----------|----------|------------|---------|
| 100x100 | Original | 0.88 ms | - | 1x |
| | **SIMD+Par** | **0.05 ms** | - | **22x** |
| 5000x5000 | Original | 910 ms | 27M cells/s | 1x |
| (25M cells)| **SIMD+Par** | **13.8 ms** | **1.8B cells/s** | **60x** |
| 20000x20000| **SIMD+Par** | **207 ms** | **1.75B cells/s** | |
| (400M cells)| (Native) | | | |

*Using `SIMD + Parallel` with `RUSTFLAGS="-C target-cpu=native"` provides a 30% speedup over generic builds on modern CPUs.*

## 🛠️ Build & Run

### Standard Build
```bash
cargo run --release
```

### Optimized Build (Recommended)
Unlocks AVX2/BMI2 instructions for maximum speed.
```bash
RUSTFLAGS="-C target-cpu=native" cargo run --release
```

## 🎮 How to Play
- **Left Click**: Draw cells
- **Right Click**: Erase cells
- **Scroll Wheel**: Zoom
- **Middle Click**: Pan
- **Space**: Pause/Resume
- **C**: Clear Grid
- **R**: Randomize Grid
