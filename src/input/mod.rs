use macroquad::prelude::*;
// use crate::domain::Cell;
use crate::application::{GameState, Camera};
use crate::ui::{grid_area_width, CELL_SIZE};

/// Handle zoom with mouse wheel
pub fn handle_zoom(camera: &mut Camera) {
    let wheel = mouse_wheel().1;
    if wheel > 0.0 {
        camera.zoom_in(1.1);
    } else if wheel < 0.0 {
        camera.zoom_out(1.1);
    }
}

/// Handle pan with middle mouse button drag
pub fn handle_pan(camera: &mut Camera, mouse_pos: (f32, f32)) {
    // Use static for tracking last position
    static mut LAST_POS: Option<(f32, f32)> = None;
    
    unsafe {
        if is_mouse_button_down(MouseButton::Middle) {
            if let Some(last) = LAST_POS {
                let dx = mouse_pos.0 - last.0;
                let dy = mouse_pos.1 - last.1;
                camera.pan(dx, dy);
            }
            LAST_POS = Some(mouse_pos);
        } else {
            LAST_POS = None;
        }
    }
}

/// Handle mouse painting on the grid (with camera support)
pub fn handle_mouse_paint(state: &mut GameState, camera: &Camera, mouse_pos: (f32, f32)) {
    if state.is_running || mouse_pos.0 >= grid_area_width() {
        return;
    }
    
    // Convert screen coordinates to grid coordinates using camera
    let (grid_x, grid_y) = camera.screen_to_grid(mouse_pos.0, mouse_pos.1, CELL_SIZE);
    
    // Check if within grid bounds
    let (grid_width, grid_height) = state.grid.dimensions();
    if grid_x < 0 || grid_y < 0 || grid_x >= grid_width as i32 || grid_y >= grid_height as i32 {
        return;
    }
    
    let (gx, gy) = (grid_x as usize, grid_y as usize);
    
    if is_mouse_button_down(MouseButton::Left) {
        state.grid.set(gx, gy, true);
    } else if is_mouse_button_down(MouseButton::Right) {
        state.grid.set(gx, gy, false);
    }
}

/// Process keyboard input functionally
pub fn process_keyboard_input(state: GameState, camera: &mut Camera) -> GameState {
    type KeyAction = (KeyCode, fn(GameState) -> GameState);
    
    let actions: [KeyAction; 5] = [
        (KeyCode::Space, GameState::toggle_running),
        (KeyCode::C, GameState::clear),
        (KeyCode::R, GameState::randomize),
        (KeyCode::Up, |s| s.adjust_speed(1.0)),
        (KeyCode::Down, |s| s.adjust_speed(-1.0)),
    ];
    
    let new_state = actions.iter().fold(state, |s, (key, action)| {
        if is_key_pressed(*key) { action(s) } else { s }
    });
    
    // Reset camera with 'H' (home)
    if is_key_pressed(KeyCode::H) {
        camera.reset();
    }
    
    new_state
}

/// Process button clicks functionally
pub fn process_button_clicks(
    state: GameState,
    buttons: &[crate::ui::Button],
    mouse_pos: (f32, f32)
) -> GameState {
    buttons
        .iter()
        .enumerate()
        .fold(state, |s, (idx, btn)| {
            if !btn.is_clicked(mouse_pos) {
                return s;
            }
            match idx {
                0 => s.toggle_running(),
                1 => s.clear(),
                2 => s.randomize(),
                _ => s,
            }
        })
}
