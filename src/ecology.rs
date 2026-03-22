// Phase 4 — Living Ecology
// FoodSource with regrowth + saturation decay, natural spawning, day/night cycle.

use macroquad::prelude::*;

use crate::world::{Cell, World};

const DAY_LENGTH: f32 = 300.0;           // seconds per sim-day (5 min real-time)
const REGROWTH_RATE: f32 = 1.5;          // food units per second per cell (base, mid-day)
const SATURATION_TIMEOUT: f32 = 30.0;   // seconds at full max before max_quantity starts decaying
const MAX_DECAY_RATE: f32 = 0.3;         // max_quantity units lost per second while saturated
const MIN_MAX_QUANTITY: f32 = 5.0;       // floor so sources never wither completely
const INITIAL_QUANTITY: f32 = 30.0;      // starting food units per cell
const PATCH_SIZE: i32 = 4;              // grid cells per side (4×4 patch)

const SPAWN_INTERVAL_MIN: f32 = 120.0;  // seconds between spawn events (min)
const SPAWN_INTERVAL_MAX: f32 = 240.0;  // seconds between spawn events (max)
const SPAWN_BATCH_MIN: usize = 1;       // sources dropped per spawn event (min)
const SPAWN_BATCH_MAX: usize = 2;       // sources dropped per spawn event (max)
const MAX_SOURCES: usize = 9;           // cap on total simultaneous food sources
const MIN_SOURCE_DIST: f32 = 80.0;      // minimum world-pixel spacing between source centers
const SPAWN_SEARCH_DIST_MIN: f32 = 80.0;
const SPAWN_SEARCH_DIST_MAX: f32 = 220.0;

pub struct FoodSource {
    pub center: Vec2,               // world coords of patch center
    pub cells: Vec<(usize, usize)>, // grid (gx, gy) pairs covered by this source
    pub max_quantity: f32,          // current ceiling (decays when saturated too long)
    regrowth_rate: f32,
    saturation_timer: f32,          // seconds continuously at max
}

impl FoodSource {
    /// Place a new food source centered on grid cell (center_gx, center_gy).
    /// Writes Food cells into `world`.
    pub fn new(center_gx: usize, center_gy: usize, world: &mut World) -> Self {
        let mut cells: Vec<(usize, usize)> = Vec::new();

        let half = PATCH_SIZE / 2; // offset so patch is centered: -2..2 for size 4
        for dy in -half..PATCH_SIZE - half {
            for dx in -half..PATCH_SIZE - half {
                let gx = center_gx as i32 + dx;
                let gy = center_gy as i32 + dy;
                // Keep inside the map, away from border walls (index 0 and last row/col are walls)
                if gx <= 0 || gy <= 0
                    || gx >= world.width as i32 - 1
                    || gy >= world.height as i32 - 1
                {
                    continue;
                }
                let idx = gy as usize * world.width + gx as usize;
                if world.cells[idx] == Cell::Empty {
                    world.cells[idx] = Cell::Food;
                    world.food_quantities[idx] = INITIAL_QUANTITY;
                    cells.push((gx as usize, gy as usize));
                }
            }
        }

        let center = Vec2::new(
            center_gx as f32 * world.cell_size + world.cell_size * 0.5,
            center_gy as f32 * world.cell_size + world.cell_size * 0.5,
        );

        FoodSource {
            center,
            cells,
            max_quantity: INITIAL_QUANTITY,
            regrowth_rate: REGROWTH_RATE,
            saturation_timer: 0.0,
        }
    }

    fn update(&mut self, dt: f32, world: &mut World, day_modifier: f32) {
        let rate = self.regrowth_rate * day_modifier * dt;
        let mut all_at_max = true;

        for &(gx, gy) in &self.cells {
            let idx = gy * world.width + gx;
            let q = &mut world.food_quantities[idx];

            if *q < self.max_quantity {
                *q = (*q + rate).min(self.max_quantity);
                all_at_max = false;
            }

            // Restore cell type if quantity regrew from zero
            if *q > 0.0 && world.cells[idx] == Cell::Empty {
                world.cells[idx] = Cell::Food;
            }
        }

        // Saturation decay: if source stays full too long, its ceiling lowers (seasonal)
        if all_at_max {
            self.saturation_timer += dt;
            if self.saturation_timer > SATURATION_TIMEOUT {
                self.max_quantity =
                    (self.max_quantity - MAX_DECAY_RATE * dt).max(MIN_MAX_QUANTITY);
                // Also lower stored quantities that exceed new max
                for &(gx, gy) in &self.cells {
                    let idx = gy * world.width + gx;
                    if world.food_quantities[idx] > self.max_quantity {
                        world.food_quantities[idx] = self.max_quantity;
                    }
                }
            }
        } else {
            self.saturation_timer = 0.0;
        }
    }
}

