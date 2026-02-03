mod button;
mod dropdown;

pub use button::Button;
pub use dropdown::Dropdown;

// UI constants - now functions for responsive layout
use macroquad::prelude::{screen_width, screen_height};

pub const PANEL_WIDTH: f32 = 180.0;
pub const BUTTON_HEIGHT: f32 = 40.0;
pub const CELL_SIZE: f32 = 10.0;

/// Get the X position where the panel starts (right side)
pub fn panel_x() -> f32 {
    screen_width() - PANEL_WIDTH
}

/// Get the width of the grid area
pub fn grid_area_width() -> f32 {
    screen_width() - PANEL_WIDTH
}

/// Get the height of the grid area
pub fn grid_area_height() -> f32 {
    screen_height()
}

/// Grid size options - including stress test sizes
pub const GRID_SIZES: &[(usize, &str)] = &[
    (50, "50×50"),
    (100, "100×100"),
    (200, "200×200"),
    (500, "500×500"),
    (1000, "1000×1000"),
    (2000, "2K×2K"),
    (5000, "5K×5K"),
    (10000, "10K×10K"),
];

/// Algorithm names for dropdown - matches Algorithm::all() order
/// Explicit naming: Naive (1 byte/cell), BitPacked (1 bit/cell), BitSIMD (bit-packed + SIMD)
pub const ALGORITHMS: &[&str] = &[
    "Naive",
    "Naive+Par",
    "BitPacked",
    "BitPacked+Par",
    "BitSIMD",
    "BitSIMD+Par",
    "TempBlock",
    "TempBlock+Par",
];

/// Create UI buttons with standard layout
/// Button positions adjusted to make room for algorithm dropdown
pub fn create_buttons() -> Vec<Button> {
    let px = panel_x();
    vec![
        Button::new(px, 470.0, PANEL_WIDTH, BUTTON_HEIGHT, "Play/Pause"),
        Button::new(px, 520.0, PANEL_WIDTH, BUTTON_HEIGHT, "Clear"),
        Button::new(px, 570.0, PANEL_WIDTH, BUTTON_HEIGHT, "Random"),
    ]
}

