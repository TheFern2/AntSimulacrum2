use macroquad::prelude::*;

mod ant;
mod camera;
mod pheromone;
mod rendering;
mod world;

use ant::Ant;
use camera::Camera;
use pheromone::PheromoneGrid;
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

#[macroquad::main(window_conf)]
async fn main() {
    let mut world = World::new(GRID_W, GRID_H, CELL_SIZE);
    let mut camera = Camera::new(world.nest_pos);
    let mut pheromones = PheromoneGrid::new(GRID_W, GRID_H);

    // Spawn ants clustered at nest
    let mut ants: Vec<Ant> = (0..ANT_COUNT)
        .map(|_| Ant::new(world.nest_pos))
        .collect();

    loop {
        let dt = get_frame_time().min(0.05); // cap dt to avoid spiral on focus loss

        camera.handle_input(world.nest_pos);

        // Simulate
        pheromones.decay(dt);
        for ant in ants.iter_mut() {
            ant.update(dt, &mut world, &mut pheromones);
        }

        // Render
        draw_scene(&world, &camera, &pheromones, &ants);
        draw_debug_overlay(get_fps() as f32, &world, ants.len());

        next_frame().await;
    }
}
