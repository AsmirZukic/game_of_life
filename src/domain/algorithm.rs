//! Algorithm enum for selecting evolution implementation.
//!
//! This module provides a unified way to select between different
//! Game of Life evolution algorithms for demo and benchmarking purposes.

/// Available evolution algorithms for demo comparison.
/// Each algorithm trades off between speed and flexibility.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Algorithm {
    /// Original cell-by-cell with Vec<Cell>, serial processing
    Original,
    /// Original with parallel rayon
    OriginalParallel,
    /// Bit-packed grid, cell-by-cell evolution
    BitGridNaive,
    /// Bit-packed with parallelization
    BitGridNaiveParallel,
    /// SIMD bit manipulation (serial)
    Simd,
    /// SIMD with parallel rayon
    SimdParallel,
    /// Temporal blocking - multiple generations per tile (serial)
    TemporalBlocking,
    /// Temporal blocking with parallel tiles (fastest for large grids)
    #[default]
    TemporalBlockingParallel,
}

impl Algorithm {
    /// Get all available algorithms
    pub fn all() -> Vec<Algorithm> {
        vec![
            Algorithm::Original,
            Algorithm::OriginalParallel,
            Algorithm::BitGridNaive,
            Algorithm::BitGridNaiveParallel,
            Algorithm::Simd,
            Algorithm::SimdParallel,
            Algorithm::TemporalBlocking,
            Algorithm::TemporalBlockingParallel,
        ]
    }
    
    /// Display name for UI - explicit about storage and strategy
    pub fn name(&self) -> &'static str {
        match self {
            Algorithm::Original => "Naive",
            Algorithm::OriginalParallel => "Naive+Par",
            Algorithm::BitGridNaive => "BitPacked",
            Algorithm::BitGridNaiveParallel => "BitPacked+Par",
            Algorithm::Simd => "BitSIMD",
            Algorithm::SimdParallel => "BitSIMD+Par",
            Algorithm::TemporalBlocking => "TempBlock",
            Algorithm::TemporalBlockingParallel => "TempBlock+Par",
        }
    }
    
    /// Short description for tooltips/info
    pub fn description(&self) -> &'static str {
        match self {
            Algorithm::Original => "Cell enum array, 1 byte/cell, serial",
            Algorithm::OriginalParallel => "Cell enum array, 1 byte/cell, parallel",
            Algorithm::BitGridNaive => "Bit-packed 1 bit/cell, cell-by-cell",
            Algorithm::BitGridNaiveParallel => "Bit-packed 1 bit/cell, parallel rows",
            Algorithm::Simd => "Bit-packed + 64 cells at once",
            Algorithm::SimdParallel => "Bit-packed + 64 cells at once + parallel",
            Algorithm::TemporalBlocking => "4 gens/tile, reduced memory traffic",
            Algorithm::TemporalBlockingParallel => "4 gens/tile, parallel tiles",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_all_algorithms_returns_eight() {
        assert_eq!(Algorithm::all().len(), 8);
    }
    
    #[test]
    fn test_default_is_temporal_blocking_parallel() {
        assert_eq!(Algorithm::default(), Algorithm::TemporalBlockingParallel);
    }
    
    #[test]
    fn test_names_are_unique() {
        let names: Vec<_> = Algorithm::all().iter().map(|a| a.name()).collect();
        let mut unique = names.clone();
        unique.sort();
        unique.dedup();
        assert_eq!(names.len(), unique.len());
    }
}
