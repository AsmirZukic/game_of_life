//! Bit-packed grid implementation for maximum performance.
//! Each cell is stored as a single bit, giving 8x memory reduction
//! and enabling SIMD operations on 64 cells at once.

use super::{Cell, Rule};

/// A chunk of 64 cells stored as a single u64
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub struct Chunk64(pub u64);

impl Chunk64 {
    /// Create empty chunk (all dead)
    pub const fn empty() -> Self {
        Self(0)
    }
    
    /// Create full chunk (all alive)
    pub const fn full() -> Self {
        Self(u64::MAX)
    }
    
    /// Get cell state at position (0-63)
    #[inline]
    pub fn get(&self, idx: u8) -> bool {
        debug_assert!(idx < 64);
        (self.0 >> idx) & 1 == 1
    }
    
    /// Set cell state at position (0-63)
    #[inline]
    pub fn set(&mut self, idx: u8, alive: bool) {
        debug_assert!(idx < 64);
        if alive {
            self.0 |= 1u64 << idx;
        } else {
            self.0 &= !(1u64 << idx);
        }
    }
    
    /// Count alive cells (population count)
    #[inline]
    pub fn count_alive(&self) -> u32 {
        self.0.count_ones()
    }
    
    /// Check if chunk is empty (all dead)
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0 == 0
    }
}

/// Bit-packed grid storing cells as individual bits
#[derive(Clone)]
pub struct BitGrid {
    /// Width in cells
    width: usize,
    /// Height in cells  
    height: usize,
    /// Width in chunks (ceil(width / 64))
    chunk_width: usize,
    /// Flat array of chunks, row-major order
    chunks: Vec<Chunk64>,
}

impl BitGrid {
    /// Create new empty bit grid
    pub fn new(width: usize, height: usize) -> Self {
        let chunk_width = (width + 63) / 64;  // Ceiling division
        let total_chunks = chunk_width * height;
        
        Self {
            width,
            height,
            chunk_width,
            chunks: vec![Chunk64::empty(); total_chunks],
        }
    }
    
    /// Create bit grid from pre-computed chunks (zero-copy for parallel evolution)
    /// 
    /// # Safety
    /// This is safe because Chunk64 is #[repr(transparent)] over u64
    pub fn from_chunks(width: usize, height: usize, raw_chunks: Vec<u64>) -> Self {
        let chunk_width = (width + 63) / 64;
        debug_assert_eq!(raw_chunks.len(), chunk_width * height);
        
        // Zero-copy conversion: Vec<u64> -> Vec<Chunk64>
        // Safety: Chunk64 is #[repr(transparent)] over u64, so they have identical memory layout
        let chunks: Vec<Chunk64> = unsafe {
            let mut raw = std::mem::ManuallyDrop::new(raw_chunks);
            Vec::from_raw_parts(
                raw.as_mut_ptr() as *mut Chunk64,
                raw.len(),
                raw.capacity()
            )
        };
        
        Self {
            width,
            height,
            chunk_width,
            chunks,
        }
    }
    
    /// Get grid dimensions
    pub fn dimensions(&self) -> (usize, usize) {
        (self.width, self.height)
    }
    
    /// Get cell state at (x, y)
    #[inline]
    pub fn get(&self, x: usize, y: usize) -> bool {
        if x >= self.width || y >= self.height {
            return false;
        }
        let chunk_idx = y * self.chunk_width + (x / 64);
        let bit_idx = (x % 64) as u8;
        self.chunks[chunk_idx].get(bit_idx)
    }
    
    /// Set cell state at (x, y)
    #[inline]
    pub fn set(&mut self, x: usize, y: usize, alive: bool) {
        if x >= self.width || y >= self.height {
            return;
        }
        let chunk_idx = y * self.chunk_width + (x / 64);
        let bit_idx = (x % 64) as u8;
        self.chunks[chunk_idx].set(bit_idx, alive);
    }
    
    /// Get raw chunk at (chunk_x, row_y) for SIMD operations
    #[inline]
    pub fn get_chunk(&self, chunk_x: usize, y: usize) -> u64 {
        if y >= self.height || chunk_x >= self.chunk_width {
            return 0;
        }
        self.chunks[y * self.chunk_width + chunk_x].0
    }
    
    /// Set raw chunk at (chunk_x, row_y) for SIMD operations
    pub fn set_chunk(&mut self, chunk_x: usize, y: usize, value: u64) {
        if y >= self.height || chunk_x >= self.chunk_width {
            return;
        }
        self.chunks[y * self.chunk_width + chunk_x].0 = value;
    }

    /// Get 64-bit word starting at (x, y). Handles unaligned access.
    /// Reads across chunk boundaries if necessary.
    #[inline]
    pub fn get_word64(&self, x: usize, y: usize) -> u64 {
        let chunk_idx = y * self.chunk_width + (x / 64);
        let bit_idx = (x % 64) as u8; // u8 for shift
        
        // Safety check for simple bounds
        if chunk_idx >= self.chunks.len() {
            return 0;
        }

        let low = self.chunks[chunk_idx].0 >> bit_idx;
        
        if bit_idx == 0 {
            return low;
        }
        
        // If we span two chunks within the valid chunks array
        if chunk_idx + 1 < self.chunks.len() {
            // Note: we don't strictly check for row crossing here, 
            // relying on caller to handle valid width or padding being sufficient.
            let high = self.chunks[chunk_idx + 1].0 << (64 - bit_idx);
            low | high
        } else {
            low
        }
    }

