use macroquad::prelude::*;
use egui_macroquad::egui;

mod constants;
mod sir;
mod boid;
mod simulation;
mod visualization;

use constants::*;
use sir::{count_disease_states, process_infections, DiseaseModel};
use simulation::{SimParams, initialize_boids};
use visualization::PopulationHistory;

fn window_conf() -> Conf {
    Conf {
        window_title: "Boid Simulation with Disease Models - Press Enter to Restart".to_owned(),
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
        let mut model_changed = false;

        egui_macroquad::ui(|egui_ctx| {
            egui::Window::new("Simulation Parameters (Press Enter to Restart)")
                .fixed_pos(egui::pos2(10.0, 10.0))
                .fixed_size(egui::vec2(SCREEN_WIDTH - 20.0, 140.0))
                .collapsible(false)
                .show(egui_ctx, |ui| {
                    // Boid Parameters Section with grey background
                    egui::Frame::new()
                        .fill(egui::Color32::from_rgb(60, 60, 60))
                        .inner_margin(egui::Margin::same(8))
                        .corner_radius(4.0)
                        .show(ui, |ui| {
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
                        });

                    ui.add_space(6.0);

                    // Disease Model Parameters Section with red background
                    egui::Frame::new()
                        .fill(egui::Color32::from_rgb(80, 40, 40))
                        .inner_margin(egui::Margin::same(8))
                        .corner_radius(4.0)
                        .show(ui, |ui| {
                            ui.heading("Disease Model Parameters");
                            ui.horizontal(|ui| {
                                ui.vertical(|ui| {
                                    ui.label("Model Type");
                                    let old_model = params.model;
                                    egui::ComboBox::from_id_salt("model_selector")
                                        .selected_text(format!("{:?}", params.model))
                                        .show_ui(ui, |ui| {
                                            ui.selectable_value(&mut params.model, DiseaseModel::SIR, "SIR");
                                            ui.selectable_value(&mut params.model, DiseaseModel::SIS, "SIS");
                                            ui.selectable_value(&mut params.model, DiseaseModel::SEIR, "SEIR");
                                        });
                                    if params.model != old_model {
                                        model_changed = true;
                                    }
                                });
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
                                if params.model == DiseaseModel::SEIR {
                                    ui.vertical(|ui| {
                                        ui.label("Incubation Time (s)");
                                        ui.add(egui::Slider::new(&mut params.incubation_time, 1.0..=20.0));
                                    });
                                }
                                ui.vertical(|ui| {
                                    ui.label("");
                                    if ui.button("Restart").clicked() {
                                        should_restart = true;
                                    }
                                });
                            });
                        });
                });
        });

        if is_key_pressed(KeyCode::Enter) || should_restart || boid_count_changed || model_changed {
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
            boid.update_disease_state(&params, dt);
        }

        for boid in &boids {
            boid.draw();
        }

        frame_counter += 1;
        if frame_counter % 10 == 0 {
            let (s, e, i, r) = count_disease_states(&boids);
            history.add(s, e, i, r);
        }

        history.draw(
            SCREEN_WIDTH - GRAPH_WIDTH - 10.0,
            SCREEN_HEIGHT - GRAPH_HEIGHT - 10.0,
            params.num_boids as f32,
            params.model,
        );

        let (s, e, i, r) = count_disease_states(&boids);
        let status_text = match params.model {
            DiseaseModel::SIR | DiseaseModel::SIS => {
                format!("S: {} | I: {} | R: {}", s, i, r)
            }
            DiseaseModel::SEIR => {
                format!("S: {} | E: {} | I: {} | R: {}", s, e, i, r)
            }
        };
        draw_text(
            &status_text,
            20.0,
            SCREEN_HEIGHT - 20.0,
            24.0,
            WHITE,
        );

        egui_macroquad::draw();

        next_frame().await
    }
}
