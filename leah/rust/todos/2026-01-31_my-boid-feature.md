# My Boid Feature - Planning Document

**Date:** 2026-01-31
**Branch:** leah-branch

---

## Part 1: Project Structure

### Current State

Right now everything lives in a single crate:

```
leah/rust/
  boid_simulation/
    Cargo.toml          # builds both cdylib (wasm) and rlib + binary
    build_wasm.sh       # wasm pipeline -> portfolio
    src/
      main.rs           # native entry point
      lib.rs            # wasm entry point (nearly identical to main.rs)
      boid.rs, simulation.rs, sir.rs, spatial.rs, ui.rs, ...
```

The WASM build (`build_wasm.sh`) targets `boid_simulation` by name, compiles
the `lib.rs` crate type `cdylib`, and copies the `.wasm` artifact to the
portfolio. We must not change anything that would break that pipeline.

### Recommended Structure: Cargo Workspace

Convert `leah/rust/` into a **Cargo workspace** with two members:

```
leah/rust/
  Cargo.toml                    # NEW - workspace root
  boid_simulation/              # UNCHANGED - the original crate
    Cargo.toml
    build_wasm.sh
    src/ ...
  boid_playground/              # NEW - your experimental binary
    Cargo.toml
    src/
      main.rs
      my_boid.rs
      my_boid_ui.rs
```

#### Why a workspace?

- **`boid_simulation` stays untouched.** Its Cargo.toml, source, and
  `build_wasm.sh` are exactly as they are today. The WASM pipeline keeps
  working because `cargo build --release --target wasm32-unknown-unknown` run
  from inside `boid_simulation/` still produces the same artifact.
- **`boid_playground` depends on `boid_simulation` as a path dependency.**
  It imports the existing `Boid`, `SimParams`, `SpatialGrid`, etc. as a
  library (via the `rlib` crate type that already exists).
- **Shared compilation cache.** Both crates share `target/` at the workspace
  root, so dependencies compile once.
- **Clean separation.** Experimental code never contaminates the portfolio
  build. You can iterate freely in `boid_playground`.

#### Key files

**`leah/rust/Cargo.toml`** (workspace root):
```toml
[workspace]
members = ["boid_simulation", "boid_playground"]
resolver = "2"
```

**`boid_playground/Cargo.toml`**:
```toml
[package]
name = "boid_playground"
version = "0.1.0"
edition = "2024"

[dependencies]
boid_simulation = { path = "../boid_simulation" }
macroquad = { version = "0.4", default-features = false }
egui-macroquad = "0.17"
```

#### What changes in `boid_simulation`?

Ideally, nothing in its source files. However, `boid_simulation/src/lib.rs`
currently declares modules with `mod` and re-runs the full game loop as a
`#[macroquad::main]` function. For `boid_playground` to import types from
it, we need the key structs and functions to be `pub` and accessible from
`lib.rs`. A quick audit shows:

- `Boid`, `SimParams`, `SpatialGrid`, `DiseaseState`, etc. are already `pub`.
- The modules are declared in `lib.rs` but not re-exported with `pub mod`.

**One small change needed:** In `boid_simulation/src/lib.rs`, change the
`mod` declarations to `pub mod` so downstream crates can access them:

```rust
pub mod constants;
pub mod sir;
pub mod boid;
pub mod simulation;
pub mod visualization;
pub mod spatial;
pub mod ui;
```

This has **zero effect** on the WASM build (adding `pub` to modules doesn't
change the compiled output for `cdylib`). The game loop function in `lib.rs`
remains as-is.

#### Running things

```bash
# Run the original simulation (unchanged)
cd boid_simulation && cargo run

# Run the playground
cd boid_playground && cargo run

# Build WASM (unchanged)
cd boid_simulation && ./build_wasm.sh
```

---

## Part 2: "My Boid" Feature Plan

### Concept

Drop **one special boid** into the existing simulation. It lives alongside
the regular flock but has its own independent flocking parameters, its own
UI panel, and a distinct visual treatment.

