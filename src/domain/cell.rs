/// Cell represents the fundamental unit in Conway's Game of Life.
/// Each cell can be either Dead or Alive.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Cell {
    Dead,
    Alive,
}

impl Cell {
    /// Check if the cell is currently alive
    pub const fn is_alive(self) -> bool {
        matches!(self, Cell::Alive)
    }
    
    /// Toggle the cell state (not used but kept for API completeness)
    #[allow(dead_code)]
    pub const fn toggle(self) -> Self {
        match self {
            Cell::Alive => Cell::Dead,
            Cell::Dead => Cell::Alive,
        }
    }
    
    /// Pure function to compute the next state based on Conway's rules:
    /// 1. Live cell with 2-3 neighbors survives
    /// 2. Dead cell with exactly 3 neighbors becomes alive
    /// 3. All other cases result in death
    pub const fn evolve(self, neighbors: u8) -> Self {
        match (self, neighbors) {
            (Cell::Alive, 2 | 3) => Cell::Alive,
            (Cell::Dead, 3) => Cell::Alive,
            _ => Cell::Dead,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_underpopulation() {
        assert_eq!(Cell::Alive.evolve(0), Cell::Dead);
        assert_eq!(Cell::Alive.evolve(1), Cell::Dead);
    }

    #[test]
    fn test_survival() {
        assert_eq!(Cell::Alive.evolve(2), Cell::Alive);
        assert_eq!(Cell::Alive.evolve(3), Cell::Alive);
    }

    #[test]
    fn test_overpopulation() {
        assert_eq!(Cell::Alive.evolve(4), Cell::Dead);
        assert_eq!(Cell::Alive.evolve(8), Cell::Dead);
    }

    #[test]
    fn test_reproduction() {
        assert_eq!(Cell::Dead.evolve(3), Cell::Alive);
    }
}
