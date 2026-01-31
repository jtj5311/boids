use macroquad::prelude::rand;
use crate::boid::Boid;
use crate::simulation::SimParams;
use crate::spatial::SpatialGrid;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum DiseaseState {
    Susceptible,
    Exposed,
    Infected,
    Recovered,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum DiseaseModel {
    SIR,
    SIS,
    SEIR,
}

pub fn process_infections(boids: &mut [Boid], params: &SimParams, spatial_grid: &SpatialGrid) {
    let mut new_infections = Vec::new();

    for i in 0..boids.len() {
        if boids[i].disease_state == DiseaseState::Infected {
            // Only check nearby boids using spatial grid
            let nearby_indices = spatial_grid.query_nearby_indices(
                boids[i].position,
                params.infection_radius
            );

            for j in nearby_indices {
                if i != j && boids[j].disease_state == DiseaseState::Susceptible {
                    let dist = (boids[i].position - boids[j].position).length();
                    if dist < params.infection_radius {
                        if rand::gen_range(0.0, 1.0) < params.infection_probability {
                            new_infections.push(j);
                        }
                    }
                }
            }
        }
    }

    for &idx in &new_infections {
        // For SEIR, newly infected go to Exposed first
        // For SIR and SIS, they go directly to Infected
        boids[idx].disease_state = match params.model {
            DiseaseModel::SEIR => DiseaseState::Exposed,
            DiseaseModel::SIR | DiseaseModel::SIS => DiseaseState::Infected,
        };
        // Reset the timer when changing disease state
        boids[idx].state_timer = 0.0;
    }
}

pub fn count_disease_states(boids: &[Boid]) -> (usize, usize, usize, usize) {
    let mut s = 0;
    let mut e = 0;
    let mut i = 0;
    let mut r = 0;

    for boid in boids {
        match boid.disease_state {
            DiseaseState::Susceptible => s += 1,
            DiseaseState::Exposed => e += 1,
            DiseaseState::Infected => i += 1,
            DiseaseState::Recovered => r += 1,
        }
    }

    (s, e, i, r)
}
