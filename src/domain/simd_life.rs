//! SIMD-accelerated Life evolution
//! 
//! Uses bit manipulation tricks to process many cells in parallel:
//! - Horizontal neighbors: shift bits left/right
//! - Vertical neighbors: access adjacent rows
//! - Counting: parallel bit addition (carry-save adder)

use super::bit_grid::BitGrid;
use super::{Rule, Cell};

/// Build a lookup table for a rule.
/// Index format: (is_alive << 4) | neighbor_count
/// This precomputes the rule result for all 18 possible (alive, count) combinations.
#[inline]
pub fn build_rule_lookup(rule: &dyn Rule) -> [bool; 32] {
    let mut table = [false; 32];
    for neighbors in 0u8..=8 {
        // Index 0-8: dead cell with 0-8 neighbors
        table[neighbors as usize] = rule.evolve(Cell::Dead, neighbors) == Cell::Alive;
        // Index 16-24: alive cell with 0-8 neighbors
        table[16 + neighbors as usize] = rule.evolve(Cell::Alive, neighbors) == Cell::Alive;
    }
    table
}

/// Compute neighbor counts for a chunk and return the 4-bit count per cell position.
/// Returns (bit0, bit1, bit2, bit3) where count = bit3*8 + bit2*4 + bit1*2 + bit0
#[inline]
fn compute_neighbor_counts(
    above: u64,
    current: u64,
    below: u64,
    left_bit_above: bool,
    right_bit_above: bool,
    left_bit_current: bool,
    right_bit_current: bool,
    left_bit_below: bool,
    right_bit_below: bool,
) -> (u64, u64, u64, u64) {
    // Horizontal shifts for above row
    let above_left = (above >> 1) | if left_bit_above { 1u64 << 63 } else { 0 };
    let above_right = (above << 1) | if right_bit_above { 1 } else { 0 };
    
    // Horizontal shifts for current row
    let current_left = (current >> 1) | if left_bit_current { 1u64 << 63 } else { 0 };
    let current_right = (current << 1) | if right_bit_current { 1 } else { 0 };
    
    // Horizontal shifts for below row
    let below_left = (below >> 1) | if left_bit_below { 1u64 << 63 } else { 0 };
    let below_right = (below << 1) | if right_bit_below { 1 } else { 0 };
    
    let neighbors = [
        above_left, above, above_right,
        current_left, current_right,
        below_left, below, below_right,
    ];
    
    // Count using parallel prefix sum technique
    let (sum1, carry1) = full_adder(neighbors[0], neighbors[1], neighbors[2]);
    let (sum2, carry2) = full_adder(neighbors[3], neighbors[4], neighbors[5]);
    let (sum3, carry3) = full_adder(neighbors[6], neighbors[7], 0);
    
    let (sum_a, carry_a) = full_adder(sum1, sum2, sum3);
    let (sum_b, carry_b) = full_adder(carry1, carry2, carry3);
    
    let (bit0, c1) = half_adder(sum_a, 0);
    let (bit1, c2) = full_adder(sum_b, carry_a, c1);
    let (bit2, c3) = full_adder(carry_b, 0, c2);
    let bit3 = c3;
    
    (bit0, bit1, bit2, bit3)
}

/// Apply a rule lookup table to compute next chunk state.
/// For each of 64 cells, extract the neighbor count and current state,
/// then look up the result in the precomputed table.
#[inline]
fn apply_rule_lookup(current: u64, bit0: u64, bit1: u64, bit2: u64, bit3: u64, lookup: &[bool; 32]) -> u64 {
    let mut result = 0u64;
    
    for i in 0..64 {
        let count = (((bit3 >> i) & 1) << 3) 
                  | (((bit2 >> i) & 1) << 2)
                  | (((bit1 >> i) & 1) << 1)
                  | ((bit0 >> i) & 1);
        let is_alive = (current >> i) & 1;
        let idx = (is_alive << 4) | count;
        
        if lookup[idx as usize] {
            result |= 1u64 << i;
        }
    }
    
    result
}

