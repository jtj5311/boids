use egui_macroquad::egui;
use boid_simulation::sir::DiseaseModel;
use crate::my_boid::MyBoidParams;

pub struct MyBoidUIState {
    pub collapsed: bool,
}

impl Default for MyBoidUIState {
    fn default() -> Self {
        Self { collapsed: false }
    }
}

pub fn render_my_boid_panel(
    egui_ctx: &egui::Context,
    params: &mut MyBoidParams,
    ui_state: &mut MyBoidUIState,
    disease_model: DiseaseModel,
) {
    if ui_state.collapsed {
        return;
    }

    egui::Window::new("##my_boid")
        .title_bar(false)
        .default_pos(egui::pos2(10.0, 200.0))
        .default_width(380.0)
        .resizable(false)
        .show(egui_ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("My Boid");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("X [m]").clicked() {
                        ui_state.collapsed = true;
                    }
                });
            });

            ui.separator();

            egui::Frame::new()
                .fill(egui::Color32::from_rgb(40, 50, 80))
                .inner_margin(egui::Margin::same(8))
                .corner_radius(4.0)
                .show(ui, |ui| {
                    let mut style = (*ui.ctx().style()).clone();
                    style.visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(25, 30, 50);
                    style.visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(35, 40, 60);
                    style.visuals.widgets.active.bg_fill = egui::Color32::from_rgb(45, 50, 70);
                    ui.ctx().set_style(style);

                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.label("Perception Radius");
                            ui.add(egui::Slider::new(&mut params.perception_radius, 10.0..=150.0));
                        });
                        ui.vertical(|ui| {
                            ui.label("Separation Radius");
                            ui.add(egui::Slider::new(&mut params.separation_radius, 5.0..=50.0));
                        });
                    });

                    ui.add_space(4.0);

                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.label("Separation Weight");
                            ui.add(egui::Slider::new(&mut params.separation_weight, 0.0..=5.0));
                        });
                        ui.vertical(|ui| {
                            ui.label("Alignment Weight");
                            ui.add(egui::Slider::new(&mut params.alignment_weight, 0.0..=5.0));
                        });
                    });

                    ui.add_space(4.0);

                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.label("Cohesion Weight");
                            ui.add(egui::Slider::new(&mut params.cohesion_weight, 0.0..=5.0));
                        });
                        ui.vertical(|ui| {
                            ui.label("Max Speed");
                            ui.add(egui::Slider::new(&mut params.max_speed, 0.5..=5.0));
                        });
                    });

                    ui.add_space(4.0);

                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.label("Max Force");
                            ui.add(egui::Slider::new(&mut params.max_force, 0.01..=0.5));
                        });
                    });
                });

            ui.add_space(6.0);

            // Disease Affinity Section
            egui::Frame::new()
                .fill(egui::Color32::from_rgb(60, 45, 50))
                .inner_margin(egui::Margin::same(8))
                .corner_radius(4.0)
                .show(ui, |ui| {
                    let mut style = (*ui.ctx().style()).clone();
                    style.visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(35, 25, 30);
                    style.visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(45, 35, 40);
                    style.visuals.widgets.active.bg_fill = egui::Color32::from_rgb(55, 45, 50);
                    ui.ctx().set_style(style);

                    ui.heading("Disease Affinity");
                    ui.label("+ attract  /  - repel");

                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.colored_label(egui::Color32::WHITE, "Susceptible");
                            ui.add(egui::Slider::new(&mut params.affinity_susceptible, -3.0..=3.0));
                        });
                        if disease_model == DiseaseModel::SEIR {
                            ui.vertical(|ui| {
                                ui.colored_label(egui::Color32::from_rgb(255, 200, 0), "Exposed");
                                ui.add(egui::Slider::new(&mut params.affinity_exposed, -3.0..=3.0));
                            });
                        }
                    });

                    ui.add_space(4.0);

                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.colored_label(egui::Color32::from_rgb(255, 80, 80), "Infected");
                            ui.add(egui::Slider::new(&mut params.affinity_infected, -3.0..=3.0));
                        });
                        ui.vertical(|ui| {
                            ui.colored_label(egui::Color32::from_rgb(80, 130, 255), "Recovered");
                            ui.add(egui::Slider::new(&mut params.affinity_recovered, -3.0..=3.0));
                        });
                    });
                });
        });
}

pub fn render_collapsed_my_boid_button(
    egui_ctx: &egui::Context,
    ui_state: &mut MyBoidUIState,
) {
    if ui_state.collapsed {
        egui::Window::new("##collapsed_my_boid")
            .title_bar(false)
            .fixed_pos(egui::pos2(80.0, 10.0))
            .fixed_size(egui::vec2(75.0, 40.0))
            .frame(egui::Frame::new()
                .fill(egui::Color32::from_rgb(40, 50, 80))
                .corner_radius(4.0))
            .resizable(false)
            .show(egui_ctx, |ui| {
                if ui.button("My Boid [m]").clicked() {
                    ui_state.collapsed = false;
                }
            });
    }
}
