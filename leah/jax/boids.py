"""Core boid simulation logic using JAX."""

import jax
import jax.numpy as jnp
from jax import random
from typing import Tuple, NamedTuple

from config import BoidConfig


class BoidState(NamedTuple):
    """State of the boid simulation."""
    positions: jnp.ndarray  # (N, 2)
    velocities: jnp.ndarray  # (N, 2)


def initialize_boids(key: random.PRNGKey, config: BoidConfig) -> BoidState:
    """Initialize boid positions and velocities randomly.

    Args:
        key: JAX random key
        config: Simulation configuration

    Returns:
        Initial boid state
    """
    key_pos, key_vel = random.split(key)

    # Random positions within world bounds
    positions = random.uniform(
        key_pos,
        shape=(config.num_boids, 2),
        minval=jnp.array([0.0, 0.0]),
        maxval=jnp.array([config.world_width, config.world_height])
    )

    # Random velocities
    angles = random.uniform(key_vel, shape=(config.num_boids,), minval=0, maxval=2 * jnp.pi)
    speeds = random.uniform(key_vel, shape=(config.num_boids,), minval=0.5, maxval=config.max_speed)
    velocities = jnp.stack([jnp.cos(angles) * speeds, jnp.sin(angles) * speeds], axis=1)

    return BoidState(positions=positions, velocities=velocities)


def compute_pairwise_distances(positions: jnp.ndarray) -> jnp.ndarray:
    """Compute pairwise distances between all boids.

    Args:
        positions: Boid positions (N, 2)

    Returns:
        Distance matrix (N, N)
    """
    # Compute differences: (N, 1, 2) - (1, N, 2) = (N, N, 2)
    diff = positions[:, None, :] - positions[None, :, :]
    # Compute distances
    distances = jnp.linalg.norm(diff, axis=2)
    return distances


def separation_force(positions: jnp.ndarray, distances: jnp.ndarray, config: BoidConfig) -> jnp.ndarray:
    """Compute separation force to avoid crowding neighbors.

    Args:
        positions: Boid positions (N, 2)
        distances: Pairwise distances (N, N)
        config: Simulation configuration

    Returns:
        Separation forces (N, 2)
    """
    # Mask for neighbors within separation radius (excluding self)
    mask = (distances > 0) & (distances < config.separation_radius)

    # Compute difference vectors
    diff = positions[:, None, :] - positions[None, :, :]  # (N, N, 2)

    # Normalize and weight by inverse distance
    distances_safe = jnp.where(distances[:, :, None] > 0, distances[:, :, None], 1.0)
    repulsion = diff / (distances_safe + 1e-8)

    # Apply mask and sum
    repulsion = jnp.where(mask[:, :, None], repulsion, 0.0)
    force = jnp.sum(repulsion, axis=1)

    return force


def alignment_force(velocities: jnp.ndarray, distances: jnp.ndarray, config: BoidConfig) -> jnp.ndarray:
    """Compute alignment force to match velocity with neighbors.

    Args:
        velocities: Boid velocities (N, 2)
        distances: Pairwise distances (N, N)
        config: Simulation configuration

    Returns:
        Alignment forces (N, 2)
    """
    # Mask for neighbors within alignment radius (excluding self)
    mask = (distances > 0) & (distances < config.alignment_radius)

    # Count neighbors for each boid
    neighbor_counts = jnp.sum(mask, axis=1, keepdims=True)

    # Average velocity of neighbors
    avg_velocity = jnp.sum(
        jnp.where(mask[:, :, None], velocities[None, :, :], 0.0),
        axis=1
    ) / jnp.maximum(neighbor_counts, 1.0)

    # Desired change in velocity
    force = avg_velocity - velocities

    return force


