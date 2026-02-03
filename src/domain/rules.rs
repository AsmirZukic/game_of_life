use super::Cell;

/// Trait for cellular automaton rules
/// Enables different rulesets beyond Conway's Game of Life
pub trait Rule: Send + Sync {
    /// Name of the rule
    fn name(&self) -> &'static str;
    
    /// Short description
    fn description(&self) -> &'static str;
    
    /// Apply rule to compute next cell state
    fn evolve(&self, current: Cell, neighbors: u8) -> Cell;
}

/// Conway's Game of Life (B3/S23)
/// The classic cellular automaton rules
#[derive(Clone, Copy)]
pub struct ConwayRule;

impl Rule for ConwayRule {
    fn name(&self) -> &'static str {
        "Conway"
    }
    
    fn description(&self) -> &'static str {
        "B3/S23 - Classic"
    }
    
    fn evolve(&self, current: Cell, neighbors: u8) -> Cell {
        match (current, neighbors) {
            (Cell::Alive, 2 | 3) => Cell::Alive,
            (Cell::Dead, 3) => Cell::Alive,
            _ => Cell::Dead,
        }
    }
}

/// HighLife (B36/S23)
/// Like Conway's Life but cells with 6 neighbors are born
/// Creates replicators - patterns that create copies of themselves
#[derive(Clone, Copy)]
pub struct HighLifeRule;

impl Rule for HighLifeRule {
    fn name(&self) -> &'static str {
        "HighLife"
    }
    
    fn description(&self) -> &'static str {
        "B36/S23 - Replicators"
    }
    
    fn evolve(&self, current: Cell, neighbors: u8) -> Cell {
        match (current, neighbors) {
            (Cell::Alive, 2 | 3) => Cell::Alive,
            (Cell::Dead, 3 | 6) => Cell::Alive,
            _ => Cell::Dead,
        }
    }
}

/// Seeds (B2/S)
/// Every cell dies each generation
/// Creates expanding patterns
#[derive(Clone, Copy)]
pub struct SeedsRule;

impl Rule for SeedsRule {
    fn name(&self) -> &'static str {
        "Seeds"
    }
    
    fn description(&self) -> &'static str {
        "B2/S - Exploding"
    }
    
    fn evolve(&self, current: Cell, neighbors: u8) -> Cell {
        match (current, neighbors) {
            (Cell::Dead, 2) => Cell::Alive,
            _ => Cell::Dead,
        }
    }
}

/// Day & Night (B3678/S34678)
/// Symmetric rule - inverse of a pattern follows same rules
#[derive(Clone, Copy)]
pub struct DayAndNightRule;

impl Rule for DayAndNightRule {
    fn name(&self) -> &'static str {
        "Day&Night"
    }
    
    fn description(&self) -> &'static str {
        "B3678/S34678"
    }
    
    fn evolve(&self, current: Cell, neighbors: u8) -> Cell {
        match (current, neighbors) {
            (Cell::Alive, 3 | 4 | 6 | 7 | 8) => Cell::Alive,
            (Cell::Dead, 3 | 6 | 7 | 8) => Cell::Alive,
            _ => Cell::Dead,
        }
    }
}

/// Get all available rules
pub fn all_rules() -> Vec<(&'static str, Box<dyn Rule>)> {
    vec![
        ("Conway", Box::new(ConwayRule) as Box<dyn Rule>),
        ("HighLife", Box::new(HighLifeRule)),
        ("Seeds", Box::new(SeedsRule)),
        ("Day&Night", Box::new(DayAndNightRule)),
    ]
}

/// Get default rule (Conway's Life)
pub fn default_rule() -> Box<dyn Rule> {
    Box::new(ConwayRule)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conway_rules() {
        let rule = ConwayRule;
        
        // Underpopulation
        assert_eq!(rule.evolve(Cell::Alive, 0), Cell::Dead);
        assert_eq!(rule.evolve(Cell::Alive, 1), Cell::Dead);
        
        // Survival
        assert_eq!(rule.evolve(Cell::Alive, 2), Cell::Alive);
        assert_eq!(rule.evolve(Cell::Alive, 3), Cell::Alive);
        
        // Overpopulation
        assert_eq!(rule.evolve(Cell::Alive, 4), Cell::Dead);
        
        // Reproduction
        assert_eq!(rule.evolve(Cell::Dead, 3), Cell::Alive);
    }

    #[test]
    fn test_highlife_reproduction() {
        let rule = HighLifeRule;
        
        // HighLife specific: birth with 6 neighbors
        assert_eq!(rule.evolve(Cell::Dead, 6), Cell::Alive);
        assert_eq!(rule.evolve(Cell::Dead, 3), Cell::Alive);
    }

    #[test]
    fn test_seeds_always_dies() {
        let rule = SeedsRule;
        
        // All living cells die
        assert_eq!(rule.evolve(Cell::Alive, 0), Cell::Dead);
        assert_eq!(rule.evolve(Cell::Alive, 2), Cell::Dead);
        assert_eq!(rule.evolve(Cell::Alive, 8), Cell::Dead);
        
        // Only born with 2 neighbors
        assert_eq!(rule.evolve(Cell::Dead, 2), Cell::Alive);
        assert_eq!(rule.evolve(Cell::Dead, 3), Cell::Dead);
    }
}
