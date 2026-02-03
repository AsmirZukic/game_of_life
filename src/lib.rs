// Domain layer - Core business logic
pub mod domain;

// Application layer - Use cases and coordination  
pub mod application;

// Infrastructure layer - UI, rendering, input
pub mod ui;
pub mod rendering;
pub mod input;

// Re-exports for convenience
pub use domain::{Cell, Grid, Pattern, presets, Algorithm};
pub use application::{GameState, Camera};
pub use ui::Button;