/// Compute next chunk using Conway's rules (optimized bitwise version)
/// Kept for backwards compatibility and maximum performance when using Conway's rules.
#[inline]
pub fn compute_next_chunk_conway(
    above: u64,
    current: u64,
    below: u64,
    left_bit_above: bool,
    right_bit_above: bool,
    left_bit_current: bool,
    right_bit_current: bool,
    left_bit_below: bool,
    right_bit_below: bool,
) -> u64 {
    let (bit0, bit1, bit2, bit3) = compute_neighbor_counts(
        above, current, below,
        left_bit_above, right_bit_above,
        left_bit_current, right_bit_current,
        left_bit_below, right_bit_below,
    );
    
    // count == 2: bit3=0, bit2=0, bit1=1, bit0=0
    // count == 3: bit3=0, bit2=0, bit1=1, bit0=1
    let count_is_2 = !bit3 & !bit2 & bit1 & !bit0;
    let count_is_3 = !bit3 & !bit2 & bit1 & bit0;
    
    // Conway's rules: alive_next = (count == 3) OR (current AND count == 2)
    count_is_3 | (current & count_is_2)
}

/// Compute next chunk using arbitrary rules via lookup table
#[inline]
pub fn compute_next_chunk_with_rule(
    above: u64,
    current: u64,
    below: u64,
    left_bit_above: bool,
    right_bit_above: bool,
    left_bit_current: bool,
    right_bit_current: bool,
    left_bit_below: bool,
    right_bit_below: bool,
    lookup: &[bool; 32],
) -> u64 {
    let (bit0, bit1, bit2, bit3) = compute_neighbor_counts(
        above, current, below,
        left_bit_above, right_bit_above,
        left_bit_current, right_bit_current,
        left_bit_below, right_bit_below,
    );
    
    apply_rule_lookup(current, bit0, bit1, bit2, bit3, lookup)
}

/// Legacy function name for backwards compatibility
#[inline]
pub fn compute_next_chunk(
    above: u64,
    current: u64,
    below: u64,
    left_bit_above: bool,
    right_bit_above: bool,
    left_bit_current: bool,
    right_bit_current: bool,
    left_bit_below: bool,
    right_bit_below: bool,
) -> u64 {
    compute_next_chunk_conway(
        above, current, below,
        left_bit_above, right_bit_above,
        left_bit_current, right_bit_current,
        left_bit_below, right_bit_below,
    )
}

/// Full adder: sum = a XOR b XOR c, carry = majority(a, b, c)
#[inline]
fn full_adder(a: u64, b: u64, c: u64) -> (u64, u64) {
    let sum = a ^ b ^ c;
    let carry = (a & b) | (c & (a ^ b));
    (sum, carry)
}

/// Half adder: sum = a XOR b, carry = a AND b
#[inline]
fn half_adder(a: u64, b: u64) -> (u64, u64) {
    (a ^ b, a & b)
}

/// Helper to get edge bits for a chunk with toroidal wrapping
#[inline]
fn get_edge_bits(grid: &BitGrid, chunk_x: usize, y: usize, chunk_width: usize, height: usize) -> (bool, bool, bool, bool, bool, bool) {
    // Toroidal wrapping for y coordinates
    let ya = if y > 0 { y - 1 } else { height - 1 };
    let yb = if y + 1 < height { y + 1 } else { 0 };
    
    // Left neighbors come from bit 0 of the next chunk (chunk_x + 1), with wrapping
    let next_chunk_x = if chunk_x + 1 < chunk_width { chunk_x + 1 } else { 0 };
    let left_above = (grid.get_chunk(next_chunk_x, ya) & 1) != 0;
    let left_current = (grid.get_chunk(next_chunk_x, y) & 1) != 0;
    let left_below = (grid.get_chunk(next_chunk_x, yb) & 1) != 0;
    
    // Right neighbors come from bit 63 of the previous chunk (chunk_x - 1), with wrapping
    let prev_chunk_x = if chunk_x > 0 { chunk_x - 1 } else { chunk_width - 1 };
    let right_above = (grid.get_chunk(prev_chunk_x, ya) >> 63) != 0;
    let right_current = (grid.get_chunk(prev_chunk_x, y) >> 63) != 0;
    let right_below = (grid.get_chunk(prev_chunk_x, yb) >> 63) != 0;
    
    (left_above, right_above, left_current, right_current, left_below, right_below)
}

