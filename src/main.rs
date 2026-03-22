use macroquad::prelude::*;

mod ant;
mod brood;
mod camera;
mod colony;
mod ecology;
mod pheromone;
mod rendering;
mod world;

use ant::{Ant, Caste};
use brood::{advance_brood, BroodMember};
use camera::Camera;
use colony::{Colony, GameMode};
use ecology::Ecology;
use pheromone::{PheromoneGrid, DEFAULT_DECAY_RATE};
use rendering::{draw_debug_overlay, draw_scene};
use world::World;

const GRID_W: usize = 80;
const GRID_H: usize = 60;
const CELL_SIZE: f32 = 12.0;
const INITIAL_ANT_COUNT: usize = 50;  // total starting workers
const INITIAL_BATCH_SIZE: usize = 10; // spawned at once
const INITIAL_BATCH_INTERVAL: f32 = 30.0; // seconds between batches

fn window_conf() -> Conf {
    Conf {
        window_title: "AntSimulacrum".to_owned(),
        window_width: 1280,
        window_height: 720,
        ..Default::default()
    }
}

fn make_world() -> World {
    World::new(GRID_W, GRID_H, CELL_SIZE)
}

fn make_ant_batch(nest: Vec2, count: usize) -> Vec<Ant> {
    use ::rand::Rng;
    let mut rng = ::rand::thread_rng();
    (0..count)
        .map(|_| {
            let mut ant = Ant::new_with_caste(nest, Caste::Worker);
            // Varied lifespans so deaths trickle in over a wide window (240–600s)
            // rather than clustering at one point.
            ant.max_age = rng.gen_range(ant.max_age * 0.67..ant.max_age * 1.67);
            ant
        })
        .collect()
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut world = make_world();
    let mut ecology = Ecology::new(&mut world);
    let mut camera = Camera::new(world.nest_pos);
    let mut pheromones = PheromoneGrid::new(GRID_W, GRID_H);
    let mut ants = make_ant_batch(world.nest_pos, INITIAL_BATCH_SIZE);
    let mut brood: Vec<BroodMember> = Vec::new();
    let mut colony = Colony::new(GameMode::Zen);
    let mut decay_rate = DEFAULT_DECAY_RATE;
    let mut initial_spawned = INITIAL_BATCH_SIZE;
    let mut batch_timer = INITIAL_BATCH_INTERVAL;

    loop {
        let dt = get_frame_time().min(0.05);
        let shift = is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift);

        // ── Debug controls ──────────────────────────────────────────────────
        if shift && is_key_pressed(KeyCode::R) {
            world = make_world();
            ecology = Ecology::new(&mut world);
            pheromones = PheromoneGrid::new(GRID_W, GRID_H);
            ants = make_ant_batch(world.nest_pos, INITIAL_BATCH_SIZE);
            brood.clear();
            colony = Colony::new(colony.mode);
            camera = Camera::new(world.nest_pos);
            initial_spawned = INITIAL_BATCH_SIZE;
            batch_timer = INITIAL_BATCH_INTERVAL;
        }
        if shift && is_key_pressed(KeyCode::M) {
            let new_mode = match colony.mode {
                GameMode::Zen    => GameMode::Normal,
                GameMode::Normal => GameMode::Zen,
            };
            world = make_world();
            ecology = Ecology::new(&mut world);
            colony = Colony::new(new_mode);
            ants = make_ant_batch(world.nest_pos, INITIAL_BATCH_SIZE);
            brood.clear();
            pheromones = PheromoneGrid::new(GRID_W, GRID_H);
            initial_spawned = INITIAL_BATCH_SIZE;
            batch_timer = INITIAL_BATCH_INTERVAL;
        }
        if shift && is_key_pressed(KeyCode::Up) {
            decay_rate = (decay_rate * 1.5).min(5.0);
        }
        if shift && is_key_pressed(KeyCode::Down) {
            decay_rate = (decay_rate / 1.5).max(0.01);
        }

        camera.handle_input(world.nest_pos);

        // ── Staggered initial spawning ──────────────────────────────────────
        if initial_spawned < INITIAL_ANT_COUNT {
            batch_timer -= dt;
            if batch_timer <= 0.0 {
                batch_timer = INITIAL_BATCH_INTERVAL;
                let remaining = INITIAL_ANT_COUNT - initial_spawned;
                let batch = remaining.min(INITIAL_BATCH_SIZE);
                ants.extend(make_ant_batch(world.nest_pos, batch));
                initial_spawned += batch;
            }
        }

        // ── Simulate ────────────────────────────────────────────────────────

        // Ecology tick: regrowth, spawning, day/night clock
        ecology.update(dt, &mut world);

        // Colony tick: consume food, queen health, lay eggs
        let new_eggs = colony.update(dt, ants.len(), brood.len());
        for _ in 0..new_eggs {
            brood.push(BroodMember::new_egg());
        }

        // Advance brood; mature larvae hatch into new ants
        let hatched = advance_brood(&mut brood, dt, world.nest_pos);
        ants.extend(hatched);

        // Pheromone decay
        pheromones.decay(dt, decay_rate);

        // Update ants; remove any that died
        let old_ants = std::mem::take(&mut ants);
        for mut ant in old_ants {
            if ant.update(dt, &mut world, &mut pheromones, &mut colony) {
                ants.push(ant);
            }
        }

        // Zen mode: enforce minimum worker floor
        if colony.mode == GameMode::Zen {
            let workers = ants.iter().filter(|a| a.caste == Caste::Worker).count();
            let floor = Colony::zen_min_workers();
            for _ in workers..floor {
                ants.push(Ant::new_with_caste(world.nest_pos, Caste::Worker));
            }
        }

        // Colony collapse check (Normal mode only)
        if !colony.collapsed && colony.check_collapse(ants.len(), brood.len()) {
            colony.collapsed = true;
        }

        // ── Render ──────────────────────────────────────────────────────────
        draw_scene(&world, &camera, &pheromones, &ants, &ecology);
        draw_debug_overlay(get_fps() as f32, &colony, &ants, &brood, decay_rate, &ecology);

        next_frame().await;
    }
}
