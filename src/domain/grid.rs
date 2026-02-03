use super::{Cell, rules::Rule};
use rayon::prelude::*;

/// Grid manages the 2D cellular automaton grid.
/// Uses functional, immutable updates for predictable state transitions.
pub struct Grid {
    width: usize,
    height: usize,
    cells: Vec<Cell>,
}

impl Grid {
    /// Create a new grid with all cells initially dead
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            cells: vec![Cell::Dead; width * height],
        }
    }
    
    /// Get grid dimensions
    pub const fn dimensions(&self) -> (usize, usize) {
        (self.width, self.height)
    }
    
    /// Convert 2D coordinates to 1D index
    const fn get_index(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }
    
    /// Get cell at position (with bounds checking)
    pub fn get(&self, x: usize, y: usize) -> Option<Cell> {
        (x < self.width && y < self.height)
            .then(|| self.cells[self.get_index(x, y)])
    }
    
    /// Set cell at position (mutable for painting)
    pub fn set(&mut self, x: usize, y: usize, cell: Cell) {
        if x < self.width && y < self.height {
            let idx = self.get_index(x, y);
            self.cells[idx] = cell;
        }
    }
    
    /// Count live neighbors using toroidal wrapping (grid wraps like a torus)
    fn count_live_neighbors(&self, x: usize, y: usize) -> u8 {
        let w = self.width as i32;
        let h = self.height as i32;
        
        (-1..=1)
            .flat_map(|dy| (-1..=1).map(move |dx| (dx, dy)))
            .filter(|&(dx, dy)| dx != 0 || dy != 0)
            .map(|(dx, dy)| {
                // Toroidal wrapping
                let nx = ((x as i32 + dx) % w + w) % w;
                let ny = ((y as i32 + dy) % h + h) % h;
                self.get(nx as usize, ny as usize).unwrap()
            })
            .filter(|cell| cell.is_alive())
            .count() as u8
    }
    
    /// Pure functional evolution - returns new grid (serial)
    pub fn evolve(&self, rule: &dyn Rule) -> Self {
        let cells = (0..self.height)
            .flat_map(|y| (0..self.width).map(move |x| (x, y)))
            .map(|(x, y)| {
                let current = self.get(x, y).unwrap();
                let neighbors = self.count_live_neighbors(x, y);
                rule.evolve(current, neighbors)
            })
            .collect();
        
        Self {
            width: self.width,
            height: self.height,
            cells,
        }
    }
    
    /// Parallel evolution using rayon for large grids
    /// Much faster for grids > 100x100
    pub fn evolve_parallel(&self, rule: &(dyn Rule + Sync)) -> Self {
        let cells: Vec<Cell> = (0..self.height)
            .into_par_iter()
            .flat_map(|y| {
                (0..self.width).into_par_iter().map(move |x| (x, y))
            })
            .map(|(x, y)| {
                let current = self.get(x, y).unwrap();
                let neighbors = self.count_live_neighbors(x, y);
                rule.evolve(current, neighbors)
            })
            .collect();
        
        Self {
            width: self.width,
            height: self.height,
            cells,
        }
    }
    
    /// Clear all cells to dead state
    pub fn clear(mut self) -> Self {
        self.cells.iter_mut().for_each(|cell| *cell = Cell::Dead);
        self
    }
    
    /// Randomize grid (30% chance of alive)
    pub fn randomize(mut self) -> Self {
        use macroquad::rand;
        
        self.cells.iter_mut().for_each(|cell| {
            *cell = if rand::gen_range(0.0, 1.0) < 0.3 {
                Cell::Alive
            } else {
                Cell::Dead
            };
        });
        self
    }
    
    /// Iterate over all cells with their positions
    pub fn iter_cells(&self) -> impl Iterator<Item = (usize, usize, Cell)> + '_ {
        (0..self.height)
            .flat_map(move |y| (0..self.width).map(move |x| (x, y)))
            .map(|(x, y)| (x, y, self.get(x, y).unwrap()))
    }
}
