use macroquad::prelude::*;
use macroquad::prelude::rand;
use boid_simulation::boid::Boid;
use boid_simulation::constants::{SCREEN_WIDTH, SCREEN_HEIGHT};
use boid_simulation::sir::{DiseaseState, DiseaseModel};
use boid_simulation::simulation::SimParams;
use boid_simulation::spatial::SpatialGrid;

pub struct MyBoidParams {
    pub perception_radius: f32,
    pub separation_radius: f32,
    pub separation_weight: f32,
    pub alignment_weight: f32,
    pub cohesion_weight: f32,
    pub max_speed: f32,
    pub max_force: f32,
    // Disease-state affinity: positive = attract, negative = repel
    pub affinity_susceptible: f32,
    pub affinity_exposed: f32,
    pub affinity_infected: f32,
    pub affinity_recovered: f32,
}

impl Default for MyBoidParams {
    fn default() -> Self {
        Self {
            perception_radius: 50.0,
            separation_radius: 25.0,
            separation_weight: 1.5,
            alignment_weight: 1.0,
            cohesion_weight: 1.0,
            max_speed: 2.5,
            max_force: 0.1,
            affinity_susceptible: 0.0,
            affinity_exposed: 0.0,
            affinity_infected: 0.0,
            affinity_recovered: 0.0,
        }
    }
}

pub struct MyBoid {
    pub position: Vec2,
    pub velocity: Vec2,
    pub disease_state: DiseaseState,
    pub state_timer: f32,
}

impl MyBoid {
    pub fn new() -> Self {
        let x = rand::gen_range(100.0, SCREEN_WIDTH - 100.0);
        let y = rand::gen_range(100.0, SCREEN_HEIGHT - 100.0);
        let angle = rand::gen_range(0.0, std::f32::consts::TAU);
        let speed = rand::gen_range(1.5, 2.5);
        Self {
            position: vec2(x, y),
            velocity: vec2(angle.cos() * speed, angle.sin() * speed),
            disease_state: DiseaseState::Susceptible,
            state_timer: 0.0,
        }
    }

    /// Check nearby regular boids for infection, and also infect nearby susceptible boids if we're infected.
    pub fn process_infection(&mut self, boids: &mut [Boid], spatial_grid: &SpatialGrid, params: &SimParams) {
        let nearby_indices = spatial_grid.query_nearby_indices(self.position, params.infection_radius);

        for idx in nearby_indices {
            let dist = (self.position - boids[idx].position).length();
            if dist < params.infection_radius {
                // We can catch it from infected boids
                if self.disease_state == DiseaseState::Susceptible
                    && boids[idx].disease_state == DiseaseState::Infected
                {
                    if rand::gen_range(0.0, 1.0) < params.infection_probability {
                        self.disease_state = match params.model {
                            DiseaseModel::SEIR => DiseaseState::Exposed,
                            DiseaseModel::SIR | DiseaseModel::SIS => DiseaseState::Infected,
                        };
                        self.state_timer = 0.0;
                    }
                }

                // We can spread it to susceptible boids
                if self.disease_state == DiseaseState::Infected
                    && boids[idx].disease_state == DiseaseState::Susceptible
                {
                    if rand::gen_range(0.0, 1.0) < params.infection_probability {
                        boids[idx].disease_state = match params.model {
                            DiseaseModel::SEIR => DiseaseState::Exposed,
                            DiseaseModel::SIR | DiseaseModel::SIS => DiseaseState::Infected,
                        };
                        boids[idx].state_timer = 0.0;
                    }
                }
            }
        }
    }

    /// Advance disease state timers (same logic as regular boids).
    pub fn update_disease_state(&mut self, params: &SimParams, dt: f32) {
        self.state_timer += dt;

        match params.model {
            DiseaseModel::SIR => {
                if self.disease_state == DiseaseState::Infected && self.state_timer >= params.recovery_time {
                    self.disease_state = DiseaseState::Recovered;
                    self.state_timer = 0.0;
                }
            }
            DiseaseModel::SIS => {
                if self.disease_state == DiseaseState::Infected && self.state_timer >= params.recovery_time {
                    self.disease_state = DiseaseState::Susceptible;
                    self.state_timer = 0.0;
                }
            }
            DiseaseModel::SEIR => {
                if self.disease_state == DiseaseState::Exposed && self.state_timer >= params.incubation_time {
                    self.disease_state = DiseaseState::Infected;
                    self.state_timer = 0.0;
                } else if self.disease_state == DiseaseState::Infected && self.state_timer >= params.recovery_time {
                    self.disease_state = DiseaseState::Recovered;
                    self.state_timer = 0.0;
                }
            }
        }
    }

    fn affinity_for(state: &DiseaseState, params: &MyBoidParams) -> f32 {
        match state {
            DiseaseState::Susceptible => params.affinity_susceptible,
            DiseaseState::Exposed => params.affinity_exposed,
            DiseaseState::Infected => params.affinity_infected,
            DiseaseState::Recovered => params.affinity_recovered,
        }
    }