pub struct Ecology {
    pub sources: Vec<FoodSource>,
    pub day_time: f32,   // 0..day_length, where 0 = start of day
    pub day_length: f32, // seconds per sim-day
    pub day_count: u32,
    spawn_timer: f32,
}

impl Ecology {
    /// Initialise with the two default food clusters; writes them into `world`.
    pub fn new(world: &mut World) -> Self {
        let w = world.width;
        let h = world.height;

        let mut sources = Vec::new();
        // Top-right and bottom-left quadrants (matching previous hardcoded layout)
        sources.push(FoodSource::new(w / 4 * 3, h / 4, world));
        sources.push(FoodSource::new(w / 4, h / 4 * 3, world));

        Ecology {
            sources,
            day_time: 0.0,
            day_length: DAY_LENGTH,
            day_count: 0,
            spawn_timer: next_spawn_interval(),
        }
    }

    /// Advance the ecology simulation by `dt` seconds.
    pub fn update(&mut self, dt: f32, world: &mut World) {
        // Day/night clock
        self.day_time += dt;
        if self.day_time >= self.day_length {
            self.day_time -= self.day_length;
            self.day_count += 1;
        }

        let modifier = self.day_modifier();

        // Regrow existing sources
        for source in &mut self.sources {
            source.update(dt, world, modifier);
        }

        // Spawn new sources on a random timer (capped at MAX_SOURCES)
        if self.sources.len() < MAX_SOURCES {
            self.spawn_timer -= dt;
            if self.spawn_timer <= 0.0 {
                self.spawn_timer = next_spawn_interval();
                let batch = {
                    use ::rand::Rng;
                    ::rand::thread_rng().gen_range(SPAWN_BATCH_MIN..=SPAWN_BATCH_MAX)
                };
                for _ in 0..batch {
                    if self.sources.len() >= MAX_SOURCES { break; }
                    self.try_spawn(world);
                }
            }
        }
    }

    /// [0..1]: 0 = full daylight (noon), 1 = full darkness (midnight).
    pub fn night_amount(&self) -> f32 {
        (1.0 - (self.day_time / self.day_length * std::f32::consts::TAU).cos()) / 2.0
    }

    /// Regrowth multiplier: 1.5× at noon, 0.5× at midnight.
    pub fn day_modifier(&self) -> f32 {
        1.5 - self.night_amount() // 1.5 (day) → 0.5 (night)
    }

    pub fn is_day(&self) -> bool {
        self.night_amount() < 0.5
    }

    /// Place a food source at the given grid cell (ignores the source cap).
    pub fn add_source_at_grid(&mut self, gx: usize, gy: usize, world: &mut World) {
        let source = FoodSource::new(gx, gy, world);
        if !source.cells.is_empty() {
            self.sources.push(source);
        }
    }

    fn try_spawn(&mut self, world: &mut World) {
        use ::rand::Rng;
        let mut rng = ::rand::thread_rng();

        if self.sources.is_empty() {
            return;
        }

        // Pick a random existing source as the spawn anchor
        let parent_idx = rng.gen_range(0..self.sources.len());
        let parent_center = self.sources[parent_idx].center;

        // Try up to 25 candidate positions around the parent
        for _ in 0..25 {
            let angle = rng.gen_range(0.0f32..std::f32::consts::TAU);
            let dist = rng.gen_range(SPAWN_SEARCH_DIST_MIN..SPAWN_SEARCH_DIST_MAX);
            let candidate_world = parent_center + Vec2::new(angle.cos(), angle.sin()) * dist;

            let gx = (candidate_world.x / world.cell_size) as i32;
            let gy = (candidate_world.y / world.cell_size) as i32;

            // Must be inside map with margin for patch half-size + 1
            let margin = PATCH_SIZE / 2 + 1;
            if gx < margin
                || gy < margin
                || gx >= world.width as i32 - margin
                || gy >= world.height as i32 - margin
            {
                continue;
            }

            // Reject positions on or adjacent to walls
            if world.cells[gy as usize * world.width + gx as usize] == Cell::Wall {
                continue;
            }

            // Enforce minimum distance between source centers
            let candidate_world_snapped = Vec2::new(
                gx as f32 * world.cell_size + world.cell_size * 0.5,
                gy as f32 * world.cell_size + world.cell_size * 0.5,
            );
            let too_close = self
                .sources
                .iter()
                .any(|s| s.center.distance(candidate_world_snapped) < MIN_SOURCE_DIST);
            if too_close {
                continue;
            }

            self.sources
                .push(FoodSource::new(gx as usize, gy as usize, world));
            break;
        }
    }
}

fn next_spawn_interval() -> f32 {
    use ::rand::Rng;
    ::rand::thread_rng().gen_range(SPAWN_INTERVAL_MIN..SPAWN_INTERVAL_MAX)
}
