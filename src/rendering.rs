use macroquad::prelude::*;

use crate::ant::{Ant, AntState, Caste};
use crate::brood::{BroodMember, BroodStage};
use crate::camera::Camera;
use crate::colony::Colony;
use crate::ecology::Ecology;
use crate::pheromone::{PheromoneGrid, MAX_INTENSITY};
use crate::world::{Cell, World};

const COLOR_BG_DAY: Color   = Color { r: 0.102, g: 0.071, b: 0.031, a: 1.0 }; // #1a1208
const COLOR_BG_NIGHT: Color = Color { r: 0.039, g: 0.039, b: 0.039, a: 1.0 }; // #0a0a0a
const COLOR_WALL: Color = Color { r: 0.35, g: 0.35, b: 0.35, a: 1.0 };
const COLOR_FOOD: Color = Color { r: 0.2, g: 0.8, b: 0.2, a: 1.0 };
const COLOR_NEST_OUTER: Color = Color { r: 0.8, g: 0.6, b: 0.1, a: 0.4 };
const COLOR_NEST_INNER: Color = Color { r: 1.0, g: 0.8, b: 0.2, a: 0.9 };

// Caste colors
const COLOR_WORKER: Color = Color { r: 1.0, g: 0.7, b: 0.1, a: 1.0 }; // amber
const COLOR_SCOUT: Color  = Color { r: 0.9, g: 0.9, b: 1.0, a: 1.0 }; // pale white-blue
const COLOR_SOLDIER: Color = Color { r: 1.0, g: 0.2, b: 0.2, a: 1.0 }; // red
const COLOR_NURSE: Color   = Color { r: 0.9, g: 0.5, b: 0.9, a: 1.0 }; // lavender
const COLOR_ANT_CARRYING: Color = Color { r: 0.4, g: 1.0, b: 0.4, a: 1.0 }; // green

pub fn draw_scene(
    world: &World,
    camera: &Camera,
    pheromones: &PheromoneGrid,
    ants: &[Ant],
    ecology: &Ecology,
) {
    // Background lerps from day soil (#1a1208) to night dark (#0a0a0a)
    let n = ecology.night_amount();
    let bg = Color {
        r: lerp(COLOR_BG_DAY.r, COLOR_BG_NIGHT.r, n),
        g: lerp(COLOR_BG_DAY.g, COLOR_BG_NIGHT.g, n),
        b: lerp(COLOR_BG_DAY.b, COLOR_BG_NIGHT.b, n),
        a: 1.0,
    };
    clear_background(bg);
    draw_pheromones(world, camera, pheromones);
    draw_world_cells(world, camera);
    draw_ants(camera, ants);
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
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

            if food > 0.01 {
                let alpha = (food / MAX_INTENSITY).min(1.0) * 0.75;
                draw_rectangle(sp.x, sp.y, cs, cs, Color { r: 1.0, g: 0.6, b: 0.0, a: alpha });
            }
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

    // Nest concentric rings
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

        let color = if ant.carrying_food {
            COLOR_ANT_CARRYING
        } else {
            match ant.caste {
                Caste::Worker  => COLOR_WORKER,
                Caste::Scout   => COLOR_SCOUT,
                Caste::Soldier => COLOR_SOLDIER,
                Caste::Nurse   => COLOR_NURSE,
            }
        };

        draw_circle(sp.x, sp.y, ant_r, color);

        // Direction tick
        let tip = sp + Vec2::new(ant.direction.cos(), ant.direction.sin()) * tick_len;
        draw_line(sp.x, sp.y, tip.x, tip.y, 1.0, color);

        // Ring highlight for returning ants
        if ant.state == AntState::Returning {
            draw_circle_lines(sp.x, sp.y, ant_r + 1.0, 0.5, COLOR_ANT_CARRYING);
        }
    }
}

pub fn draw_debug_overlay(
    fps: f32,
    colony: &Colony,
    ants: &[Ant],
    brood: &[BroodMember],
    decay_rate: f32,
    ecology: &Ecology,
) {
    let workers  = ants.iter().filter(|a| a.caste == Caste::Worker).count();
    let scouts   = ants.iter().filter(|a| a.caste == Caste::Scout).count();
    let soldiers = ants.iter().filter(|a| a.caste == Caste::Soldier).count();
    let nurses   = ants.iter().filter(|a| a.caste == Caste::Nurse).count();
    let eggs     = brood.iter().filter(|b| b.stage == BroodStage::Egg).count();
    let larvae   = brood.iter().filter(|b| b.stage == BroodStage::Larva).count();

    let mode_label = match colony.mode {
        crate::colony::GameMode::Zen    => "ZEN",
        crate::colony::GameMode::Normal => "NORMAL",
    };

    let mut y = 20.0f32;
    let line = 22.0f32;

    draw_text(&format!("FPS: {:.0}", fps), 8.0, y, 20.0, WHITE); y += line;
    draw_text(&format!("[{}]  Age: {:.0}s", mode_label, colony.colony_age), 8.0, y, 18.0, GRAY); y += line;
    draw_text(&format!("Ants: {}  (W:{} Sc:{} So:{} N:{})", ants.len(), workers, scouts, soldiers, nurses), 8.0, y, 18.0, WHITE); y += line;
    draw_text(&format!("Brood: {}  (eggs:{} larvae:{})", brood.len(), eggs, larvae), 8.0, y, 18.0, WHITE); y += line;
    draw_text(&format!("Food stored: {:.0}  delivered: {}", colony.food_stored as i32, colony.total_food_delivered), 8.0, y, 18.0, WHITE); y += line;
    draw_text(&format!("Queen: {}  health: {:.0}%", colony.queen.status_label(), colony.queen.health * 100.0), 8.0, y, 18.0, WHITE); y += line;
    y += 4.0;
    draw_text(&format!("Decay: {:.3}/s  [Shift+↑/↓]", decay_rate), 8.0, y, 16.0, GRAY); y += 20.0;
    let day_icon = if ecology.is_day() { "☀" } else { "☾" };
    let time_in_day = ecology.day_time;
    draw_text(
        &format!("Day {} {}  ({:.0}s / {:.0}s)  Sources: {}",
            ecology.day_count + 1, day_icon, time_in_day, ecology.day_length,
            ecology.sources.len()),
        8.0, y, 16.0, GRAY,
    ); y += 20.0;
    draw_text("Shift+R: reset  |  Shift+M: toggle mode", 8.0, y, 16.0, GRAY);

    if colony.collapsed {
        let cw = screen_width();
        let ch = screen_height();
        draw_text("COLONY COLLAPSED", cw / 2.0 - 120.0, ch / 2.0, 40.0, RED);
    }
}
