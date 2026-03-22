pub const MAX_INTENSITY: f32 = 10.0;
pub const DEFAULT_DECAY_RATE: f32 = 0.087;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum PheromoneVis {
    Both,
    ToFood,
    ToHome,
    Off,
}

pub struct PheromoneGrid {
    pub width: usize,
    pub height: usize,
    pub to_home: Vec<f32>,
    pub to_food: Vec<f32>,
}

impl PheromoneGrid {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            to_home: vec![0.0; width * height],
            to_food: vec![0.0; width * height],
        }
    }

    pub fn deposit_home(&mut self, gx: i32, gy: i32, amount: f32) {
        if let Some(idx) = self.idx(gx, gy) {
            self.to_home[idx] = (self.to_home[idx] + amount).min(MAX_INTENSITY);
        }
    }

    pub fn deposit_food(&mut self, gx: i32, gy: i32, amount: f32) {
        if let Some(idx) = self.idx(gx, gy) {
            self.to_food[idx] = (self.to_food[idx] + amount).min(MAX_INTENSITY);
        }
    }

    /// Read-only sample — no degradation (used during 32-dir sweep)
    pub fn read_food(&self, gx: i32, gy: i32) -> f32 {
        self.idx(gx, gy).map(|i| self.to_food[i]).unwrap_or(0.0)
    }

    pub fn read_home(&self, gx: i32, gy: i32) -> f32 {
        self.idx(gx, gy).map(|i| self.to_home[i]).unwrap_or(0.0)
    }

    /// Degrade the chosen best cell (prevents trail oversaturation)
    pub fn degrade_food(&mut self, gx: i32, gy: i32) {
        if let Some(idx) = self.idx(gx, gy) {
            self.to_food[idx] *= 0.999;
        }
    }

    pub fn degrade_home(&mut self, gx: i32, gy: i32) {
        if let Some(idx) = self.idx(gx, gy) {
            self.to_home[idx] *= 0.999;
        }
    }

    pub fn decay(&mut self, dt: f32, rate: f32) {
        let d = rate * dt;
        for v in self.to_home.iter_mut() {
            *v = (*v - d).max(0.0);
        }
        for v in self.to_food.iter_mut() {
            *v = (*v - d).max(0.0);
        }
    }

    fn idx(&self, gx: i32, gy: i32) -> Option<usize> {
        if gx < 0 || gy < 0 || gx >= self.width as i32 || gy >= self.height as i32 {
            None
        } else {
            Some(gy as usize * self.width + gx as usize)
        }
    }
}
