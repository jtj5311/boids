use macroquad::prelude::*;
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, Default)]
struct Vec2f {
    x: f32,
    y: f32,
}

impl Vec2f {
    fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    fn add(self, other: Vec2f) -> Vec2f {
        Vec2f::new(self.x + other.x, self.y + other.y)
    }

    fn sub(self, other: Vec2f) -> Vec2f {
        Vec2f::new(self.x - other.x, self.y - other.y)
    }

    fn mul(self, s: f32) -> Vec2f {
        Vec2f::new(self.x * s, self.y * s)
    }

    fn div(self, s: f32) -> Vec2f {
        Vec2f::new(self.x / s, self.y / s)
    }

    fn length(self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    fn normalize(self) -> Vec2f {
        let len = self.length();
        if len > 0.0 {
            self.div(len)
        } else {
            Vec2f::default()
        }
    }

    fn limit(self, max: f32) -> Vec2f {
        let len = self.length();
        if len > max { self.mul(max / len) } else { self }
    }
}

#[derive(Clone, Copy, Debug)]
struct Boid {
    pos: Vec2f,
    vel: Vec2f,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct CellKey {
    x: i32,
    y: i32,
}

struct SpatialHash {
    cell_size: f32,
    buckets: HashMap<CellKey, Vec<usize>>,
}

impl SpatialHash {
    fn new(cell_size: f32) -> Self {
        Self {
            cell_size,
            buckets: HashMap::new(),
        }
    }

    fn clear(&mut self) {
        self.buckets.clear();
    }

    fn cell_key(&self, pos: Vec2f) -> CellKey {
        CellKey {
            x: (pos.x / self.cell_size).floor() as i32,
            y: (pos.y / self.cell_size).floor() as i32,
        }
    }

    fn insert(&mut self, idx: usize, pos: Vec2f) {
        let key = self.cell_key(pos);
        self.buckets.entry(key).or_default().push(idx);
    }

    fn for_each_neighbor(&self, pos: Vec2f, mut f: impl FnMut(usize)) {
        let key = self.cell_key(pos);
        for dy in -1..=1 {
            for dx in -1..=1 {
                let nk = CellKey {
                    x: key.x + dx,
                    y: key.y + dy,
                };
                if let Some(items) = self.buckets.get(&nk) {
                    for &idx in items {
                        f(idx);
                    }
                }
            }
        }
    }
}

struct SimConfig {
    world_size: Vec2f,
    max_speed: f32,
    max_force: f32,
    neighbor_radius: f32,
    separation_radius: f32,
    weight_align: f32,
    weight_cohesion: f32,
    weight_separation: f32,
}

struct Simulation {
    boids: Vec<Boid>,
    grid: SpatialHash,
    cfg: SimConfig,
}

impl Simulation {
    fn new(count: usize, mut cfg: SimConfig, seed: u32) -> Self {
        let mut boids = Vec::with_capacity(count);
        let mut rng = Lcg::new(seed);
        for _ in 0..count {
            let pos = Vec2f::new(
                rng.next_f32() * cfg.world_size.x,
                rng.next_f32() * cfg.world_size.y,
            );
            let angle = rng.next_f32() * std::f32::consts::TAU;
            let speed = cfg.max_speed * (0.3 + 0.7 * rng.next_f32());
            let vel = Vec2f::new(angle.cos(), angle.sin()).mul(speed);
            boids.push(Boid { pos, vel });
        }
        cfg.neighbor_radius = cfg.neighbor_radius.max(1.0);
        Self {
            boids,
            grid: SpatialHash::new(cfg.neighbor_radius),
            cfg,
        }
    }

    fn set_world_size(&mut self, size: Vec2f) {
        self.cfg.world_size = size;
    }

    fn rebuild_grid(&mut self) {
        self.grid.clear();
        for (i, b) in self.boids.iter().enumerate() {
            self.grid.insert(i, b.pos);
        }
    }

