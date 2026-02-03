use macroquad::prelude::*;

/// Button UI component with hover and click detection
#[derive(Clone)]
pub struct Button {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    text: String,
    color: Color,
    hover_color: Color,
}

impl Button {
    pub fn new(x: f32, y: f32, width: f32, height: f32, text: impl Into<String>) -> Self {
        Self {
            x,
            y,
            width,
            height,
            text: text.into(),
            color: Color::from_rgba(70, 130, 180, 255),
            hover_color: Color::from_rgba(100, 149, 237, 255),
        }
    }
    
    /// Check if mouse is hovering over button
    pub fn is_hovered(&self, mouse_pos: (f32, f32)) -> bool {
        mouse_pos.0 >= self.x 
            && mouse_pos.0 <= self.x + self.width 
            && mouse_pos.1 >= self.y 
            && mouse_pos.1 <= self.y + self.height
    }
    
    /// Draw button with hover effect
    pub fn draw(&self, mouse_pos: (f32, f32)) {
        let color = if self.is_hovered(mouse_pos) {
            self.hover_color
        } else {
            self.color
        };
        
        draw_rectangle(self.x, self.y, self.width, self.height, color);
        draw_rectangle_lines(self.x, self.y, self.width, self.height, 2.0, WHITE);
        
        let text_size = measure_text(&self.text, None, 20, 1.0);
        draw_text(
            &self.text,
            self.x + (self.width - text_size.width) / 2.0,
            self.y + (self.height + text_size.height) / 2.0,
            20.0,
            WHITE,
        );
    }
    
    /// Check if button was clicked this frame
    pub fn is_clicked(&self, mouse_pos: (f32, f32)) -> bool {
        self.is_hovered(mouse_pos) && is_mouse_button_pressed(MouseButton::Left)
    }
}
