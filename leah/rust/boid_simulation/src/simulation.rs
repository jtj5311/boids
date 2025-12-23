use macroquad::prelude::rand;
use crate::boid::Boid;
use crate::constants::{SCREEN_WIDTH, SCREEN_HEIGHT, UI_HEIGHT, GRAPH_HEIGHT};
use crate::sir::{DiseaseState, DiseaseModel};

pub struct SimParams {
    pub perception_radius: f32,
    pub separation_radius: f32,
    pub separation_weight: f32,
    pub alignment_weight: f32,
    pub cohesion_weight: f32,
    pub max_speed: f32,
    pub max_force: f32,
    pub num_boids: usize,
    pub infection_radius: f32,
    pub infection_probability: f32,
    pub recovery_time: f32,
    pub incubation_time: f32,
    pub initial_infected: usize,
    pub model: DiseaseModel,
}

impl Default for SimParams {
    fn default() -> Self {
        Self {
            perception_radius: 50.0,
            separation_radius: 25.0,
            separation_weight: 1.5,
            alignment_weight: 1.0,
            cohesion_weight: 1.0,
            max_speed: 2.5,
            max_force: 0.1,
            num_boids: 150,
            infection_radius: 15.0,
            infection_probability: 0.02,
            recovery_time: 5.0,
            incubation_time: 3.0,
            initial_infected: 3,
            model: DiseaseModel::SIR,
        }
    }
}

pub fn initialize_boids(num_boids: usize, initial_infected: usize) -> Vec<Boid> {
    let mut boids = Vec::new();
    let grid_size = (num_boids as f32).sqrt().ceil() as usize;
    let cell_width = SCREEN_WIDTH / grid_size as f32;
    let cell_height = (SCREEN_HEIGHT - UI_HEIGHT - GRAPH_HEIGHT) / grid_size as f32;

    let mut count = 0;
    'outer: for i in 0..grid_size {
        for j in 0..grid_size {
            if count >= num_boids {
                break 'outer;
            }

            let x = (i as f32 + rand::gen_range(0.2, 0.8)) * cell_width;
            let y = UI_HEIGHT + (j as f32 + rand::gen_range(0.2, 0.8)) * cell_height;

            let disease_state = if count < initial_infected {
                DiseaseState::Infected
            } else {
                DiseaseState::Susceptible
            };

            boids.push(Boid::new(x, y, disease_state));
            count += 1;
        }
    }

    boids
}
