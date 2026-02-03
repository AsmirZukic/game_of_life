use macroquad::prelude::*;
use crate::domain::{BitGrid, Pattern};
use crate::application::{GameState, Camera};
use crate::ui::{Button, Dropdown, panel_x, grid_area_width, grid_area_height, CELL_SIZE, PANEL_WIDTH};

/// Format large numbers with K/M/B suffixes
fn format_number(n: usize) -> String {
    if n >= 1_000_000_000 {
        format!("{:.1}B", n as f64 / 1_000_000_000.0)
    } else if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}K", n as f64 / 1_000.0)
    } else {
        format!("{}", n)
    }
}

/// Draw the cellular automaton grid with camera support
pub fn draw_grid(grid: &BitGrid, camera: &Camera) {
    let cell_size = CELL_SIZE * camera.zoom;
    let (grid_width, grid_height) = grid.dimensions();
    let area_width = grid_area_width();
    let area_height = grid_area_height();
    
    // Get visible bounds for culling
    let (min_x, min_y, max_x, max_y) = camera.visible_bounds(
        area_width,
        area_height,
        CELL_SIZE
    );
    
    // Clamp to grid bounds
    let start_x = min_x.max(0) as usize;
    let start_y = min_y.max(0) as usize;
    let end_x = (max_x + 1).min(grid_width as i32) as usize;
    let end_y = (max_y + 1).min(grid_height as i32) as usize;
    
    // Colors
    let alive_color = Color::from_rgba(0, 255, 150, 255); // Bright green
    let grid_line_color = Color::from_rgba(40, 40, 40, 255); // Dark gray
    let dead_cell_color = Color::from_rgba(15, 15, 15, 255); // Very dark gray for dead cells
    
    // Draw grid lines when zoomed in enough (before cells so cells draw on top)
    let draw_grid_lines = camera.zoom > 0.5 && cell_size >= 4.0;
    
    // Render all visible cells
    for y in start_y..end_y {
        for x in start_x..end_x {
            let (screen_x, screen_y) = camera.grid_to_screen(x, y, CELL_SIZE);
            
            // Skip if outside viewport
            if screen_x + cell_size < 0.0 || screen_x > area_width ||
               screen_y + cell_size < 0.0 || screen_y > area_height {
                continue;
            }
            
            if grid.get(x, y) {
                // Alive cell
                draw_rectangle(screen_x, screen_y, cell_size, cell_size, alive_color);
            } else if draw_grid_lines {
                // Dead cell - show faint background so grid is visible
                draw_rectangle(screen_x, screen_y, cell_size, cell_size, dead_cell_color);
            }
            
            // Draw grid lines if zoomed in enough
            if draw_grid_lines {
                draw_rectangle_lines(
                    screen_x,
                    screen_y,
                    cell_size,
                    cell_size,
                    1.0,
                    grid_line_color
                );
            }
        }
    }
}


/// Draw a semi-transparent preview of a pattern at the cursor position
pub fn draw_pattern_preview(pattern: &Pattern, camera: &Camera, mouse_pos: (f32, f32)) {
    let cell_size = CELL_SIZE * camera.zoom;
    
    // Calculate grid position centered on cursor
    let (grid_x, grid_y) = camera.screen_to_grid(mouse_pos.0, mouse_pos.1, CELL_SIZE);
    let start_x = grid_x - (pattern.width as i32 / 2);
    let start_y = grid_y - (pattern.height as i32 / 2);
    
    // Draw each alive cell of the pattern as a ghost
    for &(dx, dy) in &pattern.cells {
        let gx = start_x + dx as i32;
        let gy = start_y + dy as i32;
        
        if gx >= 0 && gy >= 0 {
            let (screen_x, screen_y) = camera.grid_to_screen(gx as usize, gy as usize, CELL_SIZE);
            
            // Semi-transparent green for preview
            draw_rectangle(
                screen_x, screen_y,
                cell_size, cell_size,
                Color::from_rgba(0, 255, 150, 120)  // 47% opacity
            );
            
            // Outline for clarity
            draw_rectangle_lines(
                screen_x, screen_y,
                cell_size, cell_size,
                1.5,
                Color::from_rgba(0, 255, 150, 200)
            );
        }
    }
    
    // Draw bounding box around entire pattern
    if start_x >= 0 && start_y >= 0 {
        let (box_x, box_y) = camera.grid_to_screen(start_x as usize, start_y as usize, CELL_SIZE);
        draw_rectangle_lines(
            box_x, box_y,
            pattern.width as f32 * cell_size,
            pattern.height as f32 * cell_size,
            2.0,
            Color::from_rgba(255, 255, 0, 180)  // Yellow outline
        );
    }
}

/// Draw control panel background
fn draw_panel_background() {
    draw_rectangle(
        panel_x(),
        0.0,
        PANEL_WIDTH,
        screen_height(),
        Color::from_rgba(30, 30, 30, 255)
    );
}

/// Helper to draw text labels
fn draw_text_label(text: &str, x: f32, y: f32, size: f32, color: Color) {
    draw_text(text, x, y, size, color);
}

