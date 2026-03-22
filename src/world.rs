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
    pub nest_pos: Vec2,
    pub cell_size: f32,
    pub food_stored: f32,
}

const FOOD_INITIAL: f32 = 30.0;

impl World {
    pub fn new(width: usize, height: usize, cell_size: f32) -> Self {
        let mut cells = vec![Cell::Empty; width * height];
        let mut food_quantities = vec![0.0f32; width * height];

        // Border walls
        for x in 0..width {
            cells[x] = Cell::Wall;
            cells[(height - 1) * width + x] = Cell::Wall;
        }
        for y in 0..height {
            cells[y * width] = Cell::Wall;
            cells[y * width + width - 1] = Cell::Wall;
        }

        // Interior wall segments
        for y in 10..20 {
            cells[y * width + 20] = Cell::Wall;
        }
        for x in 30..45 {
            cells[15 * width + x] = Cell::Wall;
        }

        // Food sources (top-right and bottom-left quadrants)
        let food_cells: Vec<(usize, usize)> = {
            let mut v = vec![];
            for dy in -3i32..=3 {
                for dx in -3i32..=3 {
                    let fx = (width as i32 / 4 * 3 + dx) as usize;
                    let fy = (height as i32 / 4 + dy) as usize;
                    v.push((fx, fy));
                    let fx2 = (width as i32 / 4 + dx) as usize;
                    let fy2 = (height as i32 / 4 * 3 + dy) as usize;
                    v.push((fx2, fy2));
                }
            }
            v
        };

        for (fx, fy) in food_cells {
            let idx = fy * width + fx;
            cells[idx] = Cell::Food;
            food_quantities[idx] = FOOD_INITIAL;
        }

        let nest_pos = Vec2::new((width as f32 / 2.0) * cell_size, (height as f32 / 2.0) * cell_size);

        Self { width, height, cells, food_quantities, nest_pos, cell_size, food_stored: 0.0 }
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
    /// If quantity reaches zero the cell reverts to Empty.
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
}
