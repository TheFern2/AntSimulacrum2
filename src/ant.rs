use macroquad::prelude::*;

use crate::colony::{Colony, GameMode};
use crate::pheromone::PheromoneGrid;
use crate::world::World;

const ANT_SPEED: f32 = 55.0;                                  // pixels/second
const WANDER_NOISE: f32 = std::f32::consts::PI / 6.0;         // ±30° per tick
const NUM_SAMPLE_DIRS: usize = 32;
const SAMPLE_DIST: f32 = 28.0;                                 // pixels ahead for pheromone sampling
const DEPOSIT_INTERVAL: f32 = 0.12;                            // seconds between deposits
const NEST_RADIUS: f32 = 44.0;                                 // matches visual outer ring

// Caste movement parameters
const SCOUT_SPEED_MULT: f32 = 1.3;
pub const SOLDIER_PATROL_RADIUS: f32 = 85.0;  // pixels from nest — target orbit distance
pub const SOLDIER_SCAN_RANGE: f32 = SOLDIER_PATROL_RADIUS + 20.0; // spider detection range
const SOLDIER_PATROL_BAND: f32 = 20.0;    // ± tolerance before correction kicks in
const NURSE_MAX_RANGE: f32 = 70.0;        // pixels — nurses don't wander further than this

// Combat
const SOLDIER_CHARGE_SPEED: f32 = ANT_SPEED * 1.2; // faster when charging a spider

// Ant lifespans (seconds)
// Equilibrium population ≈ avg_lifespan / egg_interval (7.5s at max food)
// These values target ~100 ants at steady state with full food supply.
const AGE_WORKER: f32 = 720.0;
const AGE_SCOUT: f32 = 480.0;
const AGE_SOLDIER: f32 = 960.0;
const AGE_NURSE: f32 = 960.0;

// Starvation accelerates aging in Normal mode
const STARVATION_AGE_MULT: f32 = 2.5;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Caste {
    Worker,
    Scout,
    Soldier,
    Nurse,
}

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
    pub caste: Caste,
    pub age: f32,
    pub max_age: f32,
    pub health: f32,            // soldiers take combat damage; others effectively immortal
    pub soldier_target: Option<Vec2>, // set each frame by main loop when spider is nearby
    liberty: f32,       // [0..1] probability of skipping pheromone steering each tick
    deposit_timer: f32,
}

impl Ant {
    pub fn new_with_caste(pos: Vec2, caste: Caste) -> Self {
        use ::rand::Rng;
        let mut rng = ::rand::thread_rng();
        let liberty = match caste {
            Caste::Worker  => rng.gen_range(0.05f32..0.25),
            Caste::Scout   => rng.gen_range(0.25f32..0.55),
            Caste::Soldier | Caste::Nurse => rng.gen_range(0.05f32..0.15),
        };
        let max_age = match caste {
            Caste::Worker  => AGE_WORKER,
            Caste::Scout   => AGE_SCOUT,
            Caste::Soldier => AGE_SOLDIER,
            Caste::Nurse   => AGE_NURSE,
        };
        let health = if caste == Caste::Soldier { 100.0 } else { f32::MAX };
        Self {
            position: pos,
            direction: rng.gen_range(0.0..std::f32::consts::TAU),
            state: AntState::Foraging,
            carrying_food: false,
            caste,
            age: 0.0,
            max_age,
            health,
            soldier_target: None,
            liberty,
            deposit_timer: rng.gen_range(0.0..DEPOSIT_INTERVAL),
        }
    }

