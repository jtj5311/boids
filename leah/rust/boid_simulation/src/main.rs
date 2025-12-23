use macroquad::prelude::*;
use egui_macroquad::egui;
use std::collections::VecDeque;

const SCREEN_WIDTH: f32 = 1200.0;
const SCREEN_HEIGHT: f32 = 800.0;
const UI_HEIGHT: f32 = 160.0;
const GRAPH_HEIGHT: f32 = 150.0;
const GRAPH_WIDTH: f32 = 400.0;
const GRAPH_HISTORY: usize = 300;

#[derive(Clone, Copy, PartialEq)]
enum SIRState {
    Susceptible,
    Infected,
    Recovered,
}

#[derive(Clone)]
struct Boid {
    position: Vec2,
    velocity: Vec2,
    sir_state: SIRState,
    infection_timer: f32,
}

impl Boid {
    fn new(x: f32, y: f32, sir_state: SIRState) -> Self {
        let angle = rand::gen_range(0.0, std::f32::consts::TAU);
        let speed = rand::gen_range(1.5, 2.5);
        Self {
            position: vec2(x, y),
            velocity: vec2(angle.cos() * speed, angle.sin() * speed),
            sir_state,
            infection_timer: 0.0,
        }
    }

    fn update(&mut self, neighbors: &[(Vec2, Vec2)], params: &SimParams) {
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

    fn update_sir(&mut self, params: &SimParams, dt: f32) {
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

    fn draw(&self) {
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

struct SimParams {
    perception_radius: f32,
    separation_radius: f32,
    separation_weight: f32,
    alignment_weight: f32,
    cohesion_weight: f32,
    max_speed: f32,
    max_force: f32,
    num_boids: usize,
    infection_radius: f32,
    infection_probability: f32,
    recovery_time: f32,
    initial_infected: usize,
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
            initial_infected: 3,
        }
    }
}

struct PopulationHistory {
    susceptible: VecDeque<f32>,
    infected: VecDeque<f32>,
    recovered: VecDeque<f32>,
}

impl PopulationHistory {
    fn new() -> Self {
        Self {
            susceptible: VecDeque::new(),
            infected: VecDeque::new(),
            recovered: VecDeque::new(),
        }
    }

    fn add(&mut self, s: usize, i: usize, r: usize) {
        self.susceptible.push_back(s as f32);
        self.infected.push_back(i as f32);
        self.recovered.push_back(r as f32);

        if self.susceptible.len() > GRAPH_HISTORY {
            self.susceptible.pop_front();
            self.infected.pop_front();
            self.recovered.pop_front();
        }
    }

    fn clear(&mut self) {
        self.susceptible.clear();
        self.infected.clear();
        self.recovered.clear();
    }

    fn draw(&self, x: f32, y: f32, total_boids: f32) {
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

fn limit_vec(v: Vec2, max: f32) -> Vec2 {
    if v.length() > max {
        v.normalize() * max
    } else {
        v
    }
}

fn initialize_boids(num_boids: usize, initial_infected: usize) -> Vec<Boid> {
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

            let sir_state = if count < initial_infected {
                SIRState::Infected
            } else {
                SIRState::Susceptible
            };

            boids.push(Boid::new(x, y, sir_state));
            count += 1;
        }
    }

    boids
}

fn process_infections(boids: &mut [Boid], params: &SimParams) {
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

fn count_sir(boids: &[Boid]) -> (usize, usize, usize) {
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

fn window_conf() -> Conf {
    Conf {
        window_title: "Boid Simulation with SIR Model - Press Enter to Restart".to_owned(),
        window_width: SCREEN_WIDTH as i32,
        window_height: SCREEN_HEIGHT as i32,
        window_resizable: false,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut params = SimParams::default();
    let mut boids = initialize_boids(params.num_boids, params.initial_infected);
    let mut neighbor_data = Vec::new();
    let mut history = PopulationHistory::new();
    let mut frame_counter = 0;

    loop {
        clear_background(BLACK);
        let dt = get_frame_time();

        let mut should_restart = false;
        let mut boid_count_changed = false;

        egui_macroquad::ui(|egui_ctx| {
            egui::Window::new("Simulation Parameters (Press Enter to Restart)")
                .fixed_pos(egui::pos2(10.0, 10.0))
                .fixed_size(egui::vec2(SCREEN_WIDTH - 20.0, 140.0))
                .collapsible(false)
                .show(egui_ctx, |ui| {
                    ui.heading("Boid Parameters");
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.label("Number of Boids");
                            let old_count = params.num_boids;
                            ui.add(egui::Slider::new(&mut params.num_boids, 10..=500));
                            if params.num_boids != old_count {
                                boid_count_changed = true;
                            }
                        });
                        ui.vertical(|ui| {
                            ui.label("Perception Radius");
                            ui.add(egui::Slider::new(&mut params.perception_radius, 10.0..=150.0));
                        });
                        ui.vertical(|ui| {
                            ui.label("Separation Radius");
                            ui.add(egui::Slider::new(&mut params.separation_radius, 5.0..=50.0));
                        });
                        ui.vertical(|ui| {
                            ui.label("Max Speed");
                            ui.add(egui::Slider::new(&mut params.max_speed, 0.5..=5.0));
                        });
                    });

                    ui.separator();
                    ui.heading("SIR Model Parameters");
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.label("Initial Infected");
                            ui.add(egui::Slider::new(&mut params.initial_infected, 1..=20));
                        });
                        ui.vertical(|ui| {
                            ui.label("Infection Radius");
                            ui.add(egui::Slider::new(&mut params.infection_radius, 5.0..=50.0));
                        });
                        ui.vertical(|ui| {
                            ui.label("Infection Probability");
                            ui.add(egui::Slider::new(&mut params.infection_probability, 0.001..=0.1).step_by(0.001));
                        });
                        ui.vertical(|ui| {
                            ui.label("Recovery Time (s)");
                            ui.add(egui::Slider::new(&mut params.recovery_time, 1.0..=20.0));
                        });
                        ui.vertical(|ui| {
                            ui.label("");
                            if ui.button("Restart").clicked() {
                                should_restart = true;
                            }
                        });
                    });
                });
        });

        if is_key_pressed(KeyCode::Enter) || should_restart || boid_count_changed {
            boids = initialize_boids(params.num_boids, params.initial_infected);
            history.clear();
            frame_counter = 0;
        }

        neighbor_data.clear();
        for boid in &boids {
            neighbor_data.push((boid.position, boid.velocity));
        }

        process_infections(&mut boids, &params);

        for boid in &mut boids {
            boid.update(&neighbor_data, &params);
            boid.update_sir(&params, dt);
        }

        for boid in &boids {
            boid.draw();
        }

        frame_counter += 1;
        if frame_counter % 10 == 0 {
            let (s, i, r) = count_sir(&boids);
            history.add(s, i, r);
        }

        history.draw(
            SCREEN_WIDTH - GRAPH_WIDTH - 10.0,
            SCREEN_HEIGHT - GRAPH_HEIGHT - 10.0,
            params.num_boids as f32,
        );

        let (s, i, r) = count_sir(&boids);
        draw_text(
            &format!("S: {} | I: {} | R: {}", s, i, r),
            20.0,
            SCREEN_HEIGHT - 20.0,
            24.0,
            WHITE,
        );

        egui_macroquad::draw();

        next_frame().await
    }
}
