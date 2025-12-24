use egui_macroquad::egui;
use crate::simulation::SimParams;
use crate::sir::DiseaseModel;
use crate::constants::SCREEN_WIDTH;

pub struct UIState {
    pub show_graph: bool,
    pub params_collapsed: bool,
}

impl Default for UIState {
    fn default() -> Self {
        Self {
            show_graph: true,
            params_collapsed: false,
        }
    }
}

pub struct UIControls {
    pub should_restart: bool,
    pub boid_count_changed: bool,
    pub model_changed: bool,
}

impl Default for UIControls {
    fn default() -> Self {
        Self {
            should_restart: false,
            boid_count_changed: false,
            model_changed: false,
        }
    }
}

pub fn render_parameter_panel(
    egui_ctx: &egui::Context,
    params: &mut SimParams,
    ui_state: &mut UIState,
) -> UIControls {
    let mut controls = UIControls::default();

    // Only show if not collapsed
    if ui_state.params_collapsed {
        return controls;
    }

    egui::Window::new("##params")
        .title_bar(false)
        .default_pos(egui::pos2(10.0, 10.0))
        .default_width(SCREEN_WIDTH - 20.0)
        .resizable(false)
        .show(egui_ctx, |ui| {
            // Custom title bar with collapse button
            ui.horizontal(|ui| {
                ui.heading("Parameters");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("X [p]").clicked() {
                        ui_state.params_collapsed = true;
                    }
                });
            });

            ui.separator();
            // Boid Parameters Section with grey background
            egui::Frame::new()
                .fill(egui::Color32::from_rgb(60, 60, 60))
                .inner_margin(egui::Margin::same(8))
                .corner_radius(4.0)
                .show(ui, |ui| {
                    // Apply custom style for sliders in this section
                    let mut style = (*ui.ctx().style()).clone();
                    // Darken the slider rail background
                    style.visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(30, 30, 30);
                    style.visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(40, 40, 40);
                    style.visuals.widgets.active.bg_fill = egui::Color32::from_rgb(50, 50, 50);
                    ui.ctx().set_style(style);

                    ui.heading("Boid Parameters");
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.label("Number of Boids");
                            let old_count = params.num_boids;
                            ui.add(egui::Slider::new(&mut params.num_boids, 10..=3000));
                            if params.num_boids != old_count {
                                controls.boid_count_changed = true;
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
                                controls.model_changed = true;
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
                                controls.should_restart = true;
                            }
                        });
                    });
                });
        });

    controls
}

pub fn render_collapsed_params_button(
    egui_ctx: &egui::Context,
    ui_state: &mut UIState,
) {
    if ui_state.params_collapsed {
        egui::Window::new("##collapsed_params")
            .title_bar(false)
            .fixed_pos(egui::pos2(10.0, 10.0))
            .fixed_size(egui::vec2(65.0, 40.0))
            .frame(egui::Frame::new()
                .fill(egui::Color32::from_rgb(60, 60, 60))
                .corner_radius(4.0))
            .resizable(false)
            .show(egui_ctx, |ui| {
                if ui.button("≡ [p]").clicked() {
                    ui_state.params_collapsed = false;
                }
            });
    }
}

pub fn render_graph_toggle(
    egui_ctx: &egui::Context,
    ui_state: &mut UIState,
    graph_x: f32,
    graph_y: f32,
) {
    let button_text = if ui_state.show_graph { "X [g]" } else { "≡ [g]" };

    // Position at top of graph when shown, bottom-right corner when hidden
    let (pos_x, pos_y) = if ui_state.show_graph {
        (graph_x + 340.0, graph_y + 5.0)
    } else {
        (graph_x + 340.0, graph_y + 115.0) // Bottom right of screen
    };

    egui::Window::new("##graph_toggle")
        .title_bar(false)
        .fixed_pos(egui::pos2(pos_x, pos_y))
        .fixed_size(egui::vec2(55.0, 30.0))
        .frame(egui::Frame::new()
            .fill(egui::Color32::from_rgba_unmultiplied(40, 40, 40, 200))
            .corner_radius(4.0))
        .resizable(false)
        .show(egui_ctx, |ui| {
            if ui.button(button_text).clicked() {
                ui_state.show_graph = !ui_state.show_graph;
            }
        });
}
