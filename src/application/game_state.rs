use crate::domain::{BitGrid, Grid, Cell, Rule, Algorithm, default_rule, simd_life, temporal_blocking};

/// GameState orchestrates the simulation.
/// This is the application layer that coordinates domain logic.
pub struct GameState {
    pub grid: BitGrid,
    pub rule: Box<dyn Rule + Send + Sync>,
    pub algorithm: Algorithm,
    pub is_running: bool,
    pub generation: u64,
    pub update_timer: f32,
    pub updates_per_second: f32,
    pub last_evolution_time_ms: f32,  // Evolution performance metric
    pub last_render_time_ms: f32,     // Render performance metric
    /// Index of pattern pending placement (None = normal mode)
    pub pending_pattern_index: Option<usize>,
}

impl GameState {
    /// Create new game state with given grid dimensions
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            grid: BitGrid::new(width, height),
            rule: default_rule(),
            algorithm: Algorithm::default(),
            is_running: false,
            generation: 0,
            update_timer: 0.0,
            updates_per_second: 10.0,
            last_evolution_time_ms: 0.0,
            last_render_time_ms: 0.0,
            pending_pattern_index: None,
        }
    }
    
    /// Resize grid to new dimensions
    pub fn resize_grid(&mut self, width: usize, height: usize) {
        self.grid = BitGrid::new(width, height);
        self.generation = 0;
        self.is_running = false;
    }
    
    /// Set the cellular automaton rule
    pub fn set_rule(&mut self, rule: Box<dyn Rule + Send + Sync>) {
        self.rule = rule;
    }
    
    /// Set the evolution algorithm
    pub fn set_algorithm(&mut self, algorithm: Algorithm) {
        self.algorithm = algorithm;
    }
    
    /// Set running state (builder pattern)
    #[allow(dead_code)]
    pub fn with_running(mut self, running: bool) -> Self {
        self.is_running = running;
        self
    }
    
    /// Toggle play/pause state
    pub fn toggle_running(mut self) -> Self {
        self.is_running = !self.is_running;
        self
    }
    
    /// Clear grid and reset generation counter
    pub fn clear(mut self) -> Self {
        self.grid.clear();
        self.generation = 0;
        self.is_running = false;
        self
    }
    
    /// Randomize grid and reset generation counter
    pub fn randomize(mut self) -> Self {
        self.grid.randomize();
        self.generation = 0;
        self.is_running = false;
        self
    }
    
    /// Adjust simulation speed
    pub fn adjust_speed(mut self, delta: f32) -> Self {
        self.updates_per_second = (self.updates_per_second + delta).clamp(1.0, 60.0);
        self
    }
    
    /// Update simulation by one frame
    /// This is the main game loop coordination
    pub fn tick(mut self, delta_time: f32) -> Self {
        if !self.is_running {
            return self;
        }
        
        self.update_timer += delta_time;
        let update_interval = 1.0 / self.updates_per_second;
        
        if self.update_timer >= update_interval {
            // Measure evolution time
            let start = std::time::Instant::now();
            
            // Dispatch to selected algorithm
            self.grid = match self.algorithm {
                Algorithm::Original => {
                    let grid = Self::bitgrid_to_grid(&self.grid);
                    let evolved = grid.evolve(self.rule.as_ref());
                    Self::grid_to_bitgrid(&evolved)
                }
                Algorithm::OriginalParallel => {
                    let grid = Self::bitgrid_to_grid(&self.grid);
                    let evolved = grid.evolve_parallel(self.rule.as_ref());
                    Self::grid_to_bitgrid(&evolved)
                }
                Algorithm::BitGridNaive => {
                    self.grid.evolve(self.rule.as_ref())
                }
                Algorithm::BitGridNaiveParallel => {
                    self.grid.evolve_parallel(self.rule.as_ref())
                }
                Algorithm::Simd => {
                    simd_life::evolve_simd(&self.grid, self.rule.as_ref())
                }
                Algorithm::SimdParallel => {
                    simd_life::evolve_simd_parallel(&self.grid, self.rule.as_ref())
                }
                Algorithm::TemporalBlocking => {
                    temporal_blocking::evolve_temporal_blocking(&self.grid, self.rule.as_ref(), 4)
                }
                Algorithm::TemporalBlockingParallel => {
                    temporal_blocking::evolve_temporal_blocking_parallel(&self.grid, self.rule.as_ref(), 4)
                }
            };
            
            self.last_evolution_time_ms = start.elapsed().as_secs_f32() * 1000.0;
            self.generation += 1;
            self.update_timer = 0.0;
        }
        
        self
    }
    
    /// Convert BitGrid to Grid for Original algorithms
    fn bitgrid_to_grid(bg: &BitGrid) -> Grid {
        let (w, h) = bg.dimensions();
        let mut grid = Grid::new(w, h);
        for y in 0..h {
            for x in 0..w {
                if bg.get(x, y) {
                    grid.set(x, y, Cell::Alive);
                }
            }
        }
        grid
    }
    
    /// Convert Grid to BitGrid after evolution
    fn grid_to_bitgrid(g: &Grid) -> BitGrid {
        let (w, h) = g.dimensions();
        let mut bg = BitGrid::new(w, h);
        for y in 0..h {
            for x in 0..w {
                if g.get(x, y) == Some(Cell::Alive) {
                    bg.set(x, y, true);
                }
            }
        }
        bg
    }
}
