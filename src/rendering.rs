use macroquad::prelude::*;

use crate::ant::{Ant, AntState};
use crate::camera::Camera;
use crate::pheromone::{PheromoneGrid, MAX_INTENSITY};
use crate::world::{Cell, World};

const COLOR_BG: Color = Color { r: 0.102, g: 0.071, b: 0.031, a: 1.0 }; // #1a1208
const COLOR_WALL: Color = Color { r: 0.35, g: 0.35, b: 0.35, a: 1.0 };
const COLOR_FOOD: Color = Color { r: 0.2, g: 0.8, b: 0.2, a: 1.0 };
const COLOR_NEST_OUTER: Color = Color { r: 0.8, g: 0.6, b: 0.1, a: 0.4 };
const COLOR_NEST_INNER: Color = Color { r: 1.0, g: 0.8, b: 0.2, a: 0.9 };
const COLOR_ANT: Color = Color { r: 1.0, g: 0.7, b: 0.1, a: 1.0 };
const COLOR_ANT_CARRYING: Color = Color { r: 0.4, g: 1.0, b: 0.4, a: 1.0 };

pub fn draw_scene(world: &World, camera: &Camera, pheromones: &PheromoneGrid, ants: &[Ant]) {
    clear_background(COLOR_BG);

    draw_pheromones(world, camera, pheromones);
    draw_world_cells(world, camera);
    draw_ants(camera, ants);
}

fn draw_pheromones(world: &World, camera: &Camera, pheromones: &PheromoneGrid) {
    let cs = world.cell_size * camera.zoom();

    for y in 0..world.height {
        for x in 0..world.width {
            let idx = y * world.width + x;
            let home = pheromones.to_home[idx];
            let food = pheromones.to_food[idx];

            let sp = camera.world_to_screen(Vec2::new(
                x as f32 * world.cell_size,
                y as f32 * world.cell_size,
            ));

            // to_food: amber #ff9900
            if food > 0.01 {
                let alpha = (food / MAX_INTENSITY).min(1.0) * 0.75;
                draw_rectangle(sp.x, sp.y, cs, cs, Color { r: 1.0, g: 0.6, b: 0.0, a: alpha });
            }

            // to_home: blue #0099ff
            if home > 0.01 {
                let alpha = (home / MAX_INTENSITY).min(1.0) * 0.75;
                draw_rectangle(sp.x, sp.y, cs, cs, Color { r: 0.0, g: 0.6, b: 1.0, a: alpha });
            }
        }
    }
}

fn draw_world_cells(world: &World, camera: &Camera) {
    let cs = world.cell_size * camera.zoom();

    for y in 0..world.height {
        for x in 0..world.width {
            match world.get(x, y) {
                Cell::Empty => {}
                Cell::Wall => {
                    let sp = camera.world_to_screen(Vec2::new(
                        x as f32 * world.cell_size,
                        y as f32 * world.cell_size,
                    ));
                    draw_rectangle(sp.x, sp.y, cs, cs, COLOR_WALL);
                }
                Cell::Food => {
                    let center = camera.world_to_screen(Vec2::new(
                        (x as f32 + 0.5) * world.cell_size,
                        (y as f32 + 0.5) * world.cell_size,
                    ));
                    let q = world.food_quantities[y * world.width + x];
                    let radius = (cs * 0.15 + cs * 0.25 * (q / 30.0)).min(cs * 0.45);
                    draw_circle(center.x, center.y, radius, COLOR_FOOD);
                }
            }
        }
    }

    // Nest — concentric rings
    let nest_s = camera.world_to_screen(world.nest_pos);
    let base_r = 20.0 * camera.zoom();
    draw_circle(nest_s.x, nest_s.y, base_r * 2.2, COLOR_NEST_OUTER);
    draw_circle(nest_s.x, nest_s.y, base_r * 1.4, Color { a: 0.6, ..COLOR_NEST_OUTER });
    draw_circle(nest_s.x, nest_s.y, base_r, COLOR_NEST_INNER);
}

fn draw_ants(camera: &Camera, ants: &[Ant]) {
    let ant_r = 2.5 * camera.zoom();
    let tick_len = 5.0 * camera.zoom();

    for ant in ants {
        let sp = camera.world_to_screen(ant.position);
        let color = if ant.carrying_food { COLOR_ANT_CARRYING } else { COLOR_ANT };
        draw_circle(sp.x, sp.y, ant_r, color);

        // Direction tick
        let tip = sp + Vec2::new(ant.direction.cos(), ant.direction.sin()) * tick_len;
        draw_line(sp.x, sp.y, tip.x, tip.y, 1.0, color);

        // Draw returning ants with a slightly brighter head when carrying
        if ant.state == AntState::Returning {
            draw_circle_lines(sp.x, sp.y, ant_r + 1.0, 0.5, COLOR_ANT_CARRYING);
        }
    }
}

pub fn draw_debug_overlay(fps: f32, world: &World, ant_count: usize, decay_rate: f32) {
    draw_text(&format!("FPS: {:.0}", fps), 8.0, 20.0, 20.0, WHITE);
    draw_text(&format!("Ants: {}", ant_count), 8.0, 42.0, 20.0, WHITE);
    draw_text(&format!("Food stored: {:.0}", world.food_stored), 8.0, 64.0, 20.0, WHITE);
    draw_text(&format!("Decay: {:.3}/s  [Shift+↑/↓]", decay_rate), 8.0, 86.0, 18.0, GRAY);
    draw_text("Shift+R: reset", 8.0, 106.0, 18.0, GRAY);
}
