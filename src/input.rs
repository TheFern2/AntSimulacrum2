// Phase 5 — input handling: tool system, hotkeys, world interaction.

use macroquad::prelude::*;

use crate::ant::{Ant, Caste};
use crate::camera::Camera;
use crate::ecology::Ecology;
use crate::pheromone::PheromoneVis;
use crate::world::{Cell, World};

pub const TOP_BAR_H: f32    = 40.0;
pub const BOTTOM_BAR_H: f32 = 40.0;

const DROP_ANTS_COUNT: usize = 5;

// ── Tool ────────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Tool {
    Observe,
    PlaceFood,
    DrawWall,
    DropAnts,
    Eraser,
}

// ── Simulation speed ────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum SimSpeed {
    Paused,
    Normal,
    Fast,
    Max,
}

// ── Input state ─────────────────────────────────────────────────────────────

pub struct InputState {
    pub active_tool: Tool,
    pub sim_speed:   SimSpeed,
    pub phero_vis:   PheromoneVis,
    pub show_ant_labels: bool,
    pub settings_open:   bool,
    pub stats_open:      bool,
    pub show_debug:      bool,
}

impl InputState {
    pub fn new() -> Self {
        Self {
            active_tool:      Tool::Observe,
            sim_speed:        SimSpeed::Normal,
            phero_vis:        PheromoneVis::Both,
            show_ant_labels:  false,
            settings_open:    false,
            stats_open:       false,
            show_debug:       false,
        }
    }

    pub fn speed_multiplier(&self) -> f32 {
        match self.sim_speed {
            SimSpeed::Paused => 0.0,
            SimSpeed::Normal => 1.0,
            SimSpeed::Fast   => 2.0,
            SimSpeed::Max    => 8.0,
        }
    }

    /// Handle keyboard hotkeys (call once per frame).
    pub fn handle_hotkeys(&mut self) {
        // Tool selection 1–5
        if is_key_pressed(KeyCode::Key1) { self.active_tool = Tool::Observe;   }
        if is_key_pressed(KeyCode::Key2) { self.active_tool = Tool::PlaceFood; }
        if is_key_pressed(KeyCode::Key3) { self.active_tool = Tool::DrawWall;  }
        if is_key_pressed(KeyCode::Key4) { self.active_tool = Tool::DropAnts;  }
        if is_key_pressed(KeyCode::Key5) { self.active_tool = Tool::Eraser;    }

        // Space: pause / resume
        if is_key_pressed(KeyCode::Space) {
            self.sim_speed = match self.sim_speed {
                SimSpeed::Paused => SimSpeed::Normal,
                _                => SimSpeed::Paused,
            };
        }

        // F1: toggle debug overlay
        if is_key_pressed(KeyCode::F1) {
            self.show_debug = !self.show_debug;
        }
    }

    /// True if the screen y-coordinate falls within a HUD bar.
    pub fn is_over_ui(screen_y: f32) -> bool {
        screen_y < TOP_BAR_H || screen_y > screen_height() - BOTTOM_BAR_H
    }

    /// Apply the active tool on a left-click at `screen_pos`.
    /// Returns true if any world action was taken.
    pub fn apply_tool_click(
        &mut self,
        screen_pos: Vec2,
        camera: &Camera,
        world: &mut World,
        ants: &mut Vec<Ant>,
        ecology: &mut Ecology,
    ) -> bool {
        if Self::is_over_ui(screen_pos.y) { return false; }

        let world_pos = camera.screen_to_world(screen_pos);
        let gx = (world_pos.x / world.cell_size) as i32;
        let gy = (world_pos.y / world.cell_size) as i32;
        let in_bounds = gx > 0 && gy > 0
            && gx < world.width  as i32 - 1
            && gy < world.height as i32 - 1;

        match self.active_tool {
            Tool::Observe => {
                // Toggle stats panel when clicking near the nest
                let nest_screen = camera.world_to_screen(world.nest_pos);
                if screen_pos.distance(nest_screen) < 44.0 * camera.zoom() {
                    self.stats_open = !self.stats_open;
                    return true;
                }
                false
            }

            Tool::PlaceFood => {
                if in_bounds {
                    ecology.add_source_at_grid(gx as usize, gy as usize, world);
                    return true;
                }
                false
            }

            Tool::DrawWall => {
                if in_bounds {
                    let idx = gy as usize * world.width + gx as usize;
                    world.cells[idx] = Cell::Wall;
                    world.food_quantities[idx] = 0.0;
                    return true;
                }
                false
            }

            Tool::DropAnts => {
                use ::rand::Rng;
                let mut rng = ::rand::thread_rng();
                for _ in 0..DROP_ANTS_COUNT {
                    let offset = Vec2::new(
                        rng.gen_range(-6.0..6.0),
                        rng.gen_range(-6.0..6.0),
                    );
                    ants.push(Ant::new_with_caste(world_pos + offset, Caste::Worker));
                }
                true
            }

            Tool::Eraser => {
                if in_bounds {
                    let idx = gy as usize * world.width + gx as usize;
                    world.cells[idx] = Cell::Empty;
                    world.food_quantities[idx] = 0.0;
                    return true;
                }
                false
            }
        }
    }

    /// Hold-to-paint for Wall and Eraser tools (call while mouse button is held).
    pub fn apply_tool_hold(
        &self,
        screen_pos: Vec2,
        camera: &Camera,
        world: &mut World,
    ) {
        if Self::is_over_ui(screen_pos.y) { return; }
        if !matches!(self.active_tool, Tool::DrawWall | Tool::Eraser) { return; }

        let world_pos = camera.screen_to_world(screen_pos);
        let gx = (world_pos.x / world.cell_size) as i32;
        let gy = (world_pos.y / world.cell_size) as i32;

        if gx > 0 && gy > 0 && gx < world.width as i32 - 1 && gy < world.height as i32 - 1 {
            let idx = gy as usize * world.width + gx as usize;
            match self.active_tool {
                Tool::DrawWall => {
                    world.cells[idx] = Cell::Wall;
                    world.food_quantities[idx] = 0.0;
                }
                Tool::Eraser => {
                    world.cells[idx] = Cell::Empty;
                    world.food_quantities[idx] = 0.0;
                }
                _ => {}
            }
        }
    }
}
