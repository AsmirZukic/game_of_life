//! Temporal Blocking optimization for Game of Life
//!
//! Processes multiple generations per tile entirely in cache,
//! reducing memory bandwidth by ~4x for large grids.
//!
//! Optimization: Uses SIMD bit operations within tiles and double-buffering.

use super::bit_grid::BitGrid;
use super::rules::Rule;
use super::simd_life::{build_rule_lookup, compute_next_chunk_with_rule};
use rayon::prelude::*;
use std::mem;

/// Tile size (must be multiple of 64 for chunk alignment)
const TILE_SIZE: usize = 256;

/// Number of generations to process per tile
const GENERATIONS_PER_TILE: usize = 4;

/// Halo size (must equal GENERATIONS_PER_TILE for correctness)
const HALO_SIZE: usize = GENERATIONS_PER_TILE;

/// A small local buffer for tile processing
/// Stores cells as bits in u64 chunks for SIMD usage
#[derive(Clone)]
struct LocalTile {
    /// Width including halo (in cells)
    width: usize,
    /// Height including halo (in cells)
    height: usize,
    /// Bit-packed data (row-major)
    data: Vec<u64>,
    /// Width in chunks
    chunk_width: usize,
}

impl LocalTile {
    /// Create a new local tile buffer
    fn new(width: usize, height: usize) -> Self {
        let chunk_width = (width + 63) / 64;
        Self {
            width,
            height,
            chunk_width,
            data: vec![0u64; chunk_width * height],
        }
    }
    
    /// Get cell state at (x, y)
    #[inline]
    fn get(&self, x: usize, y: usize) -> bool {
        if x >= self.width || y >= self.height {
            return false;
        }
        let chunk_idx = y * self.chunk_width + (x / 64);
        let bit_idx = x % 64;
        (self.data[chunk_idx] >> bit_idx) & 1 != 0
    }
    
    /// Set cell state at (x, y)
    #[inline]
    fn set(&mut self, x: usize, y: usize, alive: bool) {
        if x >= self.width || y >= self.height {
            return;
        }
        let chunk_idx = y * self.chunk_width + (x / 64);
        let bit_idx = x % 64;
        if alive {
            self.data[chunk_idx] |= 1u64 << bit_idx;
        } else {
            self.data[chunk_idx] &= !(1u64 << bit_idx);
        }
    }
    
    /// Get raw chunk at (chunk_x, y)
    #[inline]
    fn get_chunk(&self, chunk_x: usize, y: usize) -> u64 {
        if chunk_x >= self.chunk_width || y >= self.height {
            return 0;
        }
        self.data[y * self.chunk_width + chunk_x]
    }
    
    /// Set raw chunk at (chunk_x, y)
    #[inline]
    fn set_chunk(&mut self, chunk_x: usize, y: usize, val: u64) {
        if chunk_x >= self.chunk_width || y >= self.height {
            return;
        }
        self.data[y * self.chunk_width + chunk_x] = val;
    }

