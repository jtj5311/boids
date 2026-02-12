# CLAUDE.md — leah/rust

This is the active Rust portion of a larger boids repository. The sibling `leah/jax/` directory is an older Python attempt, and `/rust_src/` at the repo root is a collaborator's version. **This directory is the one that matters.**

## What this project is

An interactive 2D boid flocking simulation with epidemiological disease modeling (SIR, SIS, SEIR). Built with macroquad + egui. Supports native and WASM builds.

## Workspace layout

This is a Cargo workspace with two crates:

- **boid_simulation/** — Core simulation. Outputs a native binary, an rlib (used by playground), and a cdylib (for WASM).
- **boid_playground/** — Experimental crate adding a user-controlled "My Boid" with disease-affinity flocking behavior. Depends on boid_simulation as a path dependency.

```
boid_simulation/src/
  main.rs / lib.rs   — Game loop entry points (native / WASM)
  boid.rs            — Boid struct, flocking rules (separation, alignment, cohesion)
  sir.rs             — DiseaseState enum, DiseaseModel enum, infection logic
  simulation.rs      — SimParams struct, boid initialization
  spatial.rs         — SpatialGrid for O(1) neighbor lookups
  visualization.rs   — PopulationHistory line graph (S/E/I/R over time)
  ui.rs              — egui parameter panel, keyboard toggle state
  constants.rs       — Screen dimensions, graph sizing

boid_playground/src/
  main.rs            — Playground game loop (regular boids + MyBoid)
  my_boid.rs         — MyBoid struct with per-disease-state affinity forces
  my_boid_ui.rs      — egui panel for MyBoid parameters
```

## Build and run

```bash
# From workspace root (leah/rust/)
cargo build --release          # Build both crates

# Run the main simulation
cargo run --release -p boid_simulation

# Run the playground (MyBoid experiment)
cargo run --release -p boid_playground
```

WASM builds target `wasm32-unknown-unknown` (see `boid_simulation/.cargo/config.toml`).

## Key types

| Type | Location | Role |
|------|----------|------|
| `Boid` | boid.rs | Autonomous agent with position, velocity, disease state |
| `MyBoid` | my_boid.rs | Special boid with affinity params per disease state |
| `DiseaseState` | sir.rs | Susceptible, Exposed, Infected, Recovered |
| `DiseaseModel` | sir.rs | SIR, SIS, SEIR — switchable at runtime |
| `SimParams` | simulation.rs | All tunable flocking + disease parameters |
| `SpatialGrid` | spatial.rs | Cell-based spatial hash for neighbor queries |
| `PopulationHistory` | visualization.rs | Rolling time-series of disease state counts |

## Key concepts

- **Flocking**: Three forces — separation, alignment, cohesion — each with configurable weight and radius. Toroidal screen wrapping.
- **Disease models**: SIR (immune after recovery), SIS (reinfectable), SEIR (adds exposed/incubation stage). Infection spreads spatially via `infection_radius` and `infection_probability`.
- **Spatial grid**: Avoids O(n^2) pairwise checks. Used for both flocking neighbor queries and infection spread.
- **MyBoid affinity**: Per-disease-state float (-3 to +3). Positive attracts toward boids in that state, negative repels.

## Keyboard controls (runtime)

- **Enter** — Restart simulation
- **P** — Toggle parameter panel
- **G** — Toggle population graph
- **M** — Toggle MyBoid panel (playground only)

## main.rs vs lib.rs

`main.rs` and `lib.rs` in boid_simulation both contain the game loop, but they are **not** kept in sync and serve different purposes:

- **`main.rs`** — Native binary entry point. Fixed window size, private modules. This is the **development version** where new features land first.
- **`lib.rs`** — Serves two roles: (1) the WASM entry point for the portfolio website, and (2) the public library (`pub mod`) that `boid_playground` imports from. Has `window_resizable: true` and `high_dpi: true` for web embedding.

**`lib.rs` is intentionally behind `main.rs`.** Changes are developed natively via `main.rs` and selectively promoted to `lib.rs` when ready for the website. Do not assume they should match — check with the user before syncing them.

## Other notes

- The release profile is optimized for WASM size (`opt-level = "z"`, LTO, strip). Native debug builds use default settings.
- Planning docs live in `todos/`. Archived plans are in `todos/old/`.