### 2.1 Data Model (`my_boid.rs`)

Create a `MyBoid` struct that wraps or parallels `Boid`:

- **Own position & velocity** (same `Vec2` fields).
- **Own `SimParams`-like parameter set** for flocking only — no disease
  state needed (or could be added later). Fields:
  - `perception_radius`
  - `separation_radius`
  - `separation_weight`, `alignment_weight`, `cohesion_weight`
  - `max_speed`, `max_force`
- **Flocking update**: Reuse the same separation/alignment/cohesion math
  from `Boid::update()`, but driven by its own parameter values. It queries
  the same `SpatialGrid` the regular boids use, so it reacts to the flock.
  The flock does **not** react to it (it isn't inserted into the grid), unless
  you decide to change that later.
- **No disease participation** for now — it's purely a flocking experiment.

### 2.2 Rendering

- **The special boid itself:** Draw it as a triangle (same shape as regular
  boids) but with **full-brightness white** (`Color::from_rgba(255, 255, 255, 255)`).
- **Glowing circle:** Draw a `draw_circle_lines()` around it in bright white
  with moderate radius (~20px) and a line thickness of ~2px. This circle
  moves with the boid. Optionally add a second, slightly larger, more
  transparent circle for a glow effect.
- **Dim the regular boids:** Modify the regular `Boid::draw()` to accept an
  alpha parameter (or use a wrapper). Set regular boid alpha to something
  like 120-160 (out of 255) so they appear faded. This keeps their color
  coding (white/red/blue/orange) readable but visually receded. Since we
  don't want to modify `boid_simulation` source, this means `boid_playground`
  will implement its own draw function for regular boids that overrides the
  alpha, rather than calling `boid.draw()` directly.

### 2.3 UI Panel (`my_boid_ui.rs`)

A separate egui window titled **"My Boid"**:

- Positioned below or beside the existing parameters panel.
- Contains sliders for the special boid's flocking parameters:
  - Perception Radius (10..150)
  - Separation Radius (5..50)
  - Separation Weight (0..5)
  - Alignment Weight (0..5)
  - Cohesion Weight (0..5)
  - Max Speed (0.5..5)
  - Max Force (0.01..0.5)
- Collapsible, same pattern as the existing panel.
- Styled with a distinct color (e.g., a blue-tinted frame) to differentiate
  it from the main parameter panel.

### 2.4 Integration in `boid_playground/src/main.rs`

The main loop in `boid_playground` will be a copy of the game loop from
`boid_simulation`, with these additions spliced in:

1. **Initialization:** Create one `MyBoid` at a random position.
2. **Per-frame, after building the spatial grid:**
   - Query neighbors for `MyBoid` using the existing grid.
   - Call `MyBoid::update()` with its own params and those neighbors.
3. **Drawing:**
   - Draw regular boids with dimmed alpha (custom draw, not `boid.draw()`).
   - Draw `MyBoid` with full brightness + circle.
4. **UI:**
   - Render the existing parameter panel (imported from `boid_simulation`).
   - Render the "My Boid" panel.

### 2.5 Files to Create

| File | Purpose |
|---|---|
| `leah/rust/Cargo.toml` | Workspace definition |
| `boid_playground/Cargo.toml` | Crate config with path dep on `boid_simulation` |
| `boid_playground/src/main.rs` | Game loop (based on original, with My Boid additions) |
| `boid_playground/src/my_boid.rs` | `MyBoid` struct, params, update, draw |
| `boid_playground/src/my_boid_ui.rs` | egui panel for My Boid parameters |

### 2.6 Files to Modify

| File | Change |
|---|---|
| `boid_simulation/src/lib.rs` | `mod` -> `pub mod` (6 lines) |

Nothing else in `boid_simulation` changes.

---

## Summary

- Workspace approach keeps the original crate and WASM pipeline safe.
- One tiny `pub mod` change in `lib.rs` to expose types.
- New `boid_playground` crate for all experimental work.
- "My Boid" = 1 special boid with own params, own UI panel, bright + circled,
  regular boids dimmed.
