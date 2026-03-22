use macroquad::prelude::*;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Cell {
    Empty,
    Wall,
    Food,
}

pub struct World {
    pub width: usize,
    pub height: usize,
    pub cells: Vec<Cell>,
    pub food_quantities: Vec<f32>,
    pub cell_size: f32,
}

impl World {
    pub fn new(width: usize, height: usize, cell_size: f32) -> Self {
        let mut cells = vec![Cell::Empty; width * height];
        let food_quantities = vec![0.0f32; width * height];

        // Border walls
        for x in 0..width {
            cells[x] = Cell::Wall;
            cells[(height - 1) * width + x] = Cell::Wall;
        }
        for y in 0..height {
            cells[y * width] = Cell::Wall;
            cells[y * width + width - 1] = Cell::Wall;
        }

        // ── Interior walls — designed for 200×150 with colonies at
        //    C0(30,130)  C1(100,20)  C2(170,130)
        // Central horizontal barriers (leave gap at x=90..110 for center passage)
        for x in 50..90  { cells[ 80 * width + x] = Cell::Wall; }
        for x in 110..150{ cells[ 80 * width + x] = Cell::Wall; }

        // Side vertical barriers creating territory lanes
        for y in 45..80 { cells[y * width + 65 ] = Cell::Wall; }
        for y in 45..80 { cells[y * width + 135] = Cell::Wall; }

        // Corner wall features (funnel traffic toward corridors)
        for x in 25..45  { cells[45 * width + x] = Cell::Wall; }
        for x in 155..175{ cells[45 * width + x] = Cell::Wall; }

        // Inner center choke features
        for y in 50..65 { cells[y * width + 90 ] = Cell::Wall; }
        for y in 50..65 { cells[y * width + 110] = Cell::Wall; }

        // Lower center barrier
        for x in 85..115{ cells[105 * width + x] = Cell::Wall; }

        // Scattered inner verticals
        for y in 90..105 { cells[y * width + 75 ] = Cell::Wall; }
        for y in 90..105 { cells[y * width + 125] = Cell::Wall; }

        // Food sources are placed by Ecology::new — not here.

        Self { width, height, cells, food_quantities, cell_size }
    }

    pub fn get(&self, x: usize, y: usize) -> Cell {
        self.cells[y * self.width + x]
    }

    pub fn get_checked(&self, gx: i32, gy: i32) -> Option<Cell> {
        if gx < 0 || gy < 0 || gx >= self.width as i32 || gy >= self.height as i32 {
            None
        } else {
            Some(self.cells[gy as usize * self.width + gx as usize])
        }
    }

    /// Remove one unit of food from a cell. Returns true if food was taken.
    pub fn take_food(&mut self, gx: i32, gy: i32) -> bool {
        if gx < 0 || gy < 0 || gx >= self.width as i32 || gy >= self.height as i32 {
            return false;
        }
        let idx = gy as usize * self.width + gx as usize;
        if self.food_quantities[idx] > 0.0 {
            self.food_quantities[idx] -= 1.0;
            if self.food_quantities[idx] <= 0.0 {
                self.cells[idx] = Cell::Empty;
                self.food_quantities[idx] = 0.0;
            }
            true
        } else {
            false
        }
    }

    pub fn world_size(&self) -> Vec2 {
        Vec2::new(self.width as f32 * self.cell_size, self.height as f32 * self.cell_size)
    }

    /// Center of the world in world coordinates — used for camera init and Home key.
    pub fn world_center(&self) -> Vec2 {
        self.world_size() * 0.5
    }
}