    /// Evolve the tile for one generation using SIMD, writing into `dest`
    /// This avoids allocation by reusing the destination buffer
    fn evolve_into(&self, dest: &mut LocalTile, lookup: &[bool; 32]) {
        for y in 0..self.height {
            // Determine y neighbors (clamp to 0, since boundaries are dead/halo)
            let ya = if y > 0 { y - 1 } else { 0 }; 
            let yb = if y + 1 < self.height { y + 1 } else { 0 };

            for chunk_x in 0..self.chunk_width {
                // Get rows
                let above = if y > 0 { self.get_chunk(chunk_x, ya) } else { 0 };
                let current = self.get_chunk(chunk_x, y);
                let below = if y + 1 < self.height { self.get_chunk(chunk_x, yb) } else { 0 };

                // Left neighbors (check bound and fetch chunk - 1)
                let left_chunk_idx = if chunk_x > 0 { chunk_x - 1 } else { usize::MAX };
                let (left_above, left_current, left_below) = if left_chunk_idx != usize::MAX {
                    (
                        (self.get_chunk(left_chunk_idx, ya) >> 63) != 0 && y > 0,
                        (self.get_chunk(left_chunk_idx, y) >> 63) != 0,
                        (self.get_chunk(left_chunk_idx, yb) >> 63) != 0 && y + 1 < self.height,
                    )
                } else {
                    (false, false, false)
                };

                // Right neighbors (check bound and fetch chunk + 1)
                let right_chunk_idx = chunk_x + 1;
                let (right_above, right_current, right_below) = if right_chunk_idx < self.chunk_width {
                    (
                        (self.get_chunk(right_chunk_idx, ya) & 1) != 0 && y > 0,
                        (self.get_chunk(right_chunk_idx, y) & 1) != 0,
                        (self.get_chunk(right_chunk_idx, yb) & 1) != 0 && y + 1 < self.height,
                    )
                } else {
                    (false, false, false)
                };

                let next_chunk = compute_next_chunk_with_rule(
                    above, current, below,
                    left_above, right_above,
                    left_current, right_current,
                    left_below, right_below,
                    lookup,
                );
                
                dest.set_chunk(chunk_x, y, next_chunk);
            }
        }
    }
}

/// Helper to evolve a tile N generations using double buffering
fn evolve_tile_n_gens(mut tile: LocalTile, generations: usize, lookup: &[bool; 32]) -> LocalTile {
    let mut aux = tile.clone(); // Scratch buffer
    
    for _ in 0..generations {
        tile.evolve_into(&mut aux, lookup);
        mem::swap(&mut tile, &mut aux);
    }
    
    tile
}

/// Copy a region from the global grid into a local tile
fn copy_to_local_tile(
    grid: &BitGrid,
    tile_x: usize,  
    tile_y: usize,  
    local_width: usize,
    local_height: usize,
) -> LocalTile {
    let (grid_width, grid_height) = grid.dimensions();
    
    // Fast path: if the region to copy doesn't wrap around grid boundaries,
    // we can do optimized chunk copies.
    // "region to copy" starts at (tile_x, tile_y) and has size (local_width, local_height).
    // Note: tile_x is already the start coordinate (which includes halo offset from origin).
    // We need to check if tile_x + local_width <= grid_width (no X wrap)
    // AND tile_y + local_height <= grid_height (no Y wrap).
    
    // Also, we support unaligned reads (arbitrary bit offset).
    // But handling Y wrap complexity is high, so fallback there.
    if tile_x + local_width <= grid_width && tile_y + local_height <= grid_height {
        return copy_to_local_tile_fast(grid, tile_x, tile_y, local_width, local_height);
    }

    // Slow path (handles wrapping)
    let mut tile = LocalTile::new(local_width, local_height);
    for ly in 0..local_height {
        for lx in 0..local_width {
            let gx = (tile_x + lx) % grid_width;
            let gy = (tile_y + ly) % grid_height;
            
            if grid.get(gx, gy) {
                tile.set(lx, ly, true);
            }
        }
    }
    tile
}

/// Optimized copy from grid to tile (no wrapping)
fn copy_to_local_tile_fast(
    grid: &BitGrid,
    start_x: usize,
    start_y: usize,
    width: usize,
    height: usize,
) -> LocalTile {
    let mut tile = LocalTile::new(width, height);
    
    // We copy row by row
    for ly in 0..height {
        let global_y = start_y + ly;
        // Global bit index start for this row
        let global_row_bit_start = start_x;
        
        // We write to tile row 'ly'. 
        // Tile rows are aligned to 0.
        // We act as if tile is a contiguous bit buffer per row?
        // Tile storage is Vec<u64>. Row pitch is `chunk_width`.
        
        // Iterate tile chunks
        for chunk_x in 0..tile.chunk_width {
            let tile_bit_offset = chunk_x * 64;
            if tile_bit_offset >= width { break; }
            
            // We want 64 bits starting at global_row_bit_start + tile_bit_offset
            let src_bit_start = global_row_bit_start + tile_bit_offset;
            
            // Read 64 bits from grid at (src_bit_start, global_y)
            // src_bit_start might not be 64-aligned.
            let val = get_u64_unaligned(grid, src_bit_start, global_y);
            
            // Mask out bits beyond width if this is the last chunk
            let bits_remaining = width.saturating_sub(tile_bit_offset);
            let masked_val = if bits_remaining < 64 {
                val & ((1u64 << bits_remaining) - 1)
            } else {
                val
            };
            
            tile.set_chunk(chunk_x, ly, masked_val);
        }
    }
    tile
}

