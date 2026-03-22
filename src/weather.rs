// Phase 6 — Rain storms
// Active in Normal + Zen modes.
// Pheromones decay 5× faster, ants move at 70% speed, food regrowth halved.

pub const RAIN_DECAY_MULT: f32 = 5.0;
pub const RAIN_SPEED_MULT: f32 = 0.7;
pub const RAIN_FOOD_MULT:  f32 = 0.5;
pub const RAIN_OVERLAY_ALPHA: f32 = 0.22;

const COOLDOWN_MIN: f32 = 1200.0; // ~4 sim-days (4 × 300s)
const COOLDOWN_MAX: f32 = 2400.0; // ~8 sim-days
const WARN_LEAD:    f32 = 10.0;   // seconds of warning before storm begins
const STORM_MIN:    f32 = 30.0;
const STORM_MAX:    f32 = 60.0;

#[derive(PartialEq, Debug)]
enum Phase {
    Cooldown,
    Warning, // 10s pre-storm
    Storm,
}

pub struct Weather {
    phase: Phase,
    timer: f32, // seconds remaining in current phase
}

pub struct WeatherEvents {
    pub warn_triggered:  bool, // "Rain approaching..." toast
    pub storm_ended:     bool, // "Rain cleared" toast
}

impl Weather {
    pub fn new() -> Self {
        use ::rand::Rng;
        let timer = ::rand::thread_rng().gen_range(COOLDOWN_MIN..COOLDOWN_MAX);
        Self { phase: Phase::Cooldown, timer }
    }

    pub fn reset(&mut self) { *self = Self::new(); }

    pub fn update(&mut self, dt_sim: f32) -> WeatherEvents {
        let mut ev = WeatherEvents { warn_triggered: false, storm_ended: false };
        self.timer -= dt_sim;
        if self.timer > 0.0 { return ev; }

        match self.phase {
            Phase::Cooldown => {
                self.phase = Phase::Warning;
                self.timer = WARN_LEAD;
                ev.warn_triggered = true;
            }
            Phase::Warning => {
                use ::rand::Rng;
                self.phase = Phase::Storm;
                self.timer = ::rand::thread_rng().gen_range(STORM_MIN..STORM_MAX);
            }
            Phase::Storm => {
                use ::rand::Rng;
                self.phase = Phase::Cooldown;
                self.timer = ::rand::thread_rng().gen_range(COOLDOWN_MIN..COOLDOWN_MAX);
                ev.storm_ended = true;
            }
        }
        ev
    }

    pub fn is_raining(&self) -> bool { self.phase == Phase::Storm }

    pub fn decay_multiplier(&self) -> f32 { if self.is_raining() { RAIN_DECAY_MULT } else { 1.0 } }
    pub fn speed_multiplier(&self) -> f32 { if self.is_raining() { RAIN_SPEED_MULT } else { 1.0 } }
    pub fn food_multiplier(&self)  -> f32 { if self.is_raining() { RAIN_FOOD_MULT  } else { 1.0 } }
}
