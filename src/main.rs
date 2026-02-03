use macroquad::prelude::*;
use game_of_life::{
    GameState, Camera, presets, Algorithm,
    domain::all_rules,
    ui::{self, Dropdown, GRID_SIZES, ALGORITHMS},
    rendering, input,
};

fn window_conf() -> Conf {
    Conf {
        window_title: "Conway's Game of Life - Algorithm Demo".to_owned(),
        window_width: 1000,
        window_height: 800,
        window_resizable: true,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    // Initialize with medium grid
    let mut state = GameState::new(100, 100);
    let mut camera = Camera::new();
    
    // Create dropdowns - simple vertical stack at top
    let px = ui::panel_x();
    let grid_size_items: Vec<String> = GRID_SIZES.iter().map(|(_, name)| name.to_string()).collect();
    let mut grid_size_dropdown = Dropdown::new(
        px, 
        20.0,
        ui::PANEL_WIDTH,
        "Grid Size",
        grid_size_items
    );
    grid_size_dropdown.set_selected(1); // Start with 100x100
    
    // Rules dropdown
    let rules = all_rules();
    let rule_items: Vec<String> = rules.iter().map(|(name, _)| name.to_string()).collect();
    let mut rule_dropdown = Dropdown::new(
        px,
        70.0,
        ui::PANEL_WIDTH,
        "Rule",
        rule_items
    );
    
    // Algorithm dropdown - NEW
    let algorithm_items: Vec<String> = ALGORITHMS.iter().map(|s| s.to_string()).collect();
    let mut algorithm_dropdown = Dropdown::new(
        px,
        120.0,
        ui::PANEL_WIDTH,
        "Algorithm",
        algorithm_items
    );
    algorithm_dropdown.set_selected(5); // SIMD+Par scales best for huge grids
    
    // Pattern dropdown - moved down
    let patterns = presets::all_patterns();
    let pattern_items: Vec<String> = patterns.iter()
        .map(|p| p.name.to_string())
        .collect();
    let mut pattern_dropdown = Dropdown::new(
        px,
        170.0,
        ui::PANEL_WIDTH,
        "Pattern",
        pattern_items
    );
    
    loop {
        let mouse_pos = mouse_position();
        
        // Update UI positions for responsiveness
        let px = ui::panel_x();
        grid_size_dropdown.set_position(px, 20.0);
        rule_dropdown.set_position(px, 70.0);
        algorithm_dropdown.set_position(px, 120.0);
        pattern_dropdown.set_position(px, 170.0);
        
        // Recreate buttons with current panel position
        let buttons = ui::create_buttons();
        
        // Update dropdowns (handle clicks) - only one can be open at a time
        if grid_size_dropdown.update(mouse_pos) {
            let size = GRID_SIZES[grid_size_dropdown.selected()].0;
            state.resize_grid(size, size);
            camera.reset();
        }
        // Close other dropdowns when grid_size opens
        if grid_size_dropdown.is_open() {
            rule_dropdown.close();
            algorithm_dropdown.close();
            pattern_dropdown.close();
        }
        
        if rule_dropdown.update(mouse_pos) {
            let rules = all_rules();
            let (_, rule) = rules.into_iter().nth(rule_dropdown.selected()).unwrap();
            state.set_rule(rule);
        }
        // Close other dropdowns when rule opens
        if rule_dropdown.is_open() {
            grid_size_dropdown.close();
            algorithm_dropdown.close();
            pattern_dropdown.close();
        }
        
        // Handle algorithm selection - NEW
        if algorithm_dropdown.update(mouse_pos) {
            let algorithms = Algorithm::all();
            let selected_algo = algorithms[algorithm_dropdown.selected()];
            state.set_algorithm(selected_algo);
        }
        // Close other dropdowns when algorithm opens
        if algorithm_dropdown.is_open() {
            grid_size_dropdown.close();
            rule_dropdown.close();
            pattern_dropdown.close();
        }
        
        // When pattern selected, enter placement mode
        if pattern_dropdown.update(mouse_pos) {
            state.pending_pattern_index = Some(pattern_dropdown.selected());
            state.is_running = false;
        }
        // Close other dropdowns when pattern opens
        if pattern_dropdown.is_open() {
            grid_size_dropdown.close();
            rule_dropdown.close();
            algorithm_dropdown.close();
        }
        
        // Handle pattern placement mode
        if let Some(pattern_idx) = state.pending_pattern_index {
            let pattern = &patterns[pattern_idx];
            
            // Right-click or Escape to cancel placement
            if is_mouse_button_pressed(MouseButton::Right) || is_key_pressed(KeyCode::Escape) {
                state.pending_pattern_index = None;
            }
            // Left-click on grid to place pattern
            else if is_mouse_button_pressed(MouseButton::Left) && mouse_pos.0 < ui::grid_area_width() {
                let (grid_x, grid_y) = camera.screen_to_grid(mouse_pos.0, mouse_pos.1, ui::CELL_SIZE);
                
                // Center pattern on click position
                let x = (grid_x as isize - pattern.width as isize / 2).max(0) as usize;
                let y = (grid_y as isize - pattern.height as isize / 2).max(0) as usize;
                
                pattern.place_on(&mut state.grid, x, y);
                state.pending_pattern_index = None;
            }
        }
        
        // Process input (skip paint if in placement mode)
        state = input::process_button_clicks(state, &buttons, mouse_pos);
        input::handle_zoom(&mut camera);
        input::handle_pan(&mut camera, mouse_pos);
        if state.pending_pattern_index.is_none() {
            input::handle_mouse_paint(&mut state, &camera, mouse_pos);
        }
        state = input::process_keyboard_input(state, &mut camera);
        
        // Update game state
        state = state.tick(get_frame_time());
        
        // Render (with timing)
        let render_start = std::time::Instant::now();
        clear_background(BLACK);
        rendering::draw_grid(&state.grid, &camera);
        
        // Draw pattern ghost preview if in placement mode
        if let Some(idx) = state.pending_pattern_index {
            if mouse_pos.0 < ui::grid_area_width() {
                rendering::draw_pattern_preview(&patterns[idx], &camera, mouse_pos);
            }
        }
        
        let dropdowns_slice: &[Dropdown] = &[
            grid_size_dropdown.clone(),
            rule_dropdown.clone(),
            algorithm_dropdown.clone(),
            pattern_dropdown.clone()
        ];
        rendering::draw_controls(&state, &camera, &buttons, dropdowns_slice, mouse_pos);
        state.last_render_time_ms = render_start.elapsed().as_secs_f32() * 1000.0;
        
        next_frame().await;
    }
}
