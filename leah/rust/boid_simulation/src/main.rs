use macroquad::prelude::*;
use egui_macroquad::egui;

const SCREEN_WIDTH: f32 = 1200.0;
const SCREEN_HEIGHT: f32 = 800.0;
const UI_HEIGHT: f32 = 120.0;

#[derive(Clone)]
struct Boid {
    position: Vec2,
    velocity: Vec2,
}

impl Boid {
    fn new(x: f32, y: f32) -> Self {
        let angle = rand::gen_range(0.0, std::f32::consts::TAU);
        let speed = rand::gen_range(1.5, 2.5);
        Self {
            position: vec2(x, y),
            velocity: vec2(angle.cos() * speed, angle.sin() * speed),
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

        draw_triangle(p1, p2, p3, WHITE);
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
        }
    }
}

fn limit_vec(v: Vec2, max: f32) -> Vec2 {
    if v.length() > max {
        v.normalize() * max
    } else {
        v
    }
}

fn initialize_boids(num_boids: usize) -> Vec<Boid> {
    let mut boids = Vec::new();
    let grid_size = (num_boids as f32).sqrt().ceil() as usize;
    let cell_width = SCREEN_WIDTH / grid_size as f32;
    let cell_height = (SCREEN_HEIGHT - UI_HEIGHT) / grid_size as f32;

    let mut count = 0;
    'outer: for i in 0..grid_size {
        for j in 0..grid_size {
            if count >= num_boids {
                break 'outer;
            }

            let x = (i as f32 + rand::gen_range(0.2, 0.8)) * cell_width;
            let y = UI_HEIGHT + (j as f32 + rand::gen_range(0.2, 0.8)) * cell_height;

            boids.push(Boid::new(x, y));
            count += 1;
        }
    }

    boids
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Boid Simulation - Press Enter to Restart".to_owned(),
        window_width: SCREEN_WIDTH as i32,
        window_height: SCREEN_HEIGHT as i32,
        window_resizable: false,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut params = SimParams::default();
    let mut boids = initialize_boids(params.num_boids);
    let mut neighbor_data = Vec::new();

    loop {
        clear_background(BLACK);

        let mut should_restart = false;
        let mut boid_count_changed = false;

        egui_macroquad::ui(|egui_ctx| {
            egui::Window::new("Boid Parameters (Press Enter to Restart)")
                .fixed_pos(egui::pos2(10.0, 10.0))
                .fixed_size(egui::vec2(SCREEN_WIDTH - 20.0, 100.0))
                .collapsible(false)
                .show(egui_ctx, |ui| {
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
                            ui.label("Separation Weight");
                            ui.add(egui::Slider::new(&mut params.separation_weight, 0.0..=3.0));
                        });
                    });
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.label("Alignment Weight");
                            ui.add(egui::Slider::new(&mut params.alignment_weight, 0.0..=3.0));
                        });
                        ui.vertical(|ui| {
                            ui.label("Cohesion Weight");
                            ui.add(egui::Slider::new(&mut params.cohesion_weight, 0.0..=3.0));
                        });
                        ui.vertical(|ui| {
                            ui.label("Max Speed");
                            ui.add(egui::Slider::new(&mut params.max_speed, 0.5..=5.0));
                        });
                        ui.vertical(|ui| {
                            ui.label("");
                            if ui.button("Restart (or press Enter)").clicked() {
                                should_restart = true;
                            }
                        });
                    });
                });
        });

        if is_key_pressed(KeyCode::Enter) || should_restart || boid_count_changed {
            boids = initialize_boids(params.num_boids);
        }

        neighbor_data.clear();
        for boid in &boids {
            neighbor_data.push((boid.position, boid.velocity));
        }

        for boid in &mut boids {
            boid.update(&neighbor_data, &params);
        }

        for boid in &boids {
            boid.draw();
        }

        egui_macroquad::draw();

        next_frame().await
    }
}
