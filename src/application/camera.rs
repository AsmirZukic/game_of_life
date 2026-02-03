/// Camera manages viewport and zoom for grid navigation
pub struct Camera {
    pub offset_x: f32,
    pub offset_y: f32,
    pub zoom: f32,  // 1.0 = normal, 2.0 = 2x zoomed in
}

impl Camera {
    pub fn new() -> Self {
        Self {
            offset_x: 0.0,
            offset_y: 0.0,
            zoom: 1.0,
        }
    }
    
    /// Zoom in by factor
    pub fn zoom_in(&mut self, factor: f32) {
        self.zoom = (self.zoom * factor).clamp(0.5, 10.0);
    }
    
    /// Zoom out by factor
    pub fn zoom_out(&mut self, factor: f32) {
        self.zoom = (self.zoom / factor).clamp(0.5, 10.0);
    }
    
    /// Pan camera
    pub fn pan(&mut self, dx: f32, dy: f32) {
        self.offset_x += dx;
        self.offset_y += dy;
    }
    
    /// Convert screen coordinates to grid coordinates
    pub fn screen_to_grid(&self, screen_x: f32, screen_y: f32, cell_size: f32) -> (i32, i32) {
        let grid_x = ((screen_x - self.offset_x) / (cell_size * self.zoom)) as i32;
        let grid_y = ((screen_y - self.offset_y) / (cell_size * self.zoom)) as i32;
        (grid_x, grid_y)
    }
    
    /// Convert grid coordinates to screen coordinates
    pub fn grid_to_screen(&self, grid_x: usize, grid_y: usize, cell_size: f32) -> (f32, f32) {
        let screen_x = grid_x as f32 * cell_size * self.zoom + self.offset_x;
        let screen_y = grid_y as f32 * cell_size * self.zoom + self.offset_y;
        (screen_x, screen_y)
    }
    
    /// Get visible grid bounds for culling
    pub fn visible_bounds(&self, viewport_width: f32, viewport_height: f32, cell_size: f32) -> (i32, i32, i32, i32) {
        let (min_x, min_y) = self.screen_to_grid(0.0, 0.0, cell_size);
        let (max_x, max_y) = self.screen_to_grid(viewport_width, viewport_height, cell_size);
        (min_x, min_y, max_x, max_y)
    }
    
    /// Reset camera to default
    pub fn reset(&mut self) {
        self.offset_x = 0.0;
        self.offset_y = 0.0;
        self.zoom = 1.0;
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self::new()
    }
}