/// Helper to read 64 bits from Grid at arbitrary X offset
/// Assumes no wrapping and valid range up to x+64 (or handles it via raw access)
#[inline]
fn get_u64_unaligned(grid: &BitGrid, x: usize, y: usize) -> u64 {
    // Access internal chunks of BitGrid.
    // Since we can't access `chunks` field directly (it's private), 
    // we must use `get_chunk` if available or `get`.
    // But `BitGrid` only exposes `get(x,y) -> bool`.
    // Wait, `BitGrid` is in our crate. We can make `chunks` public or add a helper.
    // Modifying `BitGrid` to add `get_u64_unaligned` is best.
    // For now, let's assume we added `get_word64(x, y)` to BitGrid.
    // If not, we fall back to slow path or use unsafe?
    // Let's use `get_word` which we will implement in BitGrid.
    grid.get_word64(x, y)
}

/// Copy active cells from tile back to grid
/// Optimized for sparse writes (assumes grid is zeroed)
fn copy_active_cells_to_grid(
    tile: &LocalTile,
    grid: &mut BitGrid,
    tile_x: usize,  
    tile_y: usize,  
    inner_width: usize,
    inner_height: usize,
    halo: usize,
) {
    let (grid_width, grid_height) = grid.dimensions();
    
    // Check for fast path: no wrapping
    if tile_x + inner_width <= grid_width && tile_y + inner_height <= grid_height {
        // Fast path requires disjoint parallel execution if we write full words.
        // TILE_SIZE=256 is multiple of 64.
        // `tile_x` comes from `tile_idx * TILE_SIZE`.
        // So `tile_x` is 64-aligned.
        // `inner_width` is usually `TILE_SIZE` (aligned) or edge-capped.
        // If edge-capped, it might not be aligned?
        // If `width` is 5000. `tile_x=4864`. `width - tile_x = 136`.
        // 136 is not multiple of 64.
        // So the last chunk is partial.
        // But since we assume grid is zeroed, we can OR-in the partial word safely
        // AS LONG AS no other thread touches the same word.
        // The neighbor tile starts at `tile_x + TILE_SIZE`.
        // If `tile_x` is aligned, `tile_x + 256` is aligned.
        // So the boundary between tiles is on a Chunk boundary.
        // So threads NEVER share a chunk.
        // The only exception is the LAST tile at the grid edge.
        // But there is no neighbor to the right of the last tile.
        // So it's safe!
        
        copy_active_cells_to_grid_fast(tile, grid, tile_x, tile_y, inner_width, inner_height, halo);
        return;
    }

    // Slow path
    for ly in 0..inner_height {
        for lx in 0..inner_width {
            let local_x = lx + halo;
            let local_y = ly + halo;
            
            if tile.get(local_x, local_y) {
                let gx = (tile_x + lx) % grid_width;
                let gy = (tile_y + ly) % grid_height;
                grid.set(gx, gy, true);
            }
        }
    }
}

