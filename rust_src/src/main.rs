use macroquad::prelude::*;
use std::f32::consts::PI;

mod sim;

use sim::{HealthState, SimConfig, Simulation, Vec2f};

struct Knob {
    label: &'static str,
    value: f32,
    min: f32,
    max: f32,
    center: Vec2f,
    radius: f32,
    dragging: bool,
}

impl Knob {
    fn new(
        label: &'static str,
        value: f32,
        min: f32,
        max: f32,
        center: Vec2f,
        radius: f32,
    ) -> Self {
        Self {
            label,
            value,
            min,
            max,
            center,
            radius,
            dragging: false,
        }
    }

    fn t(&self) -> f32 {
        if self.max == self.min {
            0.0
        } else {
            (self.value - self.min) / (self.max - self.min)
        }
    }

    fn angle_from_value(&self) -> f32 {
        let t = self.t().clamp(0.0, 1.0);
        let start = -3.0 * PI / 4.0;
        let end = 3.0 * PI / 4.0;
        start + t * (end - start)
    }

    fn update(&mut self) {
        let (mx, my) = mouse_position();
        let mouse = Vec2f::new(mx, my);
        let dist = mouse.sub(self.center).length();
        if is_mouse_button_pressed(MouseButton::Left) && dist <= self.radius {
            self.dragging = true;
        }
        if is_mouse_button_released(MouseButton::Left) {
            self.dragging = false;
        }
        if self.dragging && is_mouse_button_down(MouseButton::Left) {
            let angle = (mouse.y - self.center.y).atan2(mouse.x - self.center.x);
            let start = -3.0 * PI / 4.0;
            let end = 3.0 * PI / 4.0;
            let clamped = angle.clamp(start, end);
            let t = (clamped - start) / (end - start);
            self.value = self.min + t * (self.max - self.min);
        }
    }

    fn draw(&self) {
        let bg = Color::from_rgba(20, 24, 32, 255);
        let ring = Color::from_rgba(90, 110, 135, 255);
        let needle = Color::from_rgba(220, 240, 255, 255);
        draw_circle(self.center.x, self.center.y, self.radius, bg);
        draw_circle_lines(self.center.x, self.center.y, self.radius, 2.0, ring);
        let angle = self.angle_from_value();
        let dir = Vec2f::new(angle.cos(), angle.sin());
        let tip = self.center.add(dir.mul(self.radius * 0.75));
        draw_line(self.center.x, self.center.y, tip.x, tip.y, 2.5, needle);

        let label_y = self.center.y + self.radius + 12.0;
        draw_text(self.label, self.center.x - self.radius, label_y, 16.0, needle);
        let value_text = format!("{:.2}", self.value);
        draw_text(&value_text, self.center.x - self.radius, label_y + 16.0, 14.0, ring);
    }
}

#[macroquad::main("Boids")]
async fn main() {
    let cfg = SimConfig {
        world_size: Vec2f::new(screen_width(), screen_height()),
        max_speed: 160.0,
        max_force: 80.0,
        neighbor_radius: 60.0,
        separation_radius: 22.0,
        weight_align: 1.0,
        weight_cohesion: 0.8,
        weight_separation: 1.4,
        infection_radius: 18.0,
        infection_beta: 1.2,
        infectious_period: 6.0,
        initial_infected: 8,
    };
    let mut sim = Simulation::new(2400, cfg, 1337);
    let mut knobs = vec![
        Knob::new("Align", 1.0, 0.0, 3.0, Vec2f::new(70.0, 70.0), 28.0),
        Knob::new("Cohere", 0.8, 0.0, 3.0, Vec2f::new(150.0, 70.0), 28.0),
        Knob::new("Separate", 1.4, 0.0, 3.0, Vec2f::new(230.0, 70.0), 28.0),
        Knob::new("N Radius", 60.0, 20.0, 140.0, Vec2f::new(70.0, 160.0), 28.0),
        Knob::new("S Radius", 22.0, 5.0, 80.0, Vec2f::new(150.0, 160.0), 28.0),
        Knob::new("Max Spd", 160.0, 40.0, 320.0, Vec2f::new(230.0, 160.0), 28.0),
        Knob::new("Max F", 80.0, 10.0, 200.0, Vec2f::new(310.0, 160.0), 28.0),
        Knob::new("Inf R", 18.0, 4.0, 60.0, Vec2f::new(70.0, 250.0), 28.0),
        Knob::new("Beta", 1.2, 0.0, 5.0, Vec2f::new(150.0, 250.0), 28.0),
        Knob::new("Inf T", 6.0, 1.0, 20.0, Vec2f::new(230.0, 250.0), 28.0),
    ];

    loop {
        let dt = get_frame_time().min(0.05);
        sim.set_world_size(Vec2f::new(screen_width(), screen_height()));

        for knob in &mut knobs {
            knob.update();
        }

        let weight_align = knobs[0].value;
        let weight_cohesion = knobs[1].value;
        let weight_separation = knobs[2].value;
        let neighbor_radius = knobs[3].value;
        let separation_radius = knobs[4].value.min(neighbor_radius);
        let max_speed = knobs[5].value;
        let max_force = knobs[6].value;
        let infection_radius = knobs[7].value;
        let infection_beta = knobs[8].value;
        let infectious_period = knobs[9].value;

        sim.set_params(
            neighbor_radius,
            separation_radius,
            max_speed,
            max_force,
            weight_align,
            weight_cohesion,
            weight_separation,
        );
        sim.set_infection_params(infection_radius, infection_beta, infectious_period);
        sim.step(dt);

        clear_background(Color::from_rgba(8, 10, 14, 255));

        for boid in &sim.boids {
            let dir = boid.vel.normalize();
            let dir = if dir.length() > 0.0 {
                dir
            } else {
                Vec2f::new(1.0, 0.0)
            };
            let perp = Vec2f::new(-dir.y, dir.x);
            let tip = boid.pos.add(dir.mul(6.0));
            let left = boid.pos.sub(dir.mul(2.5)).add(perp.mul(3.0));
            let right = boid.pos.sub(dir.mul(2.5)).sub(perp.mul(3.0));

            let color = match boid.state {
                HealthState::Susceptible => Color::from_rgba(220, 240, 255, 255),
                HealthState::Infected => Color::from_rgba(255, 90, 90, 255),
                HealthState::Recovered => Color::from_rgba(120, 220, 140, 255),
            };

            draw_triangle(
                Vec2::new(tip.x, tip.y),
                Vec2::new(left.x, left.y),
                Vec2::new(right.x, right.y),
                color,
            );
        }

        draw_rectangle(16.0, 16.0, 340.0, 280.0, Color::from_rgba(10, 12, 18, 180));
        draw_rectangle_lines(
            16.0,
            16.0,
            340.0,
            280.0,
            1.0,
            Color::from_rgba(40, 60, 80, 200),
        );
        for knob in &knobs {
            knob.draw();
        }

        next_frame().await;
    }
}