    /// Tick this ant. Returns `false` if the ant has died and should be removed.
    /// `speed_mult` is applied to ant movement speed (e.g., 0.7 during rain storms).
    pub fn update(
        &mut self,
        dt: f32,
        world: &mut World,
        pheromones: &mut PheromoneGrid,
        colony: &mut Colony,
        speed_mult: f32,
    ) -> bool {
        // Age — starvation accelerates aging in Normal mode
        let age_mult = if colony.mode == GameMode::Normal && colony.is_starving() {
            STARVATION_AGE_MULT
        } else {
            1.0
        };
        self.age += dt * age_mult;
        if self.age >= self.max_age {
            return false;
        }

        let mut rng = ::rand::thread_rng();

        match self.caste {
            Caste::Soldier => self.update_guard(dt, world, &mut rng, speed_mult),
            Caste::Nurse   => self.update_nurse(dt, world, &mut rng, speed_mult),
            _              => self.update_forager(dt, world, pheromones, colony, &mut rng, speed_mult),
        }

        true
    }

    // ── Forager (Worker & Scout) ────────────────────────────────────────────

    fn update_forager(
        &mut self,
        dt: f32,
        world: &mut World,
        pheromones: &mut PheromoneGrid,
        colony: &mut Colony,
        rng: &mut impl ::rand::Rng,
        speed_mult: f32,
    ) {
        let base = if self.caste == Caste::Scout { ANT_SPEED * SCOUT_SPEED_MULT } else { ANT_SPEED };
        let speed = base * speed_mult;

        // Deposit pheromone on current cell
        self.deposit_timer -= dt;
        if self.deposit_timer <= 0.0 {
            self.deposit_timer = DEPOSIT_INTERVAL;
            let gx = (self.position.x / world.cell_size) as i32;
            let gy = (self.position.y / world.cell_size) as i32;
            match self.state {
                AntState::Foraging  => pheromones.deposit_home(gx, gy, 0.3),
                AntState::Returning => pheromones.deposit_food(gx, gy, 0.3),
            }
        }

        // Pheromone steering (unless liberty roll fires)
        // Returns trail strength [0..1] so we can suppress noise on highways
        let trail_strength = if rng.r#gen::<f32>() >= self.liberty {
            let wide_cone = self.caste == Caste::Scout;
            self.steer_by_pheromone(pheromones, world, wide_cone)
        } else {
            0.0
        };

        // Returning ants: direct nest-homing pull (prevents pheromone-ring orbiting)
        if self.state == AntState::Returning {
            let to_nest = world.nest_pos - self.position;
            let dist = to_nest.length();
            if dist > 1.0 {
                let nest_angle = f32::atan2(to_nest.y, to_nest.x);
                let pull = 0.15 + 0.45 * (1.0 - (dist / 300.0).min(1.0));
                let diff = angle_diff(nest_angle, self.direction);
                self.direction += diff * pull;
            }
        }

        // Wander noise — suppressed when on a strong trail so ants stick to highways
        // Strong trail (strength→1.0): noise reduced to ~15% of base
        // No trail (strength=0.0):     full noise for exploration
        let noise_scale = 1.0 - trail_strength * 0.85;
        self.direction += rng.gen_range(-WANDER_NOISE..WANDER_NOISE) * noise_scale;

        // Attempt move
        let vel = Vec2::new(self.direction.cos(), self.direction.sin()) * speed * dt;
        let new_pos = self.position + vel;
        let gx = (new_pos.x / world.cell_size) as i32;
        let gy = (new_pos.y / world.cell_size) as i32;