/// Optimized write back (no wrapping, aligned start)
fn copy_active_cells_to_grid_fast(
    tile: &LocalTile,
    grid: &mut BitGrid,
    start_x: usize,
    start_y: usize,
    width: usize,
    height: usize,
    halo: usize,
) {
    // We iterate tile rows
    for ly in 0..height {
        let global_y = start_y + ly;
        let local_y = ly + halo;
        
        // We want to read `width` bits from tile starting at `halo` (local_x)
        // and write to grid starting at `start_x`.
        // `start_x` is 64-aligned (invariant of Tiling strategy).
        // `halo` is 4. Not aligned.
        
        // So we read unaligned u64 from tile, write aligned u64 to grid.
        
        let mut bits_processed = 0;
        
        while bits_processed < width {
            // Read 64 bits from tile at (halo + bits_processed, local_y)
            let tile_val = get_tile_u64_unaligned(tile, halo + bits_processed, local_y);
            
            // Mask if partial
            let remaining = width - bits_processed;
            let val = if remaining < 64 {
                tile_val & ((1u64 << remaining) - 1)
            } else {
                tile_val
            };
            
            // Write to grid. `start_x` is aligned, so we write to chunk directly.
            // But we need to find the specific chunk.
            // grid.set_chunk_at(start_x + bits_processed, global_y, val || existing?)
            // Since we assume zeroed grid, we simply STORE `val`.
            // Wait, assumes destination is clean.
            // bit_processed increases by 64.
            // start_x is 64 aligned.
            // So start_x + bits_processed is 64 aligned.
            // Perfect alignment!
            
            grid.set_word64_or(start_x + bits_processed, global_y, val);
            
            bits_processed += 64;
        }
    }
}

/// Helper to read unaligned from Tile
#[inline]
fn get_tile_u64_unaligned(tile: &LocalTile, x: usize, y: usize) -> u64 {
    // Tile chunks are 64 bits.
    let chunk_idx = y * tile.chunk_width + (x / 64);
    let bit_offset = x % 64;
    
    if bit_offset == 0 {
        return tile.data[chunk_idx];
    }
    
    let low = tile.data[chunk_idx] >> bit_offset;
    let high = if chunk_idx + 1 < tile.data.len() { // simplistic bound check
         tile.data[chunk_idx + 1] << (64 - bit_offset)
    } else {
        0
    };
    
    low | high
}

/// Evolve a BitGrid using temporal blocking (serial version)
pub fn evolve_temporal_blocking(grid: &BitGrid, rule: &dyn Rule, generations: usize) -> BitGrid {
    let (width, height) = grid.dimensions();
    let mut result = BitGrid::new(width, height); // Zeroed output
    
    let lookup = build_rule_lookup(rule);
    let tile_stride = TILE_SIZE;
    
    for tile_y_idx in 0..(height + tile_stride - 1) / tile_stride {
        for tile_x_idx in 0..(width + tile_stride - 1) / tile_stride {
            let tile_x = tile_x_idx * tile_stride;
            let tile_y = tile_y_idx * tile_stride;
            
            let actual_width = (tile_stride).min(width - tile_x);
            let actual_height = (tile_stride).min(height - tile_y);
            let local_width = actual_width + 2 * HALO_SIZE;
            let local_height = actual_height + 2 * HALO_SIZE;
            
            let start_x = (tile_x + width - HALO_SIZE) % width;
            let start_y = (tile_y + height - HALO_SIZE) % height;
            
            let mut local = copy_to_local_tile(grid, start_x, start_y, local_width, local_height);
            
            local = evolve_tile_n_gens(local, generations, &lookup);
            
            copy_active_cells_to_grid(&local, &mut result, tile_x, tile_y, actual_width, actual_height, HALO_SIZE);
        }
    }
    
    result
}

