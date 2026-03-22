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

/// Draw the entire game world.
/// `colony_data`: `(colony_color, pheromones, ants)` per colony.
/// `nest_data`: `(nest_pos, colony_color)` per colony.
pub fn draw_scene(
    world: &World,
    camera: &Camera,
    colony_data: &[(Color, &PheromoneGrid, &[Ant])],
    nest_data: &[(Vec2, Color)],
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
        draw_pheromones(world, camera, colony_data, phero_vis);
    }
    draw_world_cells(world, camera);
    draw_nests(camera, nest_data);
    draw_ants_multi(camera, colony_data, show_ant_labels);
    draw_spiders(camera, spiders);
    if is_raining {
        draw_rain_overlay();
    }
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

fn draw_pheromones(
    world: &World,
    camera: &Camera,
    colony_data: &[(Color, &PheromoneGrid, &[Ant])],
    vis: PheromoneVis,
) {
    let cs = world.cell_size * camera.zoom();

    for (colony_color, pheromones, _) in colony_data {
        for y in 0..world.height {
            for x in 0..world.width {
                let idx = y * world.width + x;
                let home = pheromones.to_home[idx];
                let food = pheromones.to_food[idx];

                if home <= 0.01 && food <= 0.01 { continue; }

                let sp = camera.world_to_screen(Vec2::new(
                    x as f32 * world.cell_size,
                    y as f32 * world.cell_size,
                ));

                // Draw food trail (warmer/brighter tint of colony color)
                if food > 0.01 && matches!(vis, PheromoneVis::Both | PheromoneVis::ToFood) {
                    let alpha = (food / MAX_INTENSITY).min(1.0) * 0.65;
                    let c = Color {
                        r: colony_color.r * 0.9 + 0.1,
                        g: colony_color.g * 0.7,
                        b: colony_color.b * 0.3,
                        a: alpha,
                    };
                    draw_rectangle(sp.x, sp.y, cs, cs, c);
                }
                // Draw home trail (cooler/darker tint of colony color)
                if home > 0.01 && matches!(vis, PheromoneVis::Both | PheromoneVis::ToHome) {
                    let alpha = (home / MAX_INTENSITY).min(1.0) * 0.65;
                    let c = Color {
                        r: colony_color.r * 0.3,
                        g: colony_color.g * 0.5,
                        b: colony_color.b * 0.9 + 0.1,
                        a: alpha,
                    };
                    draw_rectangle(sp.x, sp.y, cs, cs, c);
                }
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
}

fn draw_nests(camera: &Camera, nest_data: &[(Vec2, Color)]) {
    for (nest_pos, colony_color) in nest_data {
        let nest_s = camera.world_to_screen(*nest_pos);
        let base_r = 20.0 * camera.zoom();

        let outer = Color { a: 0.4, ..*colony_color };
        let mid   = Color { a: 0.6, ..*colony_color };
        let inner = Color { a: 0.9, ..*colony_color };

        draw_circle(nest_s.x, nest_s.y, base_r * 2.2, outer);
        draw_circle(nest_s.x, nest_s.y, base_r * 1.4, mid);
        draw_circle(nest_s.x, nest_s.y, base_r, inner);
    }
}

fn draw_ants_multi(
    camera: &Camera,
    colony_data: &[(Color, &PheromoneGrid, &[Ant])],
    show_labels: bool,
) {
    let ant_r = 2.5 * camera.zoom();
    let tick_len = 5.0 * camera.zoom();

    for (colony_color, _, ants) in colony_data {
        for ant in *ants {
            let sp = camera.world_to_screen(ant.position);

            // Carrying food → bright green; else tint by caste lightness within colony hue
            let color = if ant.carrying_food {
                Color { r: 0.4, g: 1.0, b: 0.4, a: 1.0 }
            } else {
                match ant.caste {
                    Caste::Worker  => *colony_color,
                    Caste::Scout   => lighten(*colony_color, 0.35),
                    Caste::Soldier => darken(*colony_color, 0.25),
                    Caste::Nurse   => Color {
                        r: colony_color.r * 0.7 + 0.25,
                        g: colony_color.g * 0.5 + 0.35,
                        b: colony_color.b * 0.7 + 0.25,
                        a: 1.0,
                    },
                }
            };

            draw_circle(sp.x, sp.y, ant_r, color);

            // Direction tick
            let tip = sp + Vec2::new(ant.direction.cos(), ant.direction.sin()) * tick_len;
            draw_line(sp.x, sp.y, tip.x, tip.y, 1.0, color);

            // Ring highlight for returning ants
            if ant.state == AntState::Returning {
                draw_circle_lines(sp.x, sp.y, ant_r + 1.0, 0.5,
                    Color { r: 0.4, g: 1.0, b: 0.4, a: 1.0 });
            }

            if show_labels {
                let label = match ant.caste {
                    Caste::Worker  => "W", Caste::Scout  => "Sc",
                    Caste::Soldier => "So", Caste::Nurse => "N",
                };
                draw_text(label, sp.x + ant_r + 1.0, sp.y - ant_r, 10.0, color);
            }
        }
    }
}

fn lighten(c: Color, amount: f32) -> Color {
    Color {
        r: (c.r + amount).min(1.0),
        g: (c.g + amount).min(1.0),
        b: (c.b + amount).min(1.0),
        a: c.a,
    }
}

fn darken(c: Color, amount: f32) -> Color {
    Color {
        r: (c.r - amount).max(0.0),
        g: (c.g - amount).max(0.0),
        b: (c.b - amount).max(0.0),
        a: c.a,
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
    draw_rectangle(0.0, 0.0, sw, sh,
        Color { r: 0.05, g: 0.10, b: 0.35, a: RAIN_OVERLAY_ALPHA });
    let streak_color = Color { r: 0.50, g: 0.65, b: 1.0, a: 0.15 };
    let cols = (sw / 18.0) as usize;
    for i in 0..cols {
        let x = i as f32 * 18.0 + 5.0;
        let h = 12.0 + ((i * 7 + 3) % 14) as f32;
        let y = ((i * 31 + 17) % (sh as usize).max(1)) as f32;
        draw_line(x, y, x - 2.0, y + h, 1.0, streak_color);
    }
}

pub fn draw_debug_overlay(
    fps: f32,
    colonies: &[(&Colony, &[Ant], &[BroodMember])],
    decay_rate: f32,
    ecology: &Ecology,
) {
    let mut y = 20.0f32;
    let line = 20.0f32;

    draw_text(&format!("FPS: {:.0}", fps), 8.0, y, 20.0, WHITE); y += line;

    let mode_label = if !colonies.is_empty() {
        match colonies[0].0.mode {
            crate::colony::GameMode::Zen    => "ZEN",
            crate::colony::GameMode::Normal => "NORMAL",
        }
    } else { "?" };
    draw_text(&format!("[{}]", mode_label), 8.0, y, 18.0, GRAY); y += line;

    for (colony, ants, brood) in colonies {
        let workers  = ants.iter().filter(|a| a.caste == Caste::Worker).count();
        let scouts   = ants.iter().filter(|a| a.caste == Caste::Scout).count();
        let soldiers = ants.iter().filter(|a| a.caste == Caste::Soldier).count();
        let nurses   = ants.iter().filter(|a| a.caste == Caste::Nurse).count();
        let eggs     = brood.iter().filter(|b| b.stage == BroodStage::Egg).count();
        let larvae   = brood.iter().filter(|b| b.stage == BroodStage::Larva).count();

        let col_r = (colony.color.r * 255.0) as u8;
        let col_g = (colony.color.g * 255.0) as u8;
        let col_b = (colony.color.b * 255.0) as u8;
        draw_text(
            &format!("C{} #{:02X}{:02X}{:02X}  Ants:{} (W:{} Sc:{} So:{} N:{})  Brood:{} (e:{} l:{})  Food:{:.0}  Q:{}",
                colony.id, col_r, col_g, col_b,
                ants.len(), workers, scouts, soldiers, nurses,
                brood.len(), eggs, larvae,
                colony.food_stored, colony.queen.status_label()),
            8.0, y, 15.0, WHITE,
        );
        y += line;
    }

    y += 4.0;
    draw_text(&format!("Decay: {:.3}/s  [Shift+↑/↓]", decay_rate), 8.0, y, 16.0, GRAY); y += 20.0;
    let day_icon = if ecology.is_day() { "day" } else { "night" };
    draw_text(
        &format!("Day {} [{}]  Sources: {}",
            ecology.day_count + 1, day_icon, ecology.sources.len()),
        8.0, y, 16.0, GRAY,
    ); y += 20.0;
    draw_text("Shift+R: reset  |  Shift+M: toggle mode  |  F1: hide debug", 8.0, y, 16.0, GRAY);
}