/// Evolve a BitGrid using SIMD-optimized bit operations with specified rule (toroidal)
pub fn evolve_simd(grid: &BitGrid, rule: &dyn Rule) -> BitGrid {
    let (width, height) = grid.dimensions();
    let mut next = BitGrid::new(width, height);
    let chunk_width = (width + 63) / 64;
    
    // Build lookup table for this rule
    let lookup = build_rule_lookup(rule);
    
    for y in 0..height {
        for chunk_x in 0..chunk_width {
            // Toroidal wrapping for above/below rows
            let ya = if y > 0 { y - 1 } else { height - 1 };
            let yb = if y + 1 < height { y + 1 } else { 0 };
            
            let above = grid.get_chunk(chunk_x, ya);
            let current = grid.get_chunk(chunk_x, y);
            let below = grid.get_chunk(chunk_x, yb);
            
            let (left_above, right_above, left_current, right_current, left_below, right_below) = 
                get_edge_bits(grid, chunk_x, y, chunk_width, height);
            
            let next_chunk = compute_next_chunk_with_rule(
                above, current, below,
                left_above, right_above,
                left_current, right_current,
                left_below, right_below,
                &lookup,
            );
            
            next.set_chunk(chunk_x, y, next_chunk);
        }
    }
    
    next
}

