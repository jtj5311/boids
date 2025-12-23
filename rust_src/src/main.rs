use macroquad::prelude::*;
use std::f32::consts::PI;

mod sim;

use sim::{HealthState, SimConfig, Simulation, SirCounts, Vec2f};

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
        draw_text(
            self.label,
            self.center.x - self.radius,
            label_y,
            16.0,
            needle,
        );
        let value_text = format!("{:.2}", self.value);
        draw_text(
            &value_text,
            self.center.x - self.radius,
            label_y + 16.0,
            14.0,
            ring,
        );
    }
}

struct SirGraph {
    history: Vec<SirCounts>,
    max_len: usize,
}

impl SirGraph {
    fn new(max_len: usize) -> Self {
        Self {
            history: Vec::with_capacity(max_len),
            max_len,
        }
    }

    fn push(&mut self, counts: SirCounts) {
        if self.history.len() == self.max_len {
            self.history.remove(0);
        }
        self.history.push(counts);
    }

    fn draw(&self, origin: Vec2f, size: Vec2f, total: usize) {
        if self.history.len() < 2 || total == 0 {
            return;
        }
        let total_f = total as f32;
        let mut prev_s = self.point(0, origin, size, total_f, |c| c.susceptible as f32);
        let mut prev_i = self.point(0, origin, size, total_f, |c| c.infected as f32);
        let mut prev_r = self.point(0, origin, size, total_f, |c| c.recovered as f32);
        for idx in 1..self.history.len() {
            let cur_s = self.point(idx, origin, size, total_f, |c| c.susceptible as f32);
            let cur_i = self.point(idx, origin, size, total_f, |c| c.infected as f32);
            let cur_r = self.point(idx, origin, size, total_f, |c| c.recovered as f32);
            draw_line(
                prev_s.x,
                prev_s.y,
                cur_s.x,
                cur_s.y,
                2.0,
                Color::from_rgba(200, 220, 255, 255),
            );
            draw_line(
                prev_i.x,
                prev_i.y,
                cur_i.x,
                cur_i.y,
                2.0,
                Color::from_rgba(255, 90, 90, 255),
            );
            draw_line(
                prev_r.x,
                prev_r.y,
                cur_r.x,
                cur_r.y,
                2.0,
                Color::from_rgba(120, 220, 140, 255),
            );
            prev_s = cur_s;
            prev_i = cur_i;
            prev_r = cur_r;
        }
    }

    fn point(
        &self,
        idx: usize,
        origin: Vec2f,
        size: Vec2f,
        total: f32,
        f: impl Fn(&SirCounts) -> f32,
    ) -> Vec2f {
        let t = idx as f32 / (self.max_len.saturating_sub(1).max(1) as f32);
        let x = origin.x + t * size.x;
        let v = f(&self.history[idx]) / total;
        let y = origin.y + size.y - v * size.y;
        Vec2f::new(x, y)
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
        infection_radius: 18.0,
        infection_beta: 1.2,
        infectious_period: 6.0,
        initial_infected: 8,
    };
    let mut seed = 1337u32;
    let mut sim = Simulation::new(2400, cfg, seed);
    let mut knobs = vec![
        Knob::new("N Radius", 60.0, 20.0, 140.0, Vec2f::new(70.0, 70.0), 28.0),
        Knob::new("S Radius", 22.0, 5.0, 80.0, Vec2f::new(150.0, 70.0), 28.0),
        Knob::new("Max Spd", 160.0, 40.0, 320.0, Vec2f::new(230.0, 70.0), 28.0),
        Knob::new("Max F", 80.0, 10.0, 200.0, Vec2f::new(310.0, 70.0), 28.0),
        Knob::new("Inf R", 18.0, 4.0, 60.0, Vec2f::new(70.0, 160.0), 28.0),
        Knob::new("Beta", 1.2, 0.0, 5.0, Vec2f::new(150.0, 160.0), 28.0),
        Knob::new("Inf T", 6.0, 1.0, 20.0, Vec2f::new(230.0, 160.0), 28.0),
    ];

    let mut graph = SirGraph::new(360);

    loop {
        let dt = get_frame_time().min(0.05);
        sim.set_world_size(Vec2f::new(screen_width(), screen_height()));

        for knob in &mut knobs {
            knob.update();
        }

        let neighbor_radius = knobs[0].value;
        let separation_radius = knobs[1].value.min(neighbor_radius);
        let max_speed = knobs[2].value;
        let max_force = knobs[3].value;
        let infection_radius = knobs[4].value;
        let infection_beta = knobs[5].value;
        let infectious_period = knobs[6].value;

        if is_key_pressed(KeyCode::Enter) {
            let cfg = SimConfig {
                world_size: Vec2f::new(screen_width(), screen_height()),
                max_speed,
                max_force,
                neighbor_radius,
                separation_radius,
                infection_radius,
                infection_beta,
                infectious_period,
                initial_infected: 8,
            };
            seed = seed.wrapping_add(1);
            sim = Simulation::new(2400, cfg, seed);
            graph = SirGraph::new(360);
        }

        sim.set_motion_params(neighbor_radius, separation_radius, max_speed, max_force);
        sim.set_infection_params(infection_radius, infection_beta, infectious_period);
        sim.step(dt);
        let counts = sim.counts();
        graph.push(counts);

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

        draw_rectangle(16.0, 16.0, 340.0, 210.0, Color::from_rgba(10, 12, 18, 180));
        draw_rectangle_lines(
            16.0,
            16.0,
            340.0,
            210.0,
            1.0,
            Color::from_rgba(40, 60, 80, 200),
        );
        for knob in &knobs {
            knob.draw();
        }

        let graph_origin = Vec2f::new(380.0, 24.0);
        let graph_size = Vec2f::new(300.0, 120.0);
        draw_rectangle(
            graph_origin.x - 8.0,
            graph_origin.y - 8.0,
            graph_size.x + 16.0,
            graph_size.y + 16.0,
            Color::from_rgba(10, 12, 18, 180),
        );
        draw_rectangle_lines(
            graph_origin.x - 8.0,
            graph_origin.y - 8.0,
            graph_size.x + 16.0,
            graph_size.y + 16.0,
            1.0,
            Color::from_rgba(40, 60, 80, 200),
        );
        graph.draw(graph_origin, graph_size, sim.boids.len());

        next_frame().await;
    }
}
