use macroquad::prelude::*;

mod constants;
mod sir;
mod boid;
mod simulation;
mod visualization;
mod spatial;
mod ui;

use constants::*;
use sir::{count_disease_states, process_infections, DiseaseModel};
use simulation::{SimParams, initialize_boids};
use visualization::PopulationHistory;
use spatial::SpatialGrid;
use ui::{UIState, render_parameter_panel, render_graph_toggle, render_collapsed_params_button};

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
    let mut spatial_grid = SpatialGrid::new(50.0); // Cell size based on perception radius
    let mut history = PopulationHistory::new();
    let mut ui_state = UIState::default();
    let mut frame_counter = 0;

    loop {
        clear_background(BLACK);
        let dt = get_frame_time();

        // Render UI and get controls
        let mut should_restart = false;
        let mut boid_count_changed = false;
        let mut model_changed = false;

        // Handle keyboard shortcuts
        if is_key_pressed(KeyCode::P) {
            ui_state.params_collapsed = !ui_state.params_collapsed;
        }
        if is_key_pressed(KeyCode::G) {
            ui_state.show_graph = !ui_state.show_graph;
        }

        let graph_x = SCREEN_WIDTH - GRAPH_WIDTH - 10.0;
        let graph_y = SCREEN_HEIGHT - GRAPH_HEIGHT - 10.0;

        egui_macroquad::ui(|egui_ctx| {
            render_graph_toggle(egui_ctx, &mut ui_state, graph_x, graph_y);
            let controls = render_parameter_panel(egui_ctx, &mut params, &mut ui_state);
            render_collapsed_params_button(egui_ctx, &mut ui_state);
            should_restart = controls.should_restart;
            boid_count_changed = controls.boid_count_changed;
            model_changed = controls.model_changed;
        });

        if is_key_pressed(KeyCode::Enter) || should_restart || boid_count_changed || model_changed {
            boids = initialize_boids(params.num_boids, params.initial_infected);
            history.clear();
            frame_counter = 0;
        }

        // Build spatial grid for efficient neighbor queries
        spatial_grid.clear();
        for (i, boid) in boids.iter().enumerate() {
            spatial_grid.insert(i, boid.position);
        }

        process_infections(&mut boids, &params, &spatial_grid);

        // Update each boid using spatial queries for neighbors
        for i in 0..boids.len() {
            let neighbors = spatial_grid.query_nearby(
                boids[i].position,
                params.perception_radius,
                &boids
            );
            boids[i].update(&neighbors, &params);
            boids[i].update_disease_state(&params, dt);
        }

        for boid in &boids {
            boid.draw();
        }

        frame_counter += 1;
        if frame_counter % 10 == 0 {
            let (s, e, i, r) = count_disease_states(&boids);
            history.add(s, e, i, r);
        }

        // Only draw graph if visible
        if ui_state.show_graph {
            history.draw(
                SCREEN_WIDTH - GRAPH_WIDTH - 10.0,
                SCREEN_HEIGHT - GRAPH_HEIGHT - 10.0,
                params.num_boids as f32,
                params.model,
            );
        }

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
