use macroquad::prelude::rand;
use crate::boid::Boid;
use crate::simulation::SimParams;

#[derive(Clone, Copy, PartialEq)]
pub enum SIRState {
    Susceptible,
    Infected,
    Recovered,
}

pub fn process_infections(boids: &mut [Boid], params: &SimParams) {
    let mut new_infections = Vec::new();

    for i in 0..boids.len() {
        if boids[i].sir_state == SIRState::Infected {
            for j in 0..boids.len() {
                if i != j && boids[j].sir_state == SIRState::Susceptible {
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
        boids[idx].sir_state = SIRState::Infected;
    }
}

pub fn count_sir(boids: &[Boid]) -> (usize, usize, usize) {
    let mut s = 0;
    let mut i = 0;
    let mut r = 0;

    for boid in boids {
        match boid.sir_state {
            SIRState::Susceptible => s += 1,
            SIRState::Infected => i += 1,
            SIRState::Recovered => r += 1,
        }
    }

    (s, i, r)
}
