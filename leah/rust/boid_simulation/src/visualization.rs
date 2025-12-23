use macroquad::prelude::*;
use std::collections::VecDeque;
use crate::constants::{GRAPH_HEIGHT, GRAPH_WIDTH, GRAPH_HISTORY};

pub struct PopulationHistory {
    susceptible: VecDeque<f32>,
    infected: VecDeque<f32>,
    recovered: VecDeque<f32>,
}

impl PopulationHistory {
    pub fn new() -> Self {
        Self {
            susceptible: VecDeque::new(),
            infected: VecDeque::new(),
            recovered: VecDeque::new(),
        }
    }

    pub fn add(&mut self, s: usize, i: usize, r: usize) {
        self.susceptible.push_back(s as f32);
        self.infected.push_back(i as f32);
        self.recovered.push_back(r as f32);

        if self.susceptible.len() > GRAPH_HISTORY {
            self.susceptible.pop_front();
            self.infected.pop_front();
            self.recovered.pop_front();
        }
    }

    pub fn clear(&mut self) {
        self.susceptible.clear();
        self.infected.clear();
        self.recovered.clear();
    }

    pub fn draw(&self, x: f32, y: f32, total_boids: f32) {
        draw_rectangle(x, y, GRAPH_WIDTH, GRAPH_HEIGHT, Color::from_rgba(20, 20, 20, 255));
        draw_rectangle_lines(x, y, GRAPH_WIDTH, GRAPH_HEIGHT, 2.0, GRAY);

        draw_text("SIR Population Over Time", x + 10.0, y + 20.0, 20.0, WHITE);

        if self.susceptible.is_empty() {
            return;
        }

        let max_val = total_boids;
        let len = self.susceptible.len();

        for i in 1..len {
            let x1 = x + ((i - 1) as f32 / GRAPH_HISTORY as f32) * GRAPH_WIDTH;
            let x2 = x + (i as f32 / GRAPH_HISTORY as f32) * GRAPH_WIDTH;

            let s1 = y + GRAPH_HEIGHT - (self.susceptible[i - 1] / max_val) * (GRAPH_HEIGHT - 30.0);
            let s2 = y + GRAPH_HEIGHT - (self.susceptible[i] / max_val) * (GRAPH_HEIGHT - 30.0);
            draw_line(x1, s1, x2, s2, 2.0, WHITE);

            let i1 = y + GRAPH_HEIGHT - (self.infected[i - 1] / max_val) * (GRAPH_HEIGHT - 30.0);
            let i2 = y + GRAPH_HEIGHT - (self.infected[i] / max_val) * (GRAPH_HEIGHT - 30.0);
            draw_line(x1, i1, x2, i2, 2.0, RED);

            let r1 = y + GRAPH_HEIGHT - (self.recovered[i - 1] / max_val) * (GRAPH_HEIGHT - 30.0);
            let r2 = y + GRAPH_HEIGHT - (self.recovered[i] / max_val) * (GRAPH_HEIGHT - 30.0);
            draw_line(x1, r1, x2, r2, 2.0, BLUE);
        }

        let legend_x = x + GRAPH_WIDTH - 200.0;
        let legend_y = y + 40.0;
        draw_text("S", legend_x, legend_y, 18.0, WHITE);
        draw_text("I", legend_x + 50.0, legend_y, 18.0, RED);
        draw_text("R", legend_x + 100.0, legend_y, 18.0, BLUE);
    }
}
