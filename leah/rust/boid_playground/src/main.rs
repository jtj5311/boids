use macroquad::prelude::*;

mod my_boid;
mod my_boid_ui;

use boid_simulation::constants::*;
use boid_simulation::sir::{count_disease_states, process_infections, DiseaseModel, DiseaseState};
use boid_simulation::simulation::{SimParams, initialize_boids};
use boid_simulation::visualization::PopulationHistory;
use boid_simulation::spatial::SpatialGrid;
use boid_simulation::ui::{UIState, render_parameter_panel, render_graph_toggle, render_collapsed_params_button};

use my_boid::{MyBoid, MyBoidParams};
use my_boid_ui::{MyBoidUIState, render_my_boid_panel, render_collapsed_my_boid_button};

fn window_conf() -> Conf {
    Conf {
        window_title: "Boid Playground".to_owned(),
        window_width: SCREEN_WIDTH as i32,
        window_height: SCREEN_HEIGHT as i32,
        window_resizable: false,
        ..Default::default()
    }
}

/// Draw a regular boid with reduced alpha so My Boid stands out.
fn draw_boid_dimmed(boid: &boid_simulation::boid::Boid) {
    let angle = boid.velocity.y.atan2(boid.velocity.x);
    let size = 8.0;

    let p1 = vec2(
        boid.position.x + angle.cos() * size,
        boid.position.y + angle.sin() * size,
    );
    let p2 = vec2(
        boid.position.x + (angle + 2.5).cos() * size * 0.5,
        boid.position.y + (angle + 2.5).sin() * size * 0.5,
    );
    let p3 = vec2(
        boid.position.x + (angle - 2.5).cos() * size * 0.5,
        boid.position.y + (angle - 2.5).sin() * size * 0.5,
    );

    let (r, g, b) = match boid.disease_state {
        DiseaseState::Susceptible => (255, 255, 255),
        DiseaseState::Exposed => (255, 200, 0),
        DiseaseState::Infected => (255, 0, 0),
        DiseaseState::Recovered => (0, 0, 255),
    };

    draw_triangle(p1, p2, p3, Color::from_rgba(r, g, b, 130));
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut params = SimParams::default();
    let mut boids = initialize_boids(params.num_boids, params.initial_infected);
    let mut spatial_grid = SpatialGrid::new(50.0);
    let mut history = PopulationHistory::new();
    let mut ui_state = UIState::default();
    let mut frame_counter = 0;

    let mut my_boid = MyBoid::new();
    let mut my_boid_params = MyBoidParams::default();
    let mut my_boid_ui_state = MyBoidUIState::default();

    loop {
        clear_background(BLACK);
        let dt = get_frame_time();

        let mut should_restart = false;
        let mut boid_count_changed = false;
        let mut model_changed = false;

        // Keyboard shortcuts
        if is_key_pressed(KeyCode::P) {
            ui_state.params_collapsed = !ui_state.params_collapsed;
        }
        if is_key_pressed(KeyCode::G) {
            ui_state.show_graph = !ui_state.show_graph;
        }
        if is_key_pressed(KeyCode::M) {
            my_boid_ui_state.collapsed = !my_boid_ui_state.collapsed;
        }

        let graph_x = SCREEN_WIDTH - GRAPH_WIDTH - 10.0;
        let graph_y = SCREEN_HEIGHT - GRAPH_HEIGHT - 10.0;

        egui_macroquad::ui(|egui_ctx| {
            render_graph_toggle(egui_ctx, &mut ui_state, graph_x, graph_y);
            let controls = render_parameter_panel(egui_ctx, &mut params, &mut ui_state);
            render_collapsed_params_button(egui_ctx, &mut ui_state);
            render_my_boid_panel(egui_ctx, &mut my_boid_params, &mut my_boid_ui_state);
            render_collapsed_my_boid_button(egui_ctx, &mut my_boid_ui_state);
            should_restart = controls.should_restart;
            boid_count_changed = controls.boid_count_changed;
            model_changed = controls.model_changed;
        });

        if is_key_pressed(KeyCode::Enter) || should_restart || boid_count_changed || model_changed {
            boids = initialize_boids(params.num_boids, params.initial_infected);
            my_boid = MyBoid::new();
            history.clear();
            frame_counter = 0;
        }

        // Build spatial grid
        spatial_grid.clear();
        for (i, boid) in boids.iter().enumerate() {
            spatial_grid.insert(i, boid.position);
        }

        process_infections(&mut boids, &params, &spatial_grid);

        // My Boid disease: catch from / spread to regular boids
        my_boid.process_infection(&mut boids, &spatial_grid, &params);
        my_boid.update_disease_state(&params, dt);

        // Update regular boids
        for i in 0..boids.len() {
            let neighbors = spatial_grid.query_nearby(
                boids[i].position,
                params.perception_radius,
                &boids,
            );
            boids[i].update(&neighbors, &params);
            boids[i].update_disease_state(&params, dt);
        }

        // Update My Boid flocking
        my_boid.update(&boids, &spatial_grid, &my_boid_params);

        // Draw regular boids (dimmed)
        for boid in &boids {
            draw_boid_dimmed(boid);
        }

        // Draw My Boid (bright, with circle)
        my_boid.draw();

        // Population tracking
        frame_counter += 1;
        if frame_counter % 10 == 0 {
            let (s, e, i, r) = count_disease_states(&boids);
            history.add(s, e, i, r);
        }

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
