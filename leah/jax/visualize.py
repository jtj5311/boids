"""Visualization utilities for the boid simulation."""

import numpy as np
import matplotlib.pyplot as plt
from matplotlib.animation import FuncAnimation
from matplotlib.patches import FancyArrow
import jax.numpy as jnp

from boids import BoidState, update_boids_jit
from config import BoidConfig


class BoidVisualizer:
    """Visualizer for boid simulation using matplotlib."""

    def __init__(self, config: BoidConfig):
        """Initialize the visualizer.

        Args:
            config: Simulation configuration
        """
        self.config = config
        self.fig, self.ax = plt.subplots(figsize=(10, 10))
        self.setup_plot()

        # Scatter plot for boids
        self.scatter = None
        self.arrows = []

    def setup_plot(self):
        """Set up the matplotlib plot."""
        self.ax.set_xlim(0, self.config.world_width)
        self.ax.set_ylim(0, self.config.world_height)
        self.ax.set_aspect('equal')
        self.ax.set_title('Boid Simulation')
        self.ax.set_xlabel('X')
        self.ax.set_ylabel('Y')
        self.ax.grid(True, alpha=0.3)

    def draw_boids(self, state: BoidState):
        """Draw boids on the plot.

        Args:
            state: Current boid state
        """
        positions = np.array(state.positions)
        velocities = np.array(state.velocities)

        # Clear previous arrows
        for arrow in self.arrows:
            arrow.remove()
        self.arrows.clear()

        # Draw boids as points
        if self.scatter is None:
            self.scatter = self.ax.scatter(
                positions[:, 0],
                positions[:, 1],
                c='blue',
                s=20,
                alpha=0.6
            )
        else:
            self.scatter.set_offsets(positions)

        # Draw velocity vectors as small arrows (every Nth boid for clarity)
        step = max(1, self.config.num_boids // 50)  # Limit to ~50 arrows
        for i in range(0, len(positions), step):
            # Normalize velocity for arrow length
            vel_norm = velocities[i] / (np.linalg.norm(velocities[i]) + 1e-8)
            arrow = FancyArrow(
                positions[i, 0],
                positions[i, 1],
                vel_norm[0] * 2,
                vel_norm[1] * 2,
                width=0.3,
                head_width=0.8,
                head_length=0.5,
                fc='red',
                ec='red',
                alpha=0.5
            )
            self.arrows.append(self.ax.add_patch(arrow))

    def animate(self, state: BoidState, num_frames: int = 500, interval: int = 20):
        """Create an animation of the boid simulation.

        Args:
            state: Initial boid state
            num_frames: Number of frames to animate
            interval: Delay between frames in milliseconds

        Returns:
            matplotlib animation object
        """
        current_state = [state]  # Use list to allow mutation in closure

        def update_frame(frame):
            """Update function for animation."""
            # Update simulation
            current_state[0] = update_boids_jit(current_state[0], self.config)

            # Draw updated state
            self.draw_boids(current_state[0])

            # Update title with frame number
            self.ax.set_title(f'Boid Simulation - Frame {frame}')

            return self.scatter, *self.arrows

        anim = FuncAnimation(
            self.fig,
            update_frame,
            frames=num_frames,
            interval=interval,
            blit=False
        )

        return anim

    def show(self):
        """Display the plot."""
        plt.show()

    def save_animation(self, state: BoidState, filename: str, num_frames: int = 500, fps: int = 30):
        """Save animation to file.

        Args:
            state: Initial boid state
            filename: Output filename (e.g., 'boids.mp4' or 'boids.gif')
            num_frames: Number of frames to render
            fps: Frames per second
        """
        anim = self.animate(state, num_frames=num_frames, interval=1000//fps)

        # Determine writer based on file extension
        if filename.endswith('.gif'):
            writer = 'pillow'
        else:
            writer = 'ffmpeg'

        anim.save(filename, writer=writer, fps=fps)
        print(f"Animation saved to {filename}")


def plot_single_frame(state: BoidState, config: BoidConfig, save_path: str = None):
    """Plot a single frame of the simulation.

    Args:
        state: Boid state to visualize
        config: Simulation configuration
        save_path: Optional path to save the figure
    """
    fig, ax = plt.subplots(figsize=(10, 10))
    ax.set_xlim(0, config.world_width)
    ax.set_ylim(0, config.world_height)
    ax.set_aspect('equal')
    ax.set_title('Boid Simulation')
    ax.grid(True, alpha=0.3)

    positions = np.array(state.positions)
    velocities = np.array(state.velocities)

    # Draw boids
    ax.scatter(positions[:, 0], positions[:, 1], c='blue', s=20, alpha=0.6)

    # Draw velocity vectors
    step = max(1, config.num_boids // 50)
    for i in range(0, len(positions), step):
        vel_norm = velocities[i] / (np.linalg.norm(velocities[i]) + 1e-8)
        ax.arrow(
            positions[i, 0],
            positions[i, 1],
            vel_norm[0] * 2,
            vel_norm[1] * 2,
            head_width=0.8,
            head_length=0.5,
            fc='red',
            ec='red',
            alpha=0.5
        )

    if save_path:
        plt.savefig(save_path, dpi=150, bbox_inches='tight')
        print(f"Figure saved to {save_path}")
    else:
        plt.show()

    plt.close()
