// Colony state, Queen, food economy, and game mode.

const BASE_EGG_INTERVAL: f32 = 15.0;        // seconds between eggs at base rate
const FOOD_PER_ANT_SEC: f32 = 0.002;        // food consumed per ant per second
const QUEEN_HEALTH_DECAY_SEC: f32 = 0.00006; // queen health lost per second when starving
const EGG_FOOD_THRESHOLD: f32 = 3.0;        // min food_stored to allow egg laying
const ZEN_MIN_WORKERS: usize = 10;          // Zen mode: never below this many workers

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum GameMode {
    Zen,
    Normal,
}

pub struct Queen {
    pub health: f32,    // 0.0 (dead) → 1.0 (healthy)
    pub egg_timer: f32, // accumulates; egg laid when >= egg_interval
    pub alive: bool,
}

impl Queen {
    pub fn new() -> Self {
        Self { health: 1.0, egg_timer: 0.0, alive: true }
    }

    pub fn status_label(&self) -> &'static str {
        if !self.alive        { "DEAD"     }
        else if self.health < 0.25 { "CRITICAL" }
        else if self.health < 0.60 { "LOW"      }
        else                  { "OK"       }
    }
}

pub struct Colony {
    pub queen: Queen,
    pub food_stored: f32,
    pub mode: GameMode,
    pub colony_age: f32,         // total seconds elapsed
    pub peak_population: u32,
    pub total_food_delivered: u32,
    pub collapsed: bool,
}

impl Colony {
    pub fn new(mode: GameMode) -> Self {
        Self {
            queen: Queen::new(),
            food_stored: 0.0,
            mode,
            colony_age: 0.0,
            peak_population: 0,
            total_food_delivered: 0,
            collapsed: false,
        }
    }

    /// Tick the colony. Returns the number of new eggs the queen lays this frame.
    pub fn update(&mut self, dt: f32, ant_count: usize, brood_count: usize) -> u32 {
        self.colony_age += dt;

        let pop = (ant_count + brood_count) as u32;
        if pop > self.peak_population {
            self.peak_population = pop;
        }

        // Food consumption: ants eat from stored reserves
        let consumed = FOOD_PER_ANT_SEC * ant_count as f32 * dt;
        self.food_stored = (self.food_stored - consumed).max(0.0);

        let starving = self.food_stored < EGG_FOOD_THRESHOLD;

        // Normal mode: starvation drains queen health
        if self.mode == GameMode::Normal && starving && self.queen.alive {
            self.queen.health = (self.queen.health - QUEEN_HEALTH_DECAY_SEC * dt).max(0.0);
            if self.queen.health <= 0.0 {
                self.queen.alive = false;
            }
        }

        // Zen mode: queen is immortal
        if self.mode == GameMode::Zen {
            self.queen.alive = true;
            if self.queen.health < 1.0 {
                self.queen.health = 1.0;
            }
        }

        // No eggs if queen dead or starving
        if !self.queen.alive || starving {
            return 0;
        }

        // Egg rate scales with food stored (0.5x scarce → 2.0x abundant)
        let food_factor = (self.food_stored / 15.0).clamp(0.5, 2.0);
        let egg_interval = BASE_EGG_INTERVAL / food_factor;

        self.queen.egg_timer += dt;
        let mut eggs = 0u32;
        while self.queen.egg_timer >= egg_interval {
            self.queen.egg_timer -= egg_interval;
            eggs += 1;
        }
        eggs
    }

    pub fn deposit_food(&mut self) {
        self.food_stored += 1.0;
        self.total_food_delivered += 1;
    }

    pub fn is_starving(&self) -> bool {
        self.food_stored < EGG_FOOD_THRESHOLD
    }

    pub fn zen_min_workers() -> usize {
        ZEN_MIN_WORKERS
    }

    pub fn check_collapse(&self, ant_count: usize, brood_count: usize) -> bool {
        self.mode == GameMode::Normal
            && !self.queen.alive
            && brood_count == 0
            && ant_count == 0
    }
}
