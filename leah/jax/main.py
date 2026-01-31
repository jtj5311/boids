"""Main entry point for the boid simulation."""

import argparse
from jax import random

from config import BoidConfig
from boids import initialize_boids
from visualize import BoidVisualizer, plot_single_frame


def main():
    """Run the boid simulation."""
    parser = argparse.ArgumentParser(description='Run a boid simulation in JAX')
    parser.add_argument('--num-boids', type=int, default=200, help='Number of boids')
    parser.add_argument('--frames', type=int, default=500, help='Number of frames to simulate')
    parser.add_argument('--seed', type=int, default=42, help='Random seed')
    parser.add_argument('--save', type=str, default=None, help='Save animation to file (e.g., boids.mp4 or boids.gif)')
    parser.add_argument('--snapshot', action='store_true', help='Just save a single snapshot instead of animating')
    parser.add_argument('--world-size', type=float, default=100.0, help='World size (square)')

    args = parser.parse_args()

    # Create configuration
    config = BoidConfig()
    config.num_boids = args.num_boids
    config.world_width = args.world_size
    config.world_height = args.world_size

    print(f"Initializing simulation with {config.num_boids} boids...")
    print(f"World size: {config.world_width} x {config.world_height}")
    print(f"Separation radius: {config.separation_radius}")
    print(f"Alignment radius: {config.alignment_radius}")
    print(f"Cohesion radius: {config.cohesion_radius}")

    # Initialize boids
    key = random.PRNGKey(args.seed)
    state = initialize_boids(key, config)

    print(f"Initial state: {config.num_boids} boids initialized")

    if args.snapshot:
        # Just save a snapshot
        snapshot_path = args.save or 'boid_snapshot.png'
        plot_single_frame(state, config, save_path=snapshot_path)
    elif args.save:
        # Save animation to file
        print(f"Rendering {args.frames} frames...")
        visualizer = BoidVisualizer(config)
        visualizer.save_animation(state, args.save, num_frames=args.frames, fps=30)
    else:
        # Interactive animation
        print(f"Starting interactive animation ({args.frames} frames)...")
        print("Close the window to exit.")
        visualizer = BoidVisualizer(config)
        anim = visualizer.animate(state, num_frames=args.frames, interval=20)
        visualizer.show()


if __name__ == '__main__':
    main()
