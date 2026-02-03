use macroquad::prelude::*;

/// Dropdown selector UI component
#[derive(Clone)]
pub struct Dropdown {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    items: Vec<String>,
    selected: usize,
    is_open: bool,
    label: String,
}

impl Dropdown {
    pub fn new(x: f32, y: f32, width: f32, label: impl Into<String>, items: Vec<String>) -> Self {
        Self {
            x,
            y,
            width,
            height: 30.0,
            items,
            selected: 0,
            is_open: false,
            label: label.into(),
        }
    }
    
    /// Get currently selected index
    pub fn selected(&self) -> usize {
        self.selected
    }
    
    /// Set selected index
    pub fn set_selected(&mut self, index: usize) {
        if index < self.items.len() {
            self.selected = index;
        }
    }
    
    /// Check if dropdown is open
    pub fn is_open(&self) -> bool {
        self.is_open
    }
    
    /// Close the dropdown
    pub fn close(&mut self) {
        self.is_open = false;
    }
    
    /// Update position for responsive layout
    pub fn set_position(&mut self, x: f32, y: f32) {
        self.x = x;
        self.y = y;
    }
    
    /// Draw dropdown without handling interaction (for rendering only)
    pub fn draw(&self, mouse_pos: (f32, f32)) {
        // Draw label
        draw_text(&self.label, self.x, self.y - 5.0, 14.0, GRAY);
        
        // Main button
        let button_color = if self.is_hovered_main(mouse_pos) {
            Color::from_rgba(100, 149, 237, 255)
        } else {
            Color::from_rgba(70, 130, 180, 255)
        };
        
        draw_rectangle(self.x, self.y, self.width, self.height, button_color);
        draw_rectangle_lines(self.x, self.y, self.width, self.height, 2.0, WHITE);
        
        // Selected item text - truncate if too long
        let text = &self.items[self.selected];
        let font_size = 16.0;
        let max_width = self.width - 30.0; // Leave space for arrow
        
        // Measure and potentially truncate text
        let text_measure = measure_text(text, None, font_size as u16, 1.0);
        let display_text = if text_measure.width > max_width {
            // Truncate and add ellipsis
            let mut truncated = text.clone();
            while measure_text(&format!("{}...", truncated), None, font_size as u16, 1.0).width > max_width && truncated.len() > 0 {
                truncated.pop();
            }
            format!("{}...", truncated)
        } else {
            text.clone()
        };
        
        draw_text(&display_text, self.x + 5.0, self.y + 21.0, font_size, WHITE);
        
        // Dropdown arrow
        draw_text("▼", self.x + self.width - 18.0, self.y + 21.0, 14.0, WHITE);
        
        // Draw dropdown menu if open
        if self.is_open {
            // Draw opaque background for entire dropdown menu
            let menu_height = self.items.len() as f32 * self.height;
            draw_rectangle(
                self.x,
                self.y + self.height,
                self.width,
                menu_height,
                Color::from_rgba(30, 30, 30, 255) // Fully opaque dark background
            );
            
            for (i, item) in self.items.iter().enumerate() {
                let item_y = self.y + self.height + (i as f32 * self.height);
                
                let item_color = if self.is_hovered_item(mouse_pos, i) {
                    Color::from_rgba(100, 149, 237, 255) // Blue when hovered
                } else if i == self.selected {
                    Color::from_rgba(50, 100, 150, 255) // Darker blue when selected
                } else {
                    Color::from_rgba(45, 45, 45, 255) // Opaque gray for unselected
                };
                
                draw_rectangle(self.x, item_y, self.width, self.height, item_color);
                draw_rectangle_lines(self.x, item_y, self.width, self.height, 1.0, Color::from_rgba(80, 80, 80, 255));
                
                // Truncate item text too
                let item_measure = measure_text(item, None, font_size as u16, 1.0);
                let item_display = if item_measure.width > self.width - 10.0 {
                    let mut truncated = item.clone();
                    while measure_text(&format!("{}...", truncated), None, font_size as u16, 1.0).width > self.width - 10.0 && truncated.len() > 0 {
                        truncated.pop();
                    }
                    format!("{}...", truncated)
                } else {
                    item.clone()
                };
                
                draw_text(&item_display, self.x + 5.0, item_y + 21.0, font_size, WHITE);
            }
            
            // Draw border around entire menu
            draw_rectangle_lines(
                self.x,
                self.y + self.height,
                self.width,
                menu_height,
                2.0,
                WHITE
            );
        }
    }
    
    /// Handle interaction and return true if selection changed
    pub fn update(&mut self, mouse_pos: (f32, f32)) -> bool {
        let mut changed = false;
        
        // Handle click on main button
        if self.is_hovered_main(mouse_pos) && is_mouse_button_pressed(MouseButton::Left) {
            self.is_open = !self.is_open;
            return false; // Opening/closing dropdown is not a selection change
        }
        
        // Handle dropdown menu if open
        if self.is_open {
            for i in 0..self.items.len() {
                // Handle item click
                if self.is_hovered_item(mouse_pos, i) && is_mouse_button_pressed(MouseButton::Left) {
                    if self.selected != i {
                        self.selected = i;
                        changed = true;
                    }
                    self.is_open = false;
                    return changed;
                }
            }
            
            // Close if clicked outside
            if is_mouse_button_pressed(MouseButton::Left) && !self.is_hovered_any(mouse_pos) {
                self.is_open = false;
            }
        }
        
        changed
    }
    
    /// Draw dropdown and handle interaction (convenience method)
    /// Returns true if selection changed
    pub fn draw_and_update(&mut self, mouse_pos: (f32, f32)) -> bool {
        let changed = self.update(mouse_pos);
        self.draw(mouse_pos);
        changed
    }
    
    fn is_hovered_main(&self, mouse_pos: (f32, f32)) -> bool {
        mouse_pos.0 >= self.x 
            && mouse_pos.0 <= self.x + self.width 
            && mouse_pos.1 >= self.y 
            && mouse_pos.1 <= self.y + self.height
    }
    
    fn is_hovered_item(&self, mouse_pos: (f32, f32), index: usize) -> bool {
        let item_y = self.y + self.height + (index as f32 * self.height);
        mouse_pos.0 >= self.x 
            && mouse_pos.0 <= self.x + self.width 
            && mouse_pos.1 >= item_y 
            && mouse_pos.1 <= item_y + self.height
    }
    
    fn is_hovered_any(&self, mouse_pos: (f32, f32)) -> bool {
        if self.is_hovered_main(mouse_pos) {
            return true;
        }
        for i in 0..self.items.len() {
            if self.is_hovered_item(mouse_pos, i) {
                return true;
            }
        }
        false
    }
}