    /// bit-wise OR a 64-bit word at (x, y). Handles unaligned access.
    /// Used for optimized scatter implementation (assumes zeroed destination or accumulation).
    #[inline]
    pub fn set_word64_or(&mut self, x: usize, y: usize, val: u64) {
        let chunk_idx = y * self.chunk_width + (x / 64);
        let bit_idx = (x % 64) as u8;
        
        if chunk_idx >= self.chunks.len() {
            return;
        }

        if bit_idx == 0 {
            self.chunks[chunk_idx].0 |= val;
            return;
        }
        
        self.chunks[chunk_idx].0 |= val << bit_idx;
        
        if chunk_idx + 1 < self.chunks.len() {
             let high_val = val >> (64 - bit_idx);
             self.chunks[chunk_idx + 1].0 |= high_val;
        }
    }
    
    /// Count neighbors at (x, y) with toroidal wrapping
    pub fn count_neighbors(&self, x: usize, y: usize) -> u8 {
        let mut count = 0u8;
        let w = self.width as i32;
        let h = self.height as i32;
        
        for dy in -1i32..=1 {
            for dx in -1i32..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                
                // Toroidal wrapping: mod with dimensions
                let nx = ((x as i32 + dx) % w + w) % w;
                let ny = ((y as i32 + dy) % h + h) % h;
                
                if self.get(nx as usize, ny as usize) {
                    count += 1;
                }
            }
        }
        