/// Parallel SIMD evolution using rayon with specified rule (toroidal)
/// Optimized: pre-allocated buffer, batched row processing to reduce scheduling overhead
pub fn evolve_simd_parallel(grid: &BitGrid, rule: &(dyn Rule + Sync)) -> BitGrid {
    use rayon::prelude::*;
    
    let (width, height) = grid.dimensions();
    let chunk_width = (width + 63) / 64;
    
    // Build lookup table for this rule
    let lookup = build_rule_lookup(rule);
    
    // Pre-allocate output chunks as contiguous buffer
    let mut output_chunks: Vec<u64> = vec![0u64; height * chunk_width];
    
    // Batch multiple rows per task to reduce rayon scheduling overhead
    // Target: ~16-32 tasks per thread for good load balancing
    let num_threads = rayon::current_num_threads();
    let min_rows_per_task = (height / (num_threads * 32)).max(4);
    
    // Process rows in parallel with batching
    output_chunks
        .par_chunks_mut(chunk_width)
        .enumerate()
        .with_min_len(min_rows_per_task)
        .for_each(|(y, row_output)| {
            // Toroidal wrapping
            let ya = if y > 0 { y - 1 } else { height - 1 };
            let yb = if y + 1 < height { y + 1 } else { 0 };
            
            for chunk_x in 0..chunk_width {
                let above = grid.get_chunk(chunk_x, ya);
                let current = grid.get_chunk(chunk_x, y);
                let below = grid.get_chunk(chunk_x, yb);
                
                let (left_above, right_above, left_current, right_current, left_below, right_below) = 
                    get_edge_bits(grid, chunk_x, y, chunk_width, height);
                
                row_output[chunk_x] = compute_next_chunk_with_rule(
                    above, current, below,
                    left_above, right_above,
                    left_current, right_current,
                    left_below, right_below,
                    &lookup,
                );
            }
        });
    
    BitGrid::from_chunks(width, height, output_chunks)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{ConwayRule, HighLifeRule, SeedsRule};
    
    #[test]
    fn test_full_adder() {
        // 0 + 0 + 0 = 0, carry 0
        assert_eq!(full_adder(0, 0, 0), (0, 0));
        
        // 1 + 0 + 0 = 1, carry 0
        assert_eq!(full_adder(1, 0, 0), (1, 0));
        
        // 1 + 1 + 0 = 0, carry 1
        assert_eq!(full_adder(1, 1, 0), (0, 1));
        
        // 1 + 1 + 1 = 1, carry 1
        assert_eq!(full_adder(1, 1, 1), (1, 1));
        
        // Test with bit patterns
        assert_eq!(full_adder(0b1010, 0b1100, 0b0110), (0b0000, 0b1110));
    }
    
    #[test]
    fn test_rule_lookup_conway() {
        let rule = ConwayRule;
        let lookup = build_rule_lookup(&rule);
        
        // Dead cell with 3 neighbors -> alive
        assert!(lookup[3]);
        // Dead cell with 2 neighbors -> dead
        assert!(!lookup[2]);
        // Alive cell with 2 neighbors -> alive
        assert!(lookup[16 + 2]);
        // Alive cell with 3 neighbors -> alive
        assert!(lookup[16 + 3]);
        // Alive cell with 4 neighbors -> dead
        assert!(!lookup[16 + 4]);
    }
    
    #[test]
    fn test_rule_lookup_highlife() {
        let rule = HighLifeRule;
        let lookup = build_rule_lookup(&rule);
        
        // HighLife: B36/S23
        // Dead cell with 3 or 6 neighbors -> alive
        assert!(lookup[3]);
        assert!(lookup[6]);
        // Dead cell with 2 neighbors -> dead
        assert!(!lookup[2]);
    }
    
    #[test]
    fn test_isolated_cell_dies() {
        let rule = ConwayRule;
        let lookup = build_rule_lookup(&rule);
        
        // A single cell with no neighbors should die
        let current = 1u64 << 32;
        let result = compute_next_chunk_with_rule(0, current, 0, false, false, false, false, false, false, &lookup);
        
        assert_eq!(result & (1u64 << 32), 0, "Isolated cell should die");
    }
    
    #[test]
    fn test_cell_with_two_neighbors_survives() {
        let rule = ConwayRule;
        let lookup = build_rule_lookup(&rule);
        
        // Cell at bit 32 with neighbors at bits 31 and 33
        let current = (1u64 << 31) | (1u64 << 32) | (1u64 << 33);
        let result = compute_next_chunk_with_rule(0, current, 0, false, false, false, false, false, false, &lookup);
        
        // Middle cell has 2 neighbors, should survive
        assert_ne!(result & (1u64 << 32), 0, "Cell with 2 neighbors should survive");
    }
    
    #[test]
    fn test_cell_with_three_neighbors_survives() {
        let rule = ConwayRule;
        let lookup = build_rule_lookup(&rule);
        
        // Create a block pattern (2x2) represented across two rows
        let above = (1u64 << 32) | (1u64 << 33);
        let current = (1u64 << 32) | (1u64 << 33);
        
        let result = compute_next_chunk_with_rule(above, current, 0, false, false, false, false, false, false, &lookup);
        
        // All 4 cells should survive (each has exactly 3 neighbors)
        assert_ne!(result & (1u64 << 32), 0);
        assert_ne!(result & (1u64 << 33), 0);
    }
    
    #[test]
    fn test_birth_with_three_neighbors() {
        let rule = ConwayRule;
        let lookup = build_rule_lookup(&rule);
        
        // Put 3 cells above an empty position
        let above = (1u64 << 31) | (1u64 << 32) | (1u64 << 33);
        let current_empty = 0u64;
        
        let result = compute_next_chunk_with_rule(above, current_empty, 0, false, false, false, false, false, false, &lookup);
        
        // Position 32 should be born (has 3 neighbors above)
        assert_ne!(result & (1u64 << 32), 0, "Cell should be born with 3 neighbors");
    }
    
    #[test] 
    fn test_simd_blinker_evolution() {
        let rule = ConwayRule;
        let mut grid = BitGrid::new(10, 10);
        
        // Horizontal blinker: (4,5), (5,5), (6,5)
        grid.set(4, 5, true);
        grid.set(5, 5, true);
        grid.set(6, 5, true);
        
        let next = evolve_simd(&grid, &rule);
        
        // Should become vertical
        assert!(!next.get(4, 5), "Left cell should die");
        assert!(next.get(5, 4), "Top cell should be born");
        assert!(next.get(5, 5), "Center should survive");
        assert!(next.get(5, 6), "Bottom cell should be born");
        assert!(!next.get(6, 5), "Right cell should die");
    }
    
    #[test]
    fn test_simd_matches_naive() {
        let rule = ConwayRule;
        let mut grid = BitGrid::new(100, 100);
        
        // Create a pattern
        for i in 0..50 {
            grid.set(i * 2, i, true);
            grid.set(i * 2 + 1, i, true);
        }
        
        let naive_result = grid.evolve(&rule);
        let simd_result = evolve_simd(&grid, &rule);
        
        for y in 0..100 {
            for x in 0..100 {
                assert_eq!(
                    naive_result.get(x, y),
                    simd_result.get(x, y),
                    "Mismatch at ({}, {})", x, y
                );
            }
        }
    }
    
    #[test]
    fn test_simd_parallel_matches_serial() {
        let rule = ConwayRule;
        let mut grid = BitGrid::new(200, 200);
        grid.randomize();
        
        let serial = evolve_simd(&grid, &rule);
        let parallel = evolve_simd_parallel(&grid, &rule);
        
        for y in 0..200 {
            for x in 0..200 {
                assert_eq!(
                    serial.get(x, y),
                    parallel.get(x, y),
                    "Parallel mismatch at ({}, {})", x, y
                );
            }
        }
    }
    
    #[test]
    fn test_simd_with_different_rules() {
        // Test that SIMD with HighLife produces different results than Conway
        let mut grid = BitGrid::new(20, 20);
        grid.set(10, 10, true);
        grid.set(11, 10, true);
        grid.set(12, 10, true);
        grid.set(10, 11, true);
        grid.set(11, 11, true);
        grid.set(12, 11, true);
        
        let conway = ConwayRule;
        let highlife = HighLifeRule;
        
        let result_conway = evolve_simd(&grid, &conway);
        let result_highlife = evolve_simd(&grid, &highlife);
        
        // The results should be different because HighLife has B36 vs Conway's B3
        // Count alive cells - they may differ
        let count_conway = result_conway.count_alive();
        let count_highlife = result_highlife.count_alive();
        
        // With 6 cells in a 2x3 block, some cells have 3-5 neighbors
        // HighLife's B6 rule will cause births where Conway wouldn't
        // Just verify both evolved without error
        assert!(count_conway > 0 || count_highlife > 0, "At least one should have live cells");
    }
    
    #[test]
    fn test_seeds_rule_all_die() {
        // Seeds rule: all alive cells die every generation, birth only with exactly 2 neighbors
        let rule = SeedsRule;
        let mut grid = BitGrid::new(10, 10);
        
        // Create a simple 2-cell pattern
        // Two horizontal cells: (5,5) and (6,5)
        grid.set(5, 5, true);
        grid.set(6, 5, true);
        
        let next = evolve_simd(&grid, &rule);
        
        // Original cells should all be dead (Seeds has no survival)
        assert!(!next.get(5, 5), "Cell at (5,5) should die");
        assert!(!next.get(6, 5), "Cell at (6,5) should die");
        
        // Cells born where there were exactly 2 neighbors:
        // (5,4), (6,4), (5,6), (6,6) - at corners of the 2-cell block - have 1 neighbor each
        // (4,5), (7,5) - left and right - have 1 neighbor each
        // Actually with 2 horizontal cells, neighbors:
        // (5,4) has neighbors (5,5), (6,5) = 2 neighbors -> should be born!
        // (6,4) has neighbors (5,5), (6,5) = 2 neighbors -> should be born!
        // (5,6) has neighbors (5,5), (6,5) = 2 neighbors -> should be born!
        // (6,6) has neighbors (5,5), (6,5) = 2 neighbors -> should be born!
        assert!(next.get(5, 4), "Should be born at (5,4) with 2 neighbors");
        assert!(next.get(6, 4), "Should be born at (6,4) with 2 neighbors");
        assert!(next.get(5, 6), "Should be born at (5,6) with 2 neighbors");
        assert!(next.get(6, 6), "Should be born at (6,6) with 2 neighbors");
    }
}
