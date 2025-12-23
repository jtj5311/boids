"""Configuration parameters for the boid simulation."""

import jax.numpy as jnp


class BoidConfig:
    """Configuration for boid simulation parameters."""

    # Simulation parameters
    num_boids: int = 200
    dt: float = 0.02  # Time step

    # World boundaries
    world_width: float = 100.0
    world_height: float = 100.0

    # Boid behavior parameters
    max_speed: float = 2.0
    max_force: float = 0.03

    # Perception radii
    separation_radius: float = 3.0
    alignment_radius: float = 5.0
    cohesion_radius: float = 5.0

    # Behavior weights
    separation_weight: float = 1.5
    alignment_weight: float = 1.0
    cohesion_weight: float = 1.0

    # Edge behavior ('wrap' or 'bounce')
    edge_mode: str = 'wrap'

    # Visualization
    boid_size: float = 0.5
    trail_length: int = 0  # Set > 0 to show trails


# Default configuration
default_config = BoidConfig()
