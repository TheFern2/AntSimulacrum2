use macroquad::prelude::*;

mod ant;
mod camera;
mod pheromone;
mod rendering;
mod world;

use ant::Ant;
use camera::Camera;
use pheromone::{PheromoneGrid, DEFAULT_DECAY_RATE};
use rendering::{draw_debug_overlay, draw_scene};
use world::World;

const GRID_W: usize = 80;
const GRID_H: usize = 60;
const CELL_SIZE: f32 = 12.0;
const ANT_COUNT: usize = 60;

fn window_conf() -> Conf {
    Conf {
        window_title: "AntSimulacrum".to_owned(),
        window_width: 1280,
        window_height: 720,
        ..Default::default()
    }
}

fn make_world() -> World { World::new(GRID_W, GRID_H, CELL_SIZE) }
fn make_ants(nest: Vec2) -> Vec<Ant> { (0..ANT_COUNT).map(|_| Ant::new(nest)).collect() }

#[macroquad::main(window_conf)]
async fn main() {
    let mut world = make_world();
    let mut camera = Camera::new(world.nest_pos);
    let mut pheromones = PheromoneGrid::new(GRID_W, GRID_H);
    let mut ants = make_ants(world.nest_pos);
    let mut decay_rate = DEFAULT_DECAY_RATE;

    loop {
        let dt = get_frame_time().min(0.05);
        let shift = is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift);

        // --- Debug controls ---
        if shift && is_key_pressed(KeyCode::R) {
            world = make_world();
            pheromones = PheromoneGrid::new(GRID_W, GRID_H);
            ants = make_ants(world.nest_pos);
            camera = Camera::new(world.nest_pos);
        }
        if shift && is_key_pressed(KeyCode::Up) {
            decay_rate = (decay_rate * 1.5).min(5.0);
        }
        if shift && is_key_pressed(KeyCode::Down) {
            decay_rate = (decay_rate / 1.5).max(0.01);
        }

        camera.handle_input(world.nest_pos);

        // Simulate
        pheromones.decay(dt, decay_rate);
        for ant in ants.iter_mut() {
            ant.update(dt, &mut world, &mut pheromones);
        }

        // Render
        draw_scene(&world, &camera, &pheromones, &ants);
        draw_debug_overlay(get_fps() as f32, &world, ants.len(), decay_rate);

        next_frame().await;
    }
}
