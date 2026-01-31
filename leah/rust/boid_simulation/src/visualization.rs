use macroquad::prelude::*;
use std::collections::VecDeque;
use crate::constants::{GRAPH_HEIGHT, GRAPH_WIDTH, GRAPH_HISTORY};
use crate::sir::DiseaseModel;

pub struct PopulationHistory {
    susceptible: VecDeque<f32>,
    exposed: VecDeque<f32>,
    infected: VecDeque<f32>,
    recovered: VecDeque<f32>,
}

impl PopulationHistory {
    pub fn new() -> Self {
        Self {
            susceptible: VecDeque::new(),
            exposed: VecDeque::new(),
            infected: VecDeque::new(),
            recovered: VecDeque::new(),
        }
    }

    pub fn add(&mut self, s: usize, e: usize, i: usize, r: usize) {
        self.susceptible.push_back(s as f32);
        self.exposed.push_back(e as f32);
        self.infected.push_back(i as f32);
        self.recovered.push_back(r as f32);

        if self.susceptible.len() > GRAPH_HISTORY {
            self.susceptible.pop_front();
            self.exposed.pop_front();
            self.infected.pop_front();
            self.recovered.pop_front();
        }
    }

    pub fn clear(&mut self) {
        self.susceptible.clear();
        self.exposed.clear();
        self.infected.clear();
        self.recovered.clear();
    }

    pub fn draw(&self, x: f32, y: f32, total_boids: f32, model: DiseaseModel) {
        draw_rectangle(x, y, GRAPH_WIDTH, GRAPH_HEIGHT, Color::from_rgba(20, 20, 20, 255));
        draw_rectangle_lines(x, y, GRAPH_WIDTH, GRAPH_HEIGHT, 2.0, GRAY);

        let title = match model {
            DiseaseModel::SIR => "SIR Population Over Time",
            DiseaseModel::SIS => "SIS Population Over Time",
            DiseaseModel::SEIR => "SEIR Population Over Time",
        };
        draw_text(title, x + 10.0, y + 20.0, 20.0, WHITE);

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

            // Only draw Exposed line for SEIR model
            if model == DiseaseModel::SEIR {
                let e1 = y + GRAPH_HEIGHT - (self.exposed[i - 1] / max_val) * (GRAPH_HEIGHT - 30.0);
                let e2 = y + GRAPH_HEIGHT - (self.exposed[i] / max_val) * (GRAPH_HEIGHT - 30.0);
                draw_line(x1, e1, x2, e2, 2.0, Color::from_rgba(255, 200, 0, 255));
            }

            let i1 = y + GRAPH_HEIGHT - (self.infected[i - 1] / max_val) * (GRAPH_HEIGHT - 30.0);
            let i2 = y + GRAPH_HEIGHT - (self.infected[i] / max_val) * (GRAPH_HEIGHT - 30.0);
            draw_line(x1, i1, x2, i2, 2.0, RED);

            // Only draw Recovered line for SIR and SEIR models
            if model != DiseaseModel::SIS {
                let r1 = y + GRAPH_HEIGHT - (self.recovered[i - 1] / max_val) * (GRAPH_HEIGHT - 30.0);
                let r2 = y + GRAPH_HEIGHT - (self.recovered[i] / max_val) * (GRAPH_HEIGHT - 30.0);
                draw_line(x1, r1, x2, r2, 2.0, BLUE);
            }
        }

        // Draw legend based on model
        let legend_x = x + GRAPH_WIDTH - 250.0;
        let legend_y = y + 40.0;
        draw_text("S", legend_x, legend_y, 18.0, WHITE);

        let mut offset = 50.0;
        if model == DiseaseModel::SEIR {
            draw_text("E", legend_x + offset, legend_y, 18.0, Color::from_rgba(255, 200, 0, 255));
            offset += 50.0;
        }

        draw_text("I", legend_x + offset, legend_y, 18.0, RED);
        offset += 50.0;

        if model != DiseaseModel::SIS {
            draw_text("R", legend_x + offset, legend_y, 18.0, BLUE);
        }
    }
}
