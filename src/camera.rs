use macroquad::prelude::*;

pub struct Camera {
    pub offset: Vec2,
    pub zoom: f32,
    drag_start: Option<(Vec2, Vec2)>, // (mouse screen pos, offset at drag start)
}

const ZOOM_MIN: f32 = 0.2;
const ZOOM_MAX: f32 = 8.0;
const ZOOM_SPEED: f32 = 0.1;

impl Camera {
    pub fn new(world_center: Vec2) -> Self {
        let screen_center = Vec2::new(screen_width() / 2.0, screen_height() / 2.0);
        Self {
            offset: screen_center - world_center,
            zoom: 1.0,
            drag_start: None,
        }
    }

    pub fn handle_input(&mut self, world_center: Vec2, ui_captured: bool) {
        if ui_captured { return; }
        // Scroll to zoom (centered on mouse)
        let scroll = mouse_wheel().1;
        if scroll != 0.0 {
            let mouse = Vec2::from(mouse_position());
            let world_before = self.screen_to_world(mouse);
            let factor = if scroll > 0.0 { 1.0 + ZOOM_SPEED } else { 1.0 - ZOOM_SPEED };
            self.zoom = (self.zoom * factor).clamp(ZOOM_MIN, ZOOM_MAX);
            let world_after = self.screen_to_world(mouse);
            self.offset += (world_after - world_before) * self.zoom;
        }

        // Right-click drag to pan
        if is_mouse_button_pressed(MouseButton::Right) {
            self.drag_start = Some((Vec2::from(mouse_position()), self.offset));
        }
        if is_mouse_button_released(MouseButton::Right) {
            self.drag_start = None;
        }
        if let Some((start_mouse, start_offset)) = self.drag_start {
            if is_mouse_button_down(MouseButton::Right) {
                let delta = Vec2::from(mouse_position()) - start_mouse;
                self.offset = start_offset + delta;
            }
        }

        // Home key centers nest
        if is_key_pressed(KeyCode::Home) {
            let screen_center = Vec2::new(screen_width() / 2.0, screen_height() / 2.0);
            self.offset = screen_center - world_center * self.zoom;
        }
    }

    /// World position → screen position
    pub fn world_to_screen(&self, world: Vec2) -> Vec2 {
        world * self.zoom + self.offset
    }

    /// Screen position → world position
    pub fn screen_to_world(&self, screen: Vec2) -> Vec2 {
        (screen - self.offset) / self.zoom
    }

    pub fn zoom(&self) -> f32 {
        self.zoom
    }
}
