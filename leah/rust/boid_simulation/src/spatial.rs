use std::collections::HashMap;
use macroquad::prelude::Vec2;
use crate::boid::Boid;

pub struct SpatialGrid {
    cell_size: f32,
    cells: HashMap<(i32, i32), Vec<usize>>,
}

impl SpatialGrid {
    pub fn new(cell_size: f32) -> Self {
        Self {
            cell_size,
            cells: HashMap::new(),
        }
    }

    pub fn clear(&mut self) {
        self.cells.clear();
    }

    pub fn insert(&mut self, index: usize, position: Vec2) {
        let cell = self.get_cell(position);
        self.cells.entry(cell).or_insert_with(Vec::new).push(index);
    }

    pub fn query_nearby(&self, position: Vec2, radius: f32, boids: &[Boid]) -> Vec<(Vec2, Vec2)> {
        let mut nearby = Vec::new();

        // Determine which cells to check
        let min_cell = self.get_cell(Vec2::new(position.x - radius, position.y - radius));
        let max_cell = self.get_cell(Vec2::new(position.x + radius, position.y + radius));

        // Check all cells in the range
        for x in min_cell.0..=max_cell.0 {
            for y in min_cell.1..=max_cell.1 {
                if let Some(indices) = self.cells.get(&(x, y)) {
                    for &idx in indices {
                        nearby.push((boids[idx].position, boids[idx].velocity));
                    }
                }
            }
        }

        nearby
    }

    pub fn query_nearby_indices(&self, position: Vec2, radius: f32) -> Vec<usize> {
        let mut nearby = Vec::new();

        let min_cell = self.get_cell(Vec2::new(position.x - radius, position.y - radius));
        let max_cell = self.get_cell(Vec2::new(position.x + radius, position.y + radius));

        for x in min_cell.0..=max_cell.0 {
            for y in min_cell.1..=max_cell.1 {
                if let Some(indices) = self.cells.get(&(x, y)) {
                    nearby.extend_from_slice(indices);
                }
            }
        }

        nearby
    }

    fn get_cell(&self, position: Vec2) -> (i32, i32) {
        (
            (position.x / self.cell_size).floor() as i32,
            (position.y / self.cell_size).floor() as i32,
        )
    }
}
