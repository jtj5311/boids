use macroquad::prelude::*;
use macroquad::prelude::rand;
use crate::constants::{SCREEN_WIDTH, SCREEN_HEIGHT, UI_HEIGHT};
use crate::simulation::SimParams;
use crate::sir::SIRState;

#[derive(Clone)]
pub struct Boid {
    pub position: Vec2,
    pub velocity: Vec2,
    pub sir_state: SIRState,
    pub infection_timer: f32,
}

impl Boid {
    pub fn new(x: f32, y: f32, sir_state: SIRState) -> Self {
        let angle = rand::gen_range(0.0, std::f32::consts::TAU);
        let speed = rand::gen_range(1.5, 2.5);
        Self {
            position: vec2(x, y),
            velocity: vec2(angle.cos() * speed, angle.sin() * speed),
            sir_state,
            infection_timer: 0.0,
        }
    }

    pub fn update(&mut self, neighbors: &[(Vec2, Vec2)], params: &SimParams) {
        let mut separation = vec2(0.0, 0.0);
        let mut alignment = vec2(0.0, 0.0);
        let mut cohesion = vec2(0.0, 0.0);

        let mut separation_count = 0;
        let mut alignment_count = 0;
        let mut cohesion_count = 0;

        for &(other_pos, other_vel) in neighbors {
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

        let mut acceleration = vec2(0.0, 0.0);
        acceleration += separation * params.separation_weight;
        acceleration += alignment * params.alignment_weight;
        acceleration += cohesion * params.cohesion_weight;

        self.velocity += acceleration;
        self.velocity = limit_vec(self.velocity, params.max_speed);
        self.position += self.velocity;

        if self.position.x < 0.0 {
            self.position.x += SCREEN_WIDTH;
        }
        if self.position.x > SCREEN_WIDTH {
            self.position.x -= SCREEN_WIDTH;
        }
        if self.position.y < UI_HEIGHT {
            self.position.y += SCREEN_HEIGHT - UI_HEIGHT;
        }
        if self.position.y > SCREEN_HEIGHT {
            self.position.y -= SCREEN_HEIGHT - UI_HEIGHT;
        }
    }

    pub fn update_sir(&mut self, params: &SimParams, dt: f32) {
        match self.sir_state {
            SIRState::Infected => {
                self.infection_timer += dt;
                if self.infection_timer >= params.recovery_time {
                    self.sir_state = SIRState::Recovered;
                    self.infection_timer = 0.0;
                }
            }
            _ => {}
        }
    }

    pub fn draw(&self) {
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

        let color = match self.sir_state {
            SIRState::Susceptible => WHITE,
            SIRState::Infected => RED,
            SIRState::Recovered => BLUE,
        };

        draw_triangle(p1, p2, p3, color);
    }
}

fn limit_vec(v: Vec2, max: f32) -> Vec2 {
    if v.length() > max {
        v.normalize() * max
    } else {
        v
    }
}