    fn step(&mut self, dt: f32) {
        self.rebuild_grid();
        let mut accelerations = vec![Vec2f::default(); self.boids.len()];

        for i in 0..self.boids.len() {
            let boid = self.boids[i];
            let mut align_sum = Vec2f::default();
            let mut cohesion_sum = Vec2f::default();
            let mut separation_sum = Vec2f::default();
            let mut count = 0;
            let mut sep_count = 0;

            self.grid.for_each_neighbor(boid.pos, |j| {
                if i == j {
                    return;
                }
                let other = self.boids[j];
                let offset = other.pos.sub(boid.pos);
                let dist = offset.length();
                if dist < self.cfg.neighbor_radius {
                    align_sum = align_sum.add(other.vel);
                    cohesion_sum = cohesion_sum.add(other.pos);
                    count += 1;
                    if dist < self.cfg.separation_radius && dist > 0.0 {
                        separation_sum = separation_sum.sub(offset.div(dist));
                        sep_count += 1;
                    }
                }
            });

            let mut accel = Vec2f::default();
            if count > 0 {
                let align = align_sum
                    .div(count as f32)
                    .normalize()
                    .mul(self.cfg.max_speed);
                let align = align.sub(boid.vel).limit(self.cfg.max_force);
                accel = accel.add(align.mul(self.cfg.weight_align));

                let center = cohesion_sum.div(count as f32);
                let cohesion = center.sub(boid.pos).normalize().mul(self.cfg.max_speed);
                let cohesion = cohesion.sub(boid.vel).limit(self.cfg.max_force);
                accel = accel.add(cohesion.mul(self.cfg.weight_cohesion));
            }
            if sep_count > 0 {
                let sep = separation_sum
                    .div(sep_count as f32)
                    .normalize()
                    .mul(self.cfg.max_speed);
                let sep = sep.sub(boid.vel).limit(self.cfg.max_force);
                accel = accel.add(sep.mul(self.cfg.weight_separation));
            }

            accelerations[i] = accel;
        }

        for (boid, accel) in self.boids.iter_mut().zip(accelerations) {
            boid.vel = boid.vel.add(accel.mul(dt)).limit(self.cfg.max_speed);
            boid.pos = boid.pos.add(boid.vel.mul(dt));
            boid.pos = wrap_position(boid.pos, self.cfg.world_size);
        }
    }
}

fn wrap_position(pos: Vec2f, size: Vec2f) -> Vec2f {
    let mut x = pos.x;
    let mut y = pos.y;
    if x < 0.0 {
        x += size.x;
    } else if x >= size.x {
        x -= size.x;
    }
    if y < 0.0 {
        y += size.y;
    } else if y >= size.y {
        y -= size.y;
    }
    Vec2f::new(x, y)
}

struct Lcg {
    state: u32,
}

impl Lcg {
    fn new(seed: u32) -> Self {
        Self { state: seed }
    }

    fn next_u32(&mut self) -> u32 {
        self.state = self.state.wrapping_mul(1664525).wrapping_add(1013904223);
        self.state
    }

    fn next_f32(&mut self) -> f32 {
        (self.next_u32() as f32) / (u32::MAX as f32)
    }
}

#[macroquad::main("Boids")]
async fn main() {
    let cfg = SimConfig {
        world_size: Vec2f::new(screen_width(), screen_height()),
        max_speed: 160.0,
        max_force: 80.0,
        neighbor_radius: 60.0,
        separation_radius: 22.0,
        weight_align: 1.0,
        weight_cohesion: 0.8,
        weight_separation: 1.4,
    };
    let mut sim = Simulation::new(2400, cfg, 1337);

    loop {
        let dt = get_frame_time().min(0.05);
        sim.set_world_size(Vec2f::new(screen_width(), screen_height()));
        sim.step(dt);

        clear_background(Color::from_rgba(8, 10, 14, 255));

        for boid in &sim.boids {
            let dir = boid.vel.normalize();
            let dir = if dir.length() > 0.0 {
                dir
            } else {
                Vec2f::new(1.0, 0.0)
            };
            let perp = Vec2f::new(-dir.y, dir.x);
            let tip = boid.pos.add(dir.mul(6.0));
            let left = boid.pos.sub(dir.mul(2.5)).add(perp.mul(3.0));
            let right = boid.pos.sub(dir.mul(2.5)).sub(perp.mul(3.0));

            draw_triangle(
                Vec2::new(tip.x, tip.y),
                Vec2::new(left.x, left.y),
                Vec2::new(right.x, right.y),
                Color::from_rgba(220, 240, 255, 255),
            );
        }

        next_frame().await;
    }
}