        use crate::world::Cell;
        match world.get_checked(gx, gy) {
            None | Some(Cell::Wall) => {
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

        // Nest interaction: deposit food, flip to Foraging
        if self.state == AntState::Returning
            && self.position.distance(world.nest_pos) < NEST_RADIUS
        {
            if self.carrying_food {
                colony.deposit_food();
                self.carrying_food = false;
            }
            self.state = AntState::Foraging;
            self.direction += std::f32::consts::PI;
        }
    }

    // ── Soldier (patrol ring around nest; attack spiders when target set) ────

    fn update_guard(
        &mut self,
        dt: f32,
        world: &mut World,
        rng: &mut impl ::rand::Rng,
        speed_mult: f32,
    ) {
        if let Some(target) = self.soldier_target {
            // Attack mode — charge toward the spider
            let to_target = target - self.position;
            if to_target.length() > 1.0 {
                let target_dir = to_target.y.atan2(to_target.x);
                self.direction += angle_diff(target_dir, self.direction) * 0.8;
            }
            self.move_simple(dt, world, rng, SOLDIER_CHARGE_SPEED * speed_mult);
        } else {
            // Patrol mode — orbit the nest
            let to_nest = world.nest_pos - self.position;
            let dist = to_nest.length();
            if dist > SOLDIER_PATROL_RADIUS + SOLDIER_PATROL_BAND {
                let pull_angle = to_nest.y.atan2(to_nest.x);
                self.direction += angle_diff(pull_angle, self.direction) * 0.5;
            } else if dist < SOLDIER_PATROL_RADIUS - SOLDIER_PATROL_BAND && dist > 1.0 {
                let push_angle = (-to_nest.y).atan2(-to_nest.x);
                self.direction += angle_diff(push_angle, self.direction) * 0.3;
            }
            self.direction += rng.gen_range(-WANDER_NOISE * 0.5..WANDER_NOISE * 0.5);
            self.move_simple(dt, world, rng, ANT_SPEED * speed_mult);
        }
    }

    // ── Nurse (stay near nest) ───────────────────────────────────────────────

    fn update_nurse(
        &mut self,
        dt: f32,
        world: &mut World,
        rng: &mut impl ::rand::Rng,
        speed_mult: f32,
    ) {
        let dist = self.position.distance(world.nest_pos);
        if dist > NURSE_MAX_RANGE {
            let to_nest = world.nest_pos - self.position;
            let pull_angle = to_nest.y.atan2(to_nest.x);
            self.direction += angle_diff(pull_angle, self.direction) * 0.6;
        }
        self.direction += rng.gen_range(-WANDER_NOISE..WANDER_NOISE);
        self.move_simple(dt, world, rng, ANT_SPEED * 0.7 * speed_mult);
    }

    // ── Shared movement helper ───────────────────────────────────────────────

    fn move_simple(&mut self, dt: f32, world: &mut World, rng: &mut impl ::rand::Rng, speed: f32) {
        let vel = Vec2::new(self.direction.cos(), self.direction.sin()) * speed * dt;
        let new_pos = self.position + vel;
        let gx = (new_pos.x / world.cell_size) as i32;
        let gy = (new_pos.y / world.cell_size) as i32;

        use crate::world::Cell;
        match world.get_checked(gx, gy) {
            None | Some(Cell::Wall) => {
                self.direction += std::f32::consts::PI + rng.gen_range(-0.5f32..0.5);
            }
            _ => {
                self.position = new_pos;
            }
        }
    }

    // ── Pheromone steering ───────────────────────────────────────────────────

    /// Steers toward the strongest pheromone in the forward cone.
    /// Returns normalized trail strength [0..1] so callers can suppress wander noise.
    fn steer_by_pheromone(
        &mut self,
        pheromones: &mut PheromoneGrid,
        world: &World,
        wide_cone: bool,
    ) -> f32 {
        // Scouts use 180° cone, others use 90°
        let half_cone = if wide_cone {
            std::f32::consts::FRAC_PI_2
        } else {
            std::f32::consts::FRAC_PI_4
        };
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
                AntState::Foraging  => pheromones.read_food(gx, gy),
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
            let diff = angle_diff(best_dir, self.direction);
            self.direction += diff * 0.65; // was 0.4 — stronger correction toward trail

            // Degrade the chosen cell to prevent trail oversaturation
            match self.state {
                AntState::Foraging  => pheromones.degrade_food(best_gx, best_gy),
                AntState::Returning => pheromones.degrade_home(best_gx, best_gy),
            }

            return (best_val / crate::pheromone::MAX_INTENSITY).min(1.0);
        }

        0.0
    }
}

fn angle_diff(target: f32, current: f32) -> f32 {
    use std::f32::consts::PI;
    let mut d = target - current;
    while d > PI  { d -= 2.0 * PI; }
    while d < -PI { d += 2.0 * PI; }
    d
}