/// Draw the control panel with buttons, dropdowns, and info
pub fn draw_controls(
    state: &GameState,
    camera: &Camera,
    buttons: &[Button],
    dropdowns: &[Dropdown],
    mouse_pos: (f32, f32)
) {
    draw_panel_background();
    
    // Draw all buttons FIRST
    buttons.iter().for_each(|btn| btn.draw(mouse_pos));
    
    let px = panel_x();
    
    // Controls help - positioned below dropdowns (after pattern at ~170+50)
    let controls = [
        ("Controls:", px, 240.0, 14.0, WHITE),
        ("LMB: Paint", px, 255.0, 12.0, GRAY),
        ("RMB: Erase", px, 268.0, 12.0, GRAY),
        ("Space: Play", px, 281.0, 12.0, GRAY),
        ("Wheel: Zoom", px, 294.0, 12.0, GRAY),
        ("Mid-drag: Pan", px, 307.0, 12.0, GRAY),
    ];
    
    controls.iter().for_each(|(text, x, y, size, color)| {
        draw_text_label(text, *x, *y, *size, *color);
    });
    
    // Grid info
    let (gw, gh) = state.grid.dimensions();
    let cells = gw * gh;
    let grid_info = format!("Grid: {}×{}\nCells: {}", gw, gh, format_number(cells));
    draw_text_label(&grid_info, px, 335.0, 12.0, Color::from_rgba(150, 150, 150, 255));
    
    // Performance metrics with algorithm name
    let evolve_ms = state.last_evolution_time_ms;
    let fps = get_fps();
    let algo_name = state.algorithm.name();
    
    // Color code the evolution time
    let perf_color = if evolve_ms < 5.0 {
        Color::from_rgba(0, 255, 0, 255)  // Green: good
    } else if evolve_ms < 33.0 {
        Color::from_rgba(255, 255, 0, 255)  // Yellow: okay
    } else if evolve_ms < 100.0 {
        Color::from_rgba(255, 165, 0, 255)  // Orange: slow
    } else {
        Color::from_rgba(255, 0, 0, 255)  // Red: very slow
    };
    
    // Color code render time too
    let render_ms = state.last_render_time_ms;
    let render_color = if render_ms < 5.0 {
        Color::from_rgba(0, 255, 0, 255)
    } else if render_ms < 16.0 {
        Color::from_rgba(255, 255, 0, 255)
    } else if render_ms < 50.0 {
        Color::from_rgba(255, 165, 0, 255)
    } else {
        Color::from_rgba(255, 0, 0, 255)
    };
    
    draw_text_label(&format!("Evolve: {:.1}ms", evolve_ms), px, 370.0, 13.0, perf_color);
    draw_text_label(&format!("Render: {:.1}ms", render_ms), px, 385.0, 13.0, render_color);
    draw_text_label(&format!("{} | FPS: {:.0}", algo_name, fps), px, 400.0, 12.0, GRAY);
    
    // Cells per second (throughput)
    if evolve_ms > 0.0 && state.is_running {
        let cells_per_sec = (cells as f64) / (evolve_ms as f64 / 1000.0);
        draw_text_label(
            &format!("Throughput: {}/s", format_number(cells_per_sec as usize)),
            px, 415.0, 11.0, 
            Color::from_rgba(100, 200, 255, 255)
        );
    }
    
    // Define all labels declaratively
    let labels = [
        ("Speed:", px, 630.0, 16.0, WHITE),
        (
            &format!("{:.0} gen/s", state.updates_per_second),
            px, 650.0, 14.0,
            Color::from_rgba(180, 180, 180, 255)
        ),
        ("Generation:", px, 680.0, 16.0, WHITE),
        (
            &format!("{}", state.generation),
            px, 700.0, 20.0,
            Color::from_rgba(0, 255, 150, 255)
        ),
        ("Status:", px, 735.0, 16.0, WHITE),
        (
            if state.is_running { "Running" } else { "Paused" },
            px,
            755.0,
            16.0,
            if state.is_running {
                Color::from_rgba(0, 255, 0, 255)
            } else {
                Color::from_rgba(255, 165, 0, 255)
            }
        ),
        ("Zoom:", px, 780.0, 14.0, WHITE),
        (
            &format!("{:.1}x", camera.zoom),
            px, 795.0, 14.0,
            Color::from_rgba(180, 180, 180, 255)
        ),
    ];
    
    // Draw all labels
    labels.iter().for_each(|(text, x, y, size, color)|  {
        draw_text_label(text, *x, *y, *size, *color);
    });
    
    // Draw dropdowns LAST so they appear on top of everything
    // Draw closed dropdowns first, then open one on top
    let mut open_dropdown: Option<&Dropdown> = None;
    for dropdown in dropdowns.iter() {
        if dropdown.is_open() {
            open_dropdown = Some(dropdown);
        } else {
            dropdown.draw(mouse_pos);
        }
    }
    // Draw open dropdown last so it's on top
    if let Some(dd) = open_dropdown {
        dd.draw(mouse_pos);
    }
}