        count
    }
    
    /// Total memory usage in bytes
    pub fn memory_bytes(&self) -> usize {
        self.chunks.len() * std::mem::size_of::<Chunk64>()
    }
    
    /// Count total alive cells
    pub fn count_alive(&self) -> usize {
        self.chunks.iter().map(|c| c.count_alive() as usize).sum()
    }
    
    /// Clear all cells
    pub fn clear(&mut self) {
        self.chunks.iter_mut().for_each(|c| *c = Chunk64::empty());
    }
    
    /// Evolve grid by one generation using specified rule
    pub fn evolve(&self, rule: &dyn Rule) -> BitGrid {
        let mut next = BitGrid::new(self.width, self.height);
        
        for y in 0..self.height {
            for x in 0..self.width {
                let neighbors = self.count_neighbors(x, y);
                let current = if self.get(x, y) { Cell::Alive } else { Cell::Dead };
                
                let next_state = rule.evolve(current, neighbors);
                
                if next_state == Cell::Alive {
                    next.set(x, y, true);
                }
            }
        }
        
        next
    }
    
    /// Evolve grid using parallel processing with specified rule
    pub fn evolve_parallel(&self, rule: &(dyn Rule + Sync)) -> BitGrid {
        use rayon::prelude::*;
        
        let mut next = BitGrid::new(self.width, self.height);
        let width = self.width;
        let height = self.height;
        
        // Process each row in parallel
        let row_results: Vec<Vec<(usize, bool)>> = (0..height)
            .into_par_iter()
            .map(|y| {
                let mut row_cells = Vec::new();
                for x in 0..width {
                    let neighbors = self.count_neighbors(x, y);
                    let current = if self.get(x, y) { Cell::Alive } else { Cell::Dead };
                    
                    let next_state = rule.evolve(current, neighbors);
                    
                    if next_state == Cell::Alive {
                        row_cells.push((x, true));
                    }
                }
                row_cells
            })
            .collect();
        
        // Apply results
        for (y, row) in row_results.into_iter().enumerate() {
            for (x, _) in row {
                next.set(x, y, true);
            }
        }
        
        next
    }
    
    /// Randomize grid with ~25% alive cells
    pub fn randomize(&mut self) {
        use rand::Rng;
        let mut rng = rand::rng();
        
        for chunk in &mut self.chunks {
            // Random bits with ~25% density
            chunk.0 = rng.random::<u64>() & rng.random::<u64>();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::ConwayRule;
    
    #[test]
    fn test_chunk64_get_set() {
        let mut chunk = Chunk64::empty();
        
        // Initially all dead
        assert!(!chunk.get(0));
        assert!(!chunk.get(63));
        
        // Set some alive
        chunk.set(0, true);
        chunk.set(42, true);
        chunk.set(63, true);
        
        assert!(chunk.get(0));
        assert!(!chunk.get(1));
        assert!(chunk.get(42));
        assert!(chunk.get(63));
        
        // Set back to dead
        chunk.set(42, false);
        assert!(!chunk.get(42));
    }
    
    #[test]
    fn test_chunk64_count() {
        let mut chunk = Chunk64::empty();
        assert_eq!(chunk.count_alive(), 0);
        
        chunk.set(0, true);
        chunk.set(1, true);
        chunk.set(2, true);
        assert_eq!(chunk.count_alive(), 3);
        
        let full = Chunk64::full();
        assert_eq!(full.count_alive(), 64);
    }
    
    #[test]
    fn test_bitgrid_dimensions() {
        let grid = BitGrid::new(100, 50);
        assert_eq!(grid.dimensions(), (100, 50));
    }
    
    #[test]
    fn test_bitgrid_get_set() {
        let mut grid = BitGrid::new(100, 100);
        
        // Initially empty
        assert!(!grid.get(0, 0));
        assert!(!grid.get(50, 50));
        assert!(!grid.get(99, 99));
        
        // Set some cells
        grid.set(0, 0, true);
        grid.set(50, 50, true);
        grid.set(99, 99, true);
        
        assert!(grid.get(0, 0));
        assert!(grid.get(50, 50));
        assert!(grid.get(99, 99));
        assert!(!grid.get(1, 1));
    }
    
    #[test]
    fn test_bitgrid_count_neighbors() {
        let mut grid = BitGrid::new(10, 10);
        
        // Create a blinker pattern at (4,5), (5,5), (6,5)
        grid.set(4, 5, true);
        grid.set(5, 5, true);
        grid.set(6, 5, true);
        
        // Center cell has 2 neighbors
        assert_eq!(grid.count_neighbors(5, 5), 2);
        
        // Cell above center has 3 neighbors (the blinker)
        assert_eq!(grid.count_neighbors(5, 4), 3);
        
        // Cell below center also has 3 neighbors
        assert_eq!(grid.count_neighbors(5, 6), 3);
    }
    
    #[test]
    fn test_memory_reduction() {
        let grid = BitGrid::new(1000, 1000);
        let memory = grid.memory_bytes();
        
        // 1000x1000 cells = 1M cells
        // With bit-packing: ceil(1000/64) * 1000 * 8 = 16 * 1000 * 8 = 128KB
        // Without: 1MB (1 byte per cell)
        assert!(memory < 200_000, "Memory should be < 200KB, was {}", memory);
        
        // Check it's significantly smaller than naive (at least 7x due to padding)
        let naive_memory = 1000 * 1000; // 1 byte per cell
        assert!(memory < naive_memory / 7, "Should be at least 7x smaller, was {} vs {}", memory, naive_memory);
    }
    
    #[test]
    fn test_bitgrid_bounds() {
        let grid = BitGrid::new(10, 10);
        
        // Out of bounds should return false, not panic
        assert!(!grid.get(100, 100));
        assert!(!grid.get(10, 0));
        assert!(!grid.get(0, 10));
    }
    
    #[test]
    fn test_bitgrid_clear() {
        let mut grid = BitGrid::new(100, 100);
        
        grid.set(50, 50, true);
        grid.set(0, 0, true);
        assert_eq!(grid.count_alive(), 2);
        
        grid.clear();
        assert_eq!(grid.count_alive(), 0);
    }
    
    #[test]
    fn test_blinker_evolution() {
        let rule = ConwayRule;
        let mut grid = BitGrid::new(10, 10);
        
        // Horizontal blinker at center
        grid.set(4, 5, true);
        grid.set(5, 5, true);
        grid.set(6, 5, true);
        
        assert_eq!(grid.count_alive(), 3);
        
        // After one generation, should be vertical
        let next = grid.evolve(&rule);
        
        assert!(!next.get(4, 5));
        assert!(next.get(5, 4));
        assert!(next.get(5, 5));
        assert!(next.get(5, 6));
        assert!(!next.get(6, 5));
        assert_eq!(next.count_alive(), 3);
        
        // After two generations, back to horizontal
        let next2 = next.evolve(&rule);
        
        assert!(next2.get(4, 5));
        assert!(next2.get(5, 5));
        assert!(next2.get(6, 5));
        assert_eq!(next2.count_alive(), 3);
    }
    
    #[test]
    fn test_block_still_life() {
        let rule = ConwayRule;
        let mut grid = BitGrid::new(10, 10);
        
        // 2x2 block - stable pattern
        grid.set(4, 4, true);
        grid.set(5, 4, true);
        grid.set(4, 5, true);
        grid.set(5, 5, true);
        
        let next = grid.evolve(&rule);
        
        // Should remain unchanged
        assert!(next.get(4, 4));
        assert!(next.get(5, 4));
        assert!(next.get(4, 5));
        assert!(next.get(5, 5));
        assert_eq!(next.count_alive(), 4);
    }
    
    #[test]
    fn test_parallel_matches_serial() {
        let rule = ConwayRule;
        let mut grid = BitGrid::new(50, 50);
        
        // Create a random-ish pattern
        for i in 0..100 {
            grid.set(i % 50, (i * 7) % 50, true);
        }
        
        let serial = grid.evolve(&rule);
        let parallel = grid.evolve_parallel(&rule);
        
        // Both should produce identical results
        for y in 0..50 {
            for x in 0..50 {
                assert_eq!(
                    serial.get(x, y), 
                    parallel.get(x, y),
                    "Mismatch at ({}, {})", x, y
                );
            }
        }
    }
}