/// Evolve a BitGrid using temporal blocking (parallel version)
pub fn evolve_temporal_blocking_parallel(grid: &BitGrid, rule: &(dyn Rule + Sync), generations: usize) -> BitGrid {
    let (width, height) = grid.dimensions();
    let lookup = build_rule_lookup(rule);
    
    let tile_stride = TILE_SIZE;
    let num_tiles_x = (width + tile_stride - 1) / tile_stride;
    let num_tiles_y = (height + tile_stride - 1) / tile_stride;
    let total_tiles = num_tiles_x * num_tiles_y;
    
    // Process tiles in parallel
    let tile_results: Vec<(usize, usize, LocalTile)> = (0..total_tiles)
        .into_par_iter()
        .map(|tile_idx| {
            let tile_x_idx = tile_idx % num_tiles_x;
            let tile_y_idx = tile_idx / num_tiles_x;
            let tile_x = tile_x_idx * tile_stride;
            let tile_y = tile_y_idx * tile_stride;
            
            let actual_width = tile_stride.min(width - tile_x);
            let actual_height = tile_stride.min(height - tile_y);
            let local_width = actual_width + 2 * HALO_SIZE;
            let local_height = actual_height + 2 * HALO_SIZE;
            
            let start_x = (tile_x + width - HALO_SIZE) % width;
            let start_y = (tile_y + height - HALO_SIZE) % height;
            
            let mut local = copy_to_local_tile(grid, start_x, start_y, local_width, local_height);
            
            local = evolve_tile_n_gens(local, generations, &lookup);
            
            (tile_x, tile_y, local)
        })
        .collect();
    
    // Combine results
    // Wait, collecting to Vec then iterating serially is slow.
    // 3MB memory write is fast, but we can parallelize this too?
    // Rayon doesn't like parallel writes to same structure (BitGrid).
    // But we know writes are disjoint!
    // Unsafe `result.as_mut_ptr()` -> parallel for_each?
    // Safe Rust makes this hard.
    // However, the bottleneck was "Copy into Tile" and "Gather from Tile".
    // 50ms total. 25ms alloc/gather, 25ms scatter?
    // The "collect" phase is sequential scatter.
    // "copy_active_cells_to_grid" is called serially here.
    // Optimizing it makes the serial part fast (memory bandwidth limit).
    // Writing 3MB serially is sub-1ms.
    // The main cost is `copy_to_local_tile` inside the parallel loop.
    // So optimizing `copy_active...` is less critical but still good.
    // Optimizing `copy_to` is CRITICAL.
    
    let mut result = BitGrid::new(width, height); // Zeroed
    for (tile_x, tile_y, local) in tile_results {
        let actual_width = tile_stride.min(width - tile_x);
        let actual_height = tile_stride.min(height - tile_y);
        copy_active_cells_to_grid(&local, &mut result, tile_x, tile_y, actual_width, actual_height, HALO_SIZE);
    }
    
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::ConwayRule;
    
    #[test]
    fn test_local_tile_simd_evolution() {
        let width = 64;
        let height = 64;
        let mut tile = LocalTile::new(width, height);
        
        // Horizontal blinker
        tile.set(10, 9, true);
        tile.set(10, 10, true);
        tile.set(10, 11, true);
        
        let rule = ConwayRule;
        let lookup = build_rule_lookup(&rule);
        
        let next = evolve_tile_n_gens(tile, 1, &lookup);
        
        // Vertical blinker
        assert!(next.get(9, 10));
        assert!(next.get(10, 10));
        assert!(next.get(11, 10));
        assert!(!next.get(10, 9));
        assert!(!next.get(10, 11));
    }
    
    #[test]
    fn test_temporal_blocking_matches_reference() {
        let rule = ConwayRule;
        let mut grid = BitGrid::new(100, 100);
        
        // Blinker
        grid.set(50, 49, true);
        grid.set(50, 50, true);
        grid.set(50, 51, true);
        
        // Reference (SIMD)
        let reference = {
            let mut g = grid.clone();
            for _ in 0..4 {
                g = crate::domain::simd_life::evolve_simd(&g, &rule);
            }
            g
        };
        
        // Temporal
        let temporal = evolve_temporal_blocking(&grid, &rule, 4);
        
        // Compare
        let (w, h) = grid.dimensions();
        for y in 0..h {
            for x in 0..w {
                assert_eq!(
                    reference.get(x, y), temporal.get(x, y),
                    "Mismatch at ({}, {})", x, y
                );
            }
        }
    }
}