    pub fn update(&mut self, boids: &[Boid], spatial_grid: &SpatialGrid, params: &MyBoidParams) {
        let nearby_indices = spatial_grid.query_nearby_indices(
            self.position,
            params.perception_radius,
        );

        let mut separation = vec2(0.0, 0.0);
        let mut alignment = vec2(0.0, 0.0);
        let mut cohesion = vec2(0.0, 0.0);
        let mut affinity_force = vec2(0.0, 0.0);

        let mut separation_count = 0;
        let mut alignment_count = 0;
        let mut cohesion_count = 0;
        let mut affinity_count = 0;

        for idx in nearby_indices {
            let other_pos = boids[idx].position;
            let other_vel = boids[idx].velocity;
            let diff = self.position - other_pos;
            let dist = diff.length();

            if dist > 0.1 && dist < params.perception_radius {
                if dist < params.separation_radius {
                    separation += diff / dist;
                    separation_count += 1;
                }

                alignment += other_vel;
                alignment_count += 1;

                cohesion += other_pos;
                cohesion_count += 1;

                // Affinity: positive attracts (steer toward), negative repels (steer away)
                let a = Self::affinity_for(&boids[idx].disease_state, params);
                if a.abs() > 0.001 {
                    // Direction toward the other boid, scaled by affinity
                    let toward = -diff.normalize_or_zero();
                    affinity_force += toward * a / dist.max(1.0);
                    affinity_count += 1;
                }
            }
        }

        if separation_count > 0 {
            separation /= separation_count as f32;
            separation = separation.normalize_or_zero() * params.max_speed - self.velocity;
            separation = limit_vec(separation, params.max_force);
        }

        if alignment_count > 0 {
            alignment /= alignment_count as f32;
            alignment = alignment.normalize_or_zero() * params.max_speed - self.velocity;
            alignment = limit_vec(alignment, params.max_force);
        }

        if cohesion_count > 0 {
            cohesion /= cohesion_count as f32;
            cohesion = (cohesion - self.position).normalize_or_zero() * params.max_speed - self.velocity;
            cohesion = limit_vec(cohesion, params.max_force);
        }

        if affinity_count > 0 {
            affinity_force /= affinity_count as f32;
            affinity_force = affinity_force.normalize_or_zero() * params.max_speed - self.velocity;
            affinity_force = limit_vec(affinity_force, params.max_force);
        }

        let mut acceleration = vec2(0.0, 0.0);
        acceleration += separation * params.separation_weight;
        acceleration += alignment * params.alignment_weight;
        acceleration += cohesion * params.cohesion_weight;
        acceleration += affinity_force;

        self.velocity += acceleration;
        self.velocity = limit_vec(self.velocity, params.max_speed);
        self.position += self.velocity;

        // Wrap around screen
        if self.position.x < 0.0 {
            self.position.x += SCREEN_WIDTH;
        }
        if self.position.x > SCREEN_WIDTH {
            self.position.x -= SCREEN_WIDTH;
        }
        if self.position.y < 0.0 {
            self.position.y += SCREEN_HEIGHT;
        }
        if self.position.y > SCREEN_HEIGHT {
            self.position.y -= SCREEN_HEIGHT;
        }
    }

    pub fn draw(&self) {
        let (r, g, b) = match self.disease_state {
            DiseaseState::Susceptible => (255, 255, 255),
            DiseaseState::Exposed => (255, 200, 0),
            DiseaseState::Infected => (255, 0, 0),
            DiseaseState::Recovered => (80, 130, 255),
        };

        // Bright glowing circle (outer, faint)
        draw_circle_lines(self.position.x, self.position.y, 24.0, 1.5,
            Color::from_rgba(r, g, b, 60));
        // Inner bright circle
        draw_circle_lines(self.position.x, self.position.y, 18.0, 2.0,
            Color::from_rgba(r, g, b, 180));

        // Draw triangle (same shape as regular boids, full brightness)
        let angle = self.velocity.y.atan2(self.velocity.x);
        let size = 8.0;

        let p1 = vec2(
            self.position.x + angle.cos() * size,
            self.position.y + angle.sin() * size,
        );
        let p2 = vec2(
            self.position.x + (angle + 2.5).cos() * size * 0.5,
            self.position.y + (angle + 2.5).sin() * size * 0.5,
        );
        let p3 = vec2(
            self.position.x + (angle - 2.5).cos() * size * 0.5,
            self.position.y + (angle - 2.5).sin() * size * 0.5,
        );

        draw_triangle(p1, p2, p3, Color::from_rgba(r, g, b, 255));
    }
}

fn limit_vec(v: Vec2, max: f32) -> Vec2 {
    if v.length() > max {
        v.normalize() * max
    } else {
        v
    }
}
