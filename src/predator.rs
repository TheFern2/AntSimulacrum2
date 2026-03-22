// Phase 6 — Predators (Spiders)
// Normal mode only. Spiders spawn from map edges, hunt ants, and are fought by soldiers.

use macroquad::prelude::*;

use crate::colony::GameMode;
use crate::world::{Cell, World};

const SPIDER_SPEED: f32        = 30.0;   // pixels/second when wandering
const SPIDER_HUNT_SPEED: f32   = 82.5;   // 1.5× ant speed (55 × 1.5)
const SPIDER_DETECT_RANGE: f32 = 60.0;   // pixels — triggers Hunting
const SPIDER_LOSE_RANGE: f32   = 150.0;  // pixels — loses target
const SPIDER_WANDER_NOISE: f32 = std::f32::consts::PI / 9.0; // ±20° per tick

pub const SPIDER_KILL_RANGE: f32   = 8.0;  // contact → instant ant kill
pub const SPIDER_MAX_HEALTH: f32   = 100.0;
pub const SPIDER_FEED_TIME: f32    = 10.0;

// Combat (soldier ↔ spider)
pub const COMBAT_RANGE: f32           = 14.0; // pixels at which soldiers deal damage
pub const SOLDIER_DAMAGE_PER_SEC: f32 = 20.0; // hp/s each soldier deals to spider
pub const SPIDER_DAMAGE_PER_SEC: f32  = 35.0; // hp/s spider deals to each attacking soldier

const MAX_SPIDERS: usize       = 3;
const SPAWN_BASE: f32          = 360.0; // seconds at pop = 0
const SPAWN_FLOOR: f32         = 60.0;  // seconds at pop = 500+

#[derive(PartialEq, Debug)]
pub enum SpiderState {
    Wandering,
    Hunting,
    Feeding,
}

pub struct Spider {
    pub position: Vec2,
    pub direction: f32,
    pub health: f32,
    pub state: SpiderState,
    pub hunt_target: Vec2,
    pub feeding_timer: f32,
}

pub struct PredatorSystem {
    pub spiders: Vec<Spider>,
    spawn_timer: f32,
}

impl PredatorSystem {
    pub fn new() -> Self {
        use ::rand::Rng;
        let init_delay = ::rand::thread_rng().gen_range(SPAWN_BASE * 0.5..SPAWN_BASE);
        Self { spiders: Vec::new(), spawn_timer: init_delay }
    }

    pub fn reset(&mut self) { *self = Self::new(); }

    /// Advance spider AI. Returns true if a new spider spawned this frame.
    /// `soldiers` — current soldier count; spiders won't spawn until at least 1 exists.
    pub fn update(
        &mut self,
        dt: f32,
        world: &World,
        ant_positions: &[Vec2],
        mode: GameMode,
        pop: usize,
        soldiers: usize,
    ) -> bool {
        if mode == GameMode::Zen {
            self.spiders.clear();
            return false;
        }

        let mut rng = ::rand::thread_rng();

        // Hold the spawn timer until the colony has at least one soldier
        if soldiers == 0 {
            for spider in &mut self.spiders {
                spider.tick(dt, world, ant_positions, &mut rng);
            }
            return false;
        }

        // Spawn timer
        self.spawn_timer -= dt;
        let mut spawned = false;
        if self.spawn_timer <= 0.0 {
            if self.spiders.len() < MAX_SPIDERS {
                self.spawn_spider(world, &mut rng);
                spawned = true;
            }
            // Shorter interval as colony grows
            let t = (pop as f32 / 500.0).min(1.0);
            self.spawn_timer = SPAWN_BASE * (1.0 - t) + SPAWN_FLOOR * t;
        }

        for spider in &mut self.spiders {
            spider.tick(dt, world, ant_positions, &mut rng);
        }

        spawned
    }

    /// Remove spiders with health ≤ 0. Returns count removed.
    pub fn remove_dead(&mut self) -> usize {
        let before = self.spiders.len();
        self.spiders.retain(|s| s.health > 0.0);
        before - self.spiders.len()
    }

    fn spawn_spider(&mut self, world: &World, rng: &mut impl ::rand::Rng) {
        let ws  = world.world_size();
        let cs  = world.cell_size;
        let mg  = cs * 1.5;
        let pos = match rng.gen_range(0..4u8) {
            0 => Vec2::new(mg,        rng.gen_range(mg..ws.y - mg)), // left
            1 => Vec2::new(ws.x - mg, rng.gen_range(mg..ws.y - mg)), // right
            2 => Vec2::new(rng.gen_range(mg..ws.x - mg), mg),        // top
            _ => Vec2::new(rng.gen_range(mg..ws.x - mg), ws.y - mg), // bottom
        };
        let dir = rng.gen_range(0.0f32..std::f32::consts::TAU);
        self.spiders.push(Spider {
            position: pos,
            direction: dir,
            health: SPIDER_MAX_HEALTH,
            state: SpiderState::Wandering,
            hunt_target: pos,
            feeding_timer: 0.0,
        });
    }
}

impl Spider {
    fn tick(
        &mut self,
        dt: f32,
        world: &World,
        ant_positions: &[Vec2],
        rng: &mut impl ::rand::Rng,
    ) {
        match self.state {
            SpiderState::Feeding => {
                self.feeding_timer -= dt;
                if self.feeding_timer <= 0.0 {
                    self.state = SpiderState::Wandering;
                }
            }

            SpiderState::Wandering => {
                let my_pos = self.position;
                if let Some(&nearest) = nearest_in_range(ant_positions, my_pos, SPIDER_DETECT_RANGE) {
                    self.hunt_target = nearest;
                    self.state = SpiderState::Hunting;
                } else {
                    self.direction += rng.gen_range(-SPIDER_WANDER_NOISE..SPIDER_WANDER_NOISE);
                    self.do_move(dt, world, rng, SPIDER_SPEED);
                }
            }

            SpiderState::Hunting => {
                let my_pos = self.position;
                if let Some(&nearest) = nearest_in_range(ant_positions, my_pos, SPIDER_LOSE_RANGE) {
                    self.hunt_target = nearest;
                    let to = self.hunt_target - self.position;
                    if to.length() > 1.0 {
                        self.direction = to.y.atan2(to.x);
                    }
                    self.do_move(dt, world, rng, SPIDER_HUNT_SPEED);
                } else {
                    self.state = SpiderState::Wandering;
                }
            }
        }
    }

    /// Transition to Feeding state after killing an ant.
    pub fn feed(&mut self) {
        self.state   = SpiderState::Feeding;
        self.feeding_timer = SPIDER_FEED_TIME;
    }

    fn do_move(&mut self, dt: f32, world: &World, rng: &mut impl ::rand::Rng, speed: f32) {
        let vel = Vec2::new(self.direction.cos(), self.direction.sin()) * speed * dt;
        let new_pos = self.position + vel;
        let gx = (new_pos.x / world.cell_size) as i32;
        let gy = (new_pos.y / world.cell_size) as i32;
        match world.get_checked(gx, gy) {
            None | Some(Cell::Wall) => {
                self.direction += std::f32::consts::PI + rng.gen_range(-0.5f32..0.5);
            }
            _ => {
                self.position = new_pos;
            }
        }
    }
}

fn nearest_in_range<'a>(positions: &'a [Vec2], origin: Vec2, range: f32) -> Option<&'a Vec2> {
    positions
        .iter()
        .filter(|p| p.distance(origin) <= range)
        .min_by(|a, b| {
            a.distance(origin)
                .partial_cmp(&b.distance(origin))
                .unwrap_or(std::cmp::Ordering::Equal)
        })
}