def cohesion_force(positions: jnp.ndarray, distances: jnp.ndarray, config: BoidConfig) -> jnp.ndarray:
    """Compute cohesion force to steer toward center of mass of neighbors.

    Args:
        positions: Boid positions (N, 2)
        distances: Pairwise distances (N, N)
        config: Simulation configuration

    Returns:
        Cohesion forces (N, 2)
    """
    # Mask for neighbors within cohesion radius (excluding self)
    mask = (distances > 0) & (distances < config.cohesion_radius)

    # Count neighbors for each boid
    neighbor_counts = jnp.sum(mask, axis=1, keepdims=True)

    # Center of mass of neighbors
    center_of_mass = jnp.sum(
        jnp.where(mask[:, :, None], positions[None, :, :], 0.0),
        axis=1
    ) / jnp.maximum(neighbor_counts, 1.0)

    # Desired direction toward center of mass
    force = center_of_mass - positions

    return force


def limit_magnitude(vectors: jnp.ndarray, max_mag: float) -> jnp.ndarray:
    """Limit the magnitude of vectors.

    Args:
        vectors: Input vectors (N, 2)
        max_mag: Maximum magnitude

    Returns:
        Limited vectors (N, 2)
    """
    magnitudes = jnp.linalg.norm(vectors, axis=1, keepdims=True)
    scale = jnp.minimum(1.0, max_mag / (magnitudes + 1e-8))
    return vectors * scale


def handle_edges(positions: jnp.ndarray, velocities: jnp.ndarray, config: BoidConfig) -> Tuple[jnp.ndarray, jnp.ndarray]:
    """Handle boids at world boundaries.

    Args:
        positions: Boid positions (N, 2)
        velocities: Boid velocities (N, 2)
        config: Simulation configuration

    Returns:
        Updated positions and velocities
    """
    if config.edge_mode == 'wrap':
        # Wrap around edges
        positions = jnp.mod(positions, jnp.array([config.world_width, config.world_height]))
        return positions, velocities
    else:  # bounce
        # Bounce off edges
        new_velocities = velocities.copy()

        # Bounce on x boundaries
        hit_left = positions[:, 0] < 0
        hit_right = positions[:, 0] > config.world_width
        new_velocities = jnp.where(
            (hit_left | hit_right)[:, None],
            new_velocities * jnp.array([-1.0, 1.0]),
            new_velocities
        )

        # Bounce on y boundaries
        hit_bottom = positions[:, 1] < 0
        hit_top = positions[:, 1] > config.world_height
        new_velocities = jnp.where(
            (hit_bottom | hit_top)[:, None],
            new_velocities * jnp.array([1.0, -1.0]),
            new_velocities
        )

        # Clamp positions to boundaries
        positions = jnp.clip(
            positions,
            jnp.array([0.0, 0.0]),
            jnp.array([config.world_width, config.world_height])
        )

        return positions, new_velocities


def update_boids(state: BoidState, config: BoidConfig) -> BoidState:
    """Update boid positions and velocities for one time step.

    Args:
        state: Current boid state
        config: Simulation configuration

    Returns:
        Updated boid state
    """
    positions, velocities = state.positions, state.velocities

    # Compute pairwise distances
    distances = compute_pairwise_distances(positions)

    # Compute forces
    sep_force = separation_force(positions, distances, config)
    align_force = alignment_force(velocities, distances, config)
    coh_force = cohesion_force(positions, distances, config)

    # Weight and combine forces
    total_force = (
        config.separation_weight * limit_magnitude(sep_force, config.max_force) +
        config.alignment_weight * limit_magnitude(align_force, config.max_force) +
        config.cohesion_weight * limit_magnitude(coh_force, config.max_force)
    )

    # Update velocities
    new_velocities = velocities + total_force
    new_velocities = limit_magnitude(new_velocities, config.max_speed)

    # Update positions
    new_positions = positions + new_velocities * config.dt

    # Handle boundaries
    new_positions, new_velocities = handle_edges(new_positions, new_velocities, config)

    return BoidState(positions=new_positions, velocities=new_velocities)


# JIT compile the update function for performance
update_boids_jit = jax.jit(update_boids, static_argnames=['config'])
