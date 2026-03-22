use macroquad::prelude::*;

use crate::pheromone::PheromoneGrid;
use crate::world::World;

const ANT_SPEED: f32 = 55.0; // pixels / second
const WANDER_NOISE: f32 = std::f32::consts::PI / 6.0; // ±30° per tick jitter
const NUM_SAMPLE_DIRS: usize = 32;
const SAMPLE_DIST: f32 = 28.0; // pixels ahead for pheromone sampling
const DEPOSIT_INTERVAL: f32 = 0.06; // seconds between deposits
const NEST_RADIUS: f32 = 22.0;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum AntState {
    Foraging,
    Returning,
}

pub struct Ant {
    pub position: Vec2,
    pub direction: f32, // radians
    pub state: AntState,
    pub carrying_food: bool,
    liberty: f32, // [0..1] probability of skipping pheromone steering
    deposit_timer: f32,
}

impl Ant {
    pub fn new(pos: Vec2) -> Self {
        use ::rand::Rng;
        let mut rng = ::rand::thread_rng();
        Self {
            position: pos,
            direction: rng.gen_range(0.0..std::f32::consts::TAU),
            state: AntState::Foraging,
            carrying_food: false,
            liberty: rng.gen_range(0.05f32..0.25),
            deposit_timer: rng.gen_range(0.0..DEPOSIT_INTERVAL),
        }
    }

    pub fn update(&mut self, dt: f32, world: &mut World, pheromones: &mut PheromoneGrid) {
        use ::rand::Rng;
        let mut rng = ::rand::thread_rng();

        // Deposit pheromone on current cell
        self.deposit_timer -= dt;
        if self.deposit_timer <= 0.0 {
            self.deposit_timer = DEPOSIT_INTERVAL;
            let gx = (self.position.x / world.cell_size) as i32;
            let gy = (self.position.y / world.cell_size) as i32;
            match self.state {
                AntState::Foraging => pheromones.deposit_home(gx, gy, 0.6),
                AntState::Returning => pheromones.deposit_food(gx, gy, 0.6),
            }
        }

        // Pheromone steering (unless liberty roll fires)
        if rng.r#gen::<f32>() >= self.liberty {
            self.steer_by_pheromone(pheromones, world);
        }

        // Random forward-cone jitter
        self.direction += rng.gen_range(-WANDER_NOISE..WANDER_NOISE);

        // Attempt move
        let vel = Vec2::new(self.direction.cos(), self.direction.sin()) * ANT_SPEED * dt;
        let new_pos = self.position + vel;

        let gx = (new_pos.x / world.cell_size) as i32;
        let gy = (new_pos.y / world.cell_size) as i32;

        use crate::world::Cell;
        match world.get_checked(gx, gy) {
            None | Some(Cell::Wall) => {
                // Bounce: reverse + small random nudge
                self.direction += std::f32::consts::PI + rng.gen_range(-0.5f32..0.5);
            }
            Some(Cell::Food) => {
                if self.state == AntState::Foraging && world.take_food(gx, gy) {
                    self.carrying_food = true;
                    self.state = AntState::Returning;
                    self.direction += std::f32::consts::PI;
                }
                self.position = new_pos;
            }
            Some(Cell::Empty) => {
                self.position = new_pos;
            }
        }

        // Nest interaction
        if self.state == AntState::Returning && self.position.distance(world.nest_pos) < NEST_RADIUS {
            if self.carrying_food {
                world.food_stored += 1.0;
                self.carrying_food = false;
            }
            self.state = AntState::Foraging;
            self.direction += std::f32::consts::PI;
        }
    }

    fn steer_by_pheromone(&mut self, pheromones: &mut PheromoneGrid, world: &World) {
        let half_cone = std::f32::consts::FRAC_PI_4; // 45° each side = 90° cone
        let step = (2.0 * half_cone) / (NUM_SAMPLE_DIRS as f32 - 1.0);

        let mut best_dir = self.direction;
        let mut best_val = 0.0f32;
        let mut best_gx = 0i32;
        let mut best_gy = 0i32;

        for i in 0..NUM_SAMPLE_DIRS {
            let angle = self.direction - half_cone + step * i as f32;
            let sp = self.position + Vec2::new(angle.cos(), angle.sin()) * SAMPLE_DIST;
            let gx = (sp.x / world.cell_size) as i32;
            let gy = (sp.y / world.cell_size) as i32;

            let val = match self.state {
                AntState::Foraging => pheromones.read_food(gx, gy),
                AntState::Returning => pheromones.read_home(gx, gy),
            };

            if val > best_val {
                best_val = val;
                best_dir = angle;
                best_gx = gx;
                best_gy = gy;
            }
        }

        if best_val > 0.01 {
            // Smoothly rotate toward best direction
            let diff = angle_diff(best_dir, self.direction);
            self.direction += diff * 0.4;

            // Degrade chosen cell to prevent oversaturation
            match self.state {
                AntState::Foraging => pheromones.degrade_food(best_gx, best_gy),
                AntState::Returning => pheromones.degrade_home(best_gx, best_gy),
            }
        }
    }
}

fn angle_diff(target: f32, current: f32) -> f32 {
    use std::f32::consts::PI;
    let mut d = target - current;
    while d > PI { d -= 2.0 * PI; }
    while d < -PI { d += 2.0 * PI; }
    d
}
