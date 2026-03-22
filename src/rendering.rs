use macroquad::prelude::*;

use crate::ant::{Ant, AntState, Caste};
use crate::brood::{BroodMember, BroodStage};
use crate::camera::Camera;
use crate::colony::Colony;
use crate::ecology::Ecology;
use crate::pheromone::{PheromoneGrid, PheromoneVis, MAX_INTENSITY};
use crate::predator::{Spider, SpiderState};
use crate::weather::RAIN_OVERLAY_ALPHA;
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
    spiders: &[Spider],
    is_raining: bool,
    phero_vis: PheromoneVis,
    show_ant_labels: bool,
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
    if phero_vis != PheromoneVis::Off {
        draw_pheromones(world, camera, pheromones, phero_vis);
    }
    draw_world_cells(world, camera);
    draw_ants(camera, ants, show_ant_labels);
    draw_spiders(camera, spiders);
    if is_raining {
        draw_rain_overlay();
    }
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

fn draw_pheromones(world: &World, camera: &Camera, pheromones: &PheromoneGrid, vis: PheromoneVis) {
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

            if food > 0.01 && matches!(vis, PheromoneVis::Both | PheromoneVis::ToFood) {
                let alpha = (food / MAX_INTENSITY).min(1.0) * 0.75;
                draw_rectangle(sp.x, sp.y, cs, cs, Color { r: 1.0, g: 0.6, b: 0.0, a: alpha });
            }
            if home > 0.01 && matches!(vis, PheromoneVis::Both | PheromoneVis::ToHome) {
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

fn draw_ants(camera: &Camera, ants: &[Ant], show_labels: bool) {
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

        // Optional caste label
        if show_labels {
            let label = match ant.caste {
                Caste::Worker  => "W", Caste::Scout  => "Sc",
                Caste::Soldier => "So", Caste::Nurse => "N",
            };
            draw_text(label, sp.x + ant_r + 1.0, sp.y - ant_r, 10.0, color);
        }
    }
}

fn draw_spiders(camera: &Camera, spiders: &[Spider]) {
    let base_r = 5.5 * camera.zoom(); // ~4× ant radius (ant is 2.5)
    for spider in spiders {
        let sp = camera.world_to_screen(spider.position);
        let (body_color, ring_color) = match spider.state {
            SpiderState::Wandering => (
                Color { r: 0.25, g: 0.22, b: 0.22, a: 1.0 }, // charcoal
                Color { r: 0.45, g: 0.40, b: 0.38, a: 0.8 },
            ),
            SpiderState::Hunting => (
                Color { r: 0.70, g: 0.15, b: 0.10, a: 1.0 }, // red tint
                Color { r: 0.90, g: 0.20, b: 0.10, a: 0.9 },
            ),
            SpiderState::Feeding => (
                Color { r: 0.35, g: 0.18, b: 0.12, a: 1.0 }, // dark brown when feeding
                Color { r: 0.55, g: 0.30, b: 0.20, a: 0.6 },
            ),
        };
        draw_circle(sp.x, sp.y, base_r, body_color);
        draw_circle_lines(sp.x, sp.y, base_r + 1.0, 1.2, ring_color);

        // Draw leg stubs: 8 short lines at 45° intervals
        for i in 0..8 {
            let angle = i as f32 * std::f32::consts::TAU / 8.0 + spider.direction;
            let tip = sp + Vec2::new(angle.cos(), angle.sin()) * (base_r + 3.5 * camera.zoom());
            let base = sp + Vec2::new(angle.cos(), angle.sin()) * (base_r * 0.6);
            draw_line(base.x, base.y, tip.x, tip.y, 0.8, ring_color);
        }

        // Health bar when damaged
        if spider.health < 95.0 {
            let bar_w = base_r * 3.0;
            let bar_h = 3.0;
            let bar_x = sp.x - bar_w / 2.0;
            let bar_y = sp.y - base_r - 6.0;
            draw_rectangle(bar_x, bar_y, bar_w, bar_h,
                Color { r: 0.6, g: 0.1, b: 0.1, a: 0.8 });
            let frac = (spider.health / crate::predator::SPIDER_MAX_HEALTH).max(0.0);
            draw_rectangle(bar_x, bar_y, bar_w * frac, bar_h,
                Color { r: 0.2, g: 0.8, b: 0.2, a: 0.9 });
        }
    }
}

fn draw_rain_overlay() {
    let sw = screen_width();
    let sh = screen_height();
    // Blue-tinted semi-transparent screen overlay
    draw_rectangle(0.0, 0.0, sw, sh,
        Color { r: 0.05, g: 0.10, b: 0.35, a: RAIN_OVERLAY_ALPHA });
    // Rain streaks
    let streak_color = Color { r: 0.50, g: 0.65, b: 1.0, a: 0.15 };
    let cols = (sw / 18.0) as usize;
    for i in 0..cols {
        let x = i as f32 * 18.0 + 5.0;
        // Deterministic but varied streak heights
        let h = 12.0 + ((i * 7 + 3) % 14) as f32;
        let y = ((i * 31 + 17) % (sh as usize).max(1)) as f32;
        draw_line(x, y, x - 2.0, y + h, 1.0, streak_color);
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
    draw_text("Shift+R: reset  |  Shift+M: toggle mode  |  F1: hide debug", 8.0, y, 16.0, GRAY);
}
