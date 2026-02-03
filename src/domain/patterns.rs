// use super::{Cell, Grid};

/// Represents a pattern that can be placed on the grid
#[derive(Clone)]
pub struct Pattern {
    pub name: &'static str,
    pub description: &'static str,
    pub width: usize,
    pub height: usize,
    pub cells: Vec<(usize, usize)>,  // Relative coordinates of alive cells
}

impl Pattern {
    /// Create a new pattern from alive cell coordinates
    pub fn new(name: &'static str, description: &'static str, cells: Vec<(usize, usize)>) -> Self {
        let width = cells.iter().map(|(x, _)| *x).max().unwrap_or(0) + 1;
        let height = cells.iter().map(|(_, y)| *y).max().unwrap_or(0) + 1;
        Self { name, description, width, height, cells }
    }
    
    /// Place pattern on grid at specified position
    pub fn place_on(&self, grid: &mut super::BitGrid, x: usize, y: usize) {
        for (dx, dy) in &self.cells {
            grid.set(x + dx, y + dy, true);
        }
    }
}

/// Classic Game of Life patterns library
pub mod presets {
    use super::*;
    
    /// Glider - simplest spaceship, moves diagonally
    pub fn glider() -> Pattern {
        Pattern::new(
            "Glider",
            "Moves diagonally (period 4)",
            vec![
                (1, 0),
                (2, 1),
                (0, 2), (1, 2), (2, 2),
            ]
        )
    }
    
    /// Blinker - period 2 oscillator
    pub fn blinker() -> Pattern {
        Pattern::new(
            "Blinker",
            "Oscillator (period 2)",
            vec![
                (0, 1), (1, 1), (2, 1),
            ]
        )
    }
    
    /// Toad - period 2 oscillator
    pub fn toad() -> Pattern {
        Pattern::new(
            "Toad",
            "Oscillator (period 2)",
            vec![
                (1, 0), (2, 0), (3, 0),
                (0, 1), (1, 1), (2, 1),
            ]
        )
    }
    
    /// Beacon - period 2 oscillator
    pub fn beacon() -> Pattern {
        Pattern::new(
            "Beacon",
            "Oscillator (period 2)",
            vec![
                (0, 0), (1, 0),
                (0, 1),
                (3, 2),
                (2, 3), (3, 3),
            ]
        )
    }
    
    /// Pulsar - period 3 oscillator
    pub fn pulsar() -> Pattern {
        Pattern::new(
            "Pulsar",
            "Oscillator (period 3)",
            vec![
                // Top
                (2, 0), (3, 0), (4, 0), (8, 0), (9, 0), (10, 0),
                // Upper middle
                (0, 2), (5, 2), (7, 2), (12, 2),
                (0, 3), (5, 3), (7, 3), (12, 3),
                (0, 4), (5, 4), (7, 4), (12, 4),
                // Center
                (2, 5), (3, 5), (4, 5), (8, 5), (9, 5), (10, 5),
                (2, 7), (3, 7), (4, 7), (8, 7), (9, 7), (10, 7),
                // Lower middle
                (0, 8), (5, 8), (7, 8), (12, 8),
                (0, 9), (5, 9), (7, 9), (12, 9),
                (0, 10), (5, 10), (7, 10), (12, 10),
                // Bottom
                (2, 12), (3, 12), (4, 12), (8, 12), (9, 12), (10, 12),
            ]
        )
    }
    
    /// Lightweight Spaceship (LWSS)
    pub fn lwss() -> Pattern {
        Pattern::new(
            "LWSS",
            "Lightweight Spaceship (period 4)",
            vec![
                (1, 0), (4, 0),
                (0, 1),
                (0, 2), (4, 2),
                (0, 3), (1, 3), (2, 3), (3, 3),
            ]
        )
    }
    
    /// Gosper Glider Gun - produces gliders indefinitely
    pub fn glider_gun() -> Pattern {
        Pattern::new(
            "Gosper Glider Gun",
            "Produces gliders (period 30)",
            vec![
                // Left square
                (0, 4), (0, 5),
                (1, 4), (1, 5),
                
                // Left circle
                (10, 4), (10, 5), (10, 6),
                (11, 3), (11, 7),
                (12, 2), (12, 8),
                (13, 2), (13, 8),
                (14, 5),
                (15, 3), (15, 7),
                (16, 4), (16, 5), (16, 6),
                (17, 5),
                
                // Middle pieces
                (20, 2), (20, 3), (20, 4),
                (21, 2), (21, 3), (21, 4),
                (22, 1), (22, 5),
                (24, 0), (24, 1), (24, 5), (24, 6),
                
                // Right square
                (34, 2), (34, 3),
                (35, 2), (35, 3),
            ]
        )
    }
    
    /// R-pentomino - classic methuselah (stabilizes after 1103 generations)
    pub fn r_pentomino() -> Pattern {
        Pattern::new(
            "R-pentomino",
            "Methuselah - stabilizes at gen 1103",
            vec![
                (1, 0), (2, 0),
                (0, 1), (1, 1),
                (1, 2),
            ]
        )
    }
    
    /// Acorn - small methuselah that stabilizes after 5206 generations
    pub fn acorn() -> Pattern {
        Pattern::new(
            "Acorn",
            "Methuselah - stabilizes at gen 5206",
            vec![
                (1, 0),
                (3, 1),
                (0, 2), (1, 2), (4, 2), (5, 2), (6, 2),
            ]
        )
    }
    
    /// Block - simple still life
    pub fn block() -> Pattern {
        Pattern::new(
            "Block",
            "Still life",
            vec![
                (0, 0), (1, 0),
                (0, 1), (1, 1),
            ]
        )
    }
    
    /// Get all available patterns
    pub fn all_patterns() -> Vec<Pattern> {
        vec![
            glider(),
            blinker(),
            toad(),
            beacon(),
            pulsar(),
            lwss(),
            glider_gun(),
            r_pentomino(),
            acorn(),
            block(),
        ]
    }
}
