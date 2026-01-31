# Boid Simulation in JAX

A CPU-optimized boid simulation implementing the classic flocking behavior algorithm using JAX for efficient vectorized computation.

## Overview

This simulation implements Craig Reynolds' boids algorithm with three fundamental rules:
- **Separation**: Avoid crowding neighbors
- **Alignment**: Steer toward average heading of neighbors
- **Cohesion**: Steer toward center of mass of neighbors

## Installation

```bash
pip install -r requirements.txt
```

## Usage

### Basic simulation (interactive animation):
```bash
python main.py
```

### Custom parameters:
```bash
python main.py --num-boids 500 --frames 1000 --world-size 150
```

### Save animation to file:
```bash
# Save as MP4
python main.py --save boids.mp4 --frames 500

# Save as GIF
python main.py --save boids.gif --frames 200
```

### Save a single snapshot:
```bash
python main.py --snapshot --save snapshot.png
```

## Command-line Arguments

- `--num-boids`: Number of boids (default: 200)
- `--frames`: Number of frames to simulate (default: 500)
- `--seed`: Random seed for reproducibility (default: 42)
- `--world-size`: Size of square world (default: 100.0)
- `--save`: Save animation to file (MP4 or GIF)
- `--snapshot`: Save single frame instead of animation

## Configuration

Edit `config.py` to adjust simulation parameters:
- Perception radii (separation, alignment, cohesion)
- Behavior weights
- Speed limits
- Edge behavior (wrap or bounce)

## Performance

Runs efficiently on CPU thanks to JAX's JIT compilation. Expected performance:
- 100 boids: ~60 FPS
- 500 boids: ~20-30 FPS
- 1000 boids: ~10-15 FPS

## Project Structure

```
jax_src/
├── main.py         # Entry point
├── boids.py        # Core simulation logic
├── config.py       # Configuration parameters
├── visualize.py    # Matplotlib visualization
└── requirements.txt
```
