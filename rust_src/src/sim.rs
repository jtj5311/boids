use std::collections::HashMap;

#[derive(Clone, Copy, Debug, Default)]
pub struct Vec2f {
    pub x: f32,
    pub y: f32,
}

impl Vec2f {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn add(self, other: Vec2f) -> Vec2f {
        Vec2f::new(self.x + other.x, self.y + other.y)
    }

    pub fn sub(self, other: Vec2f) -> Vec2f {
        Vec2f::new(self.x - other.x, self.y - other.y)
    }

    pub fn mul(self, s: f32) -> Vec2f {
        Vec2f::new(self.x * s, self.y * s)
    }

    pub fn div(self, s: f32) -> Vec2f {
        Vec2f::new(self.x / s, self.y / s)
    }

    pub fn length(self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    pub fn normalize(self) -> Vec2f {
        let len = self.length();
        if len > 0.0 {
            self.div(len)
        } else {
            Vec2f::default()
        }
    }

    pub fn limit(self, max: f32) -> Vec2f {
        let len = self.length();
        if len > max { self.mul(max / len) } else { self }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HealthState {
    Susceptible,
    Infected,
    Recovered,
}

#[derive(Clone, Copy, Debug)]
pub struct Boid {
    pub pos: Vec2f,
    pub vel: Vec2f,
    pub state: HealthState,
    pub infected_time: f32,
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

    fn set_cell_size(&mut self, cell_size: f32) {
        self.cell_size = cell_size.max(1.0);
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

#[derive(Clone, Copy, Debug)]
pub struct SimConfig {
    pub world_size: Vec2f,
    pub max_speed: f32,
    pub max_force: f32,
    pub neighbor_radius: f32,
    pub separation_radius: f32,
    pub weight_align: f32,
    pub weight_cohesion: f32,
    pub weight_separation: f32,
    pub infection_radius: f32,
    pub infection_beta: f32,
    pub infectious_period: f32,
    pub initial_infected: usize,
}

pub struct Simulation {
    pub boids: Vec<Boid>,
    grid: SpatialHash,
    cfg: SimConfig,
    rng: Lcg,
}

impl Simulation {
    pub fn new(count: usize, mut cfg: SimConfig, seed: u32) -> Self {
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
            boids.push(Boid {
                pos,
                vel,
                state: HealthState::Susceptible,
                infected_time: 0.0,
            });
        }
        cfg.neighbor_radius = cfg.neighbor_radius.max(1.0);
        let mut sim = Self {
            boids,
            grid: SpatialHash::new(cfg.neighbor_radius),
            cfg,
            rng,
        };
        sim.seed_infections();
        sim
    }

    pub fn set_params(
        &mut self,
        neighbor_radius: f32,
        separation_radius: f32,
        max_speed: f32,
        max_force: f32,
        weight_align: f32,
        weight_cohesion: f32,
        weight_separation: f32,
    ) {
        self.cfg.neighbor_radius = neighbor_radius.max(1.0);
        self.cfg.separation_radius = separation_radius
            .min(self.cfg.neighbor_radius)
            .max(0.5);
        self.cfg.max_speed = max_speed.max(1.0);
        self.cfg.max_force = max_force.max(1.0);
        self.cfg.weight_align = weight_align.max(0.0);
        self.cfg.weight_cohesion = weight_cohesion.max(0.0);
        self.cfg.weight_separation = weight_separation.max(0.0);
        self.grid.set_cell_size(self.cfg.neighbor_radius);
    }

    pub fn set_infection_params(&mut self, infection_radius: f32, infection_beta: f32, infectious_period: f32) {
        self.cfg.infection_radius = infection_radius.max(1.0);
        self.cfg.infection_beta = infection_beta.max(0.0);
        self.cfg.infectious_period = infectious_period.max(0.1);
    }

    pub fn set_world_size(&mut self, size: Vec2f) {
        self.cfg.world_size = size;
    }

    pub fn step(&mut self, dt: f32) {
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

        let mut newly_infected = vec![false; self.boids.len()];
        let infect_p = 1.0 - (-self.cfg.infection_beta * dt).exp();
        for i in 0..self.boids.len() {
            if self.boids[i].state != HealthState::Susceptible {
                continue;
            }
            let pos = self.boids[i].pos;
            let mut infected_contact = false;
            self.grid.for_each_neighbor(pos, |j| {
                if i == j || infected_contact {
                    return;
                }
                let other = self.boids[j];
                if other.state != HealthState::Infected {
                    return;
                }
                let dist = other.pos.sub(pos).length();
                if dist < self.cfg.infection_radius {
                    infected_contact = true;
                }
            });
            if infected_contact && self.rng.next_f32() < infect_p {
                newly_infected[i] = true;
            }
        }

        for (boid, accel) in self.boids.iter_mut().zip(accelerations) {
            boid.vel = boid.vel.add(accel.mul(dt)).limit(self.cfg.max_speed);
            boid.pos = boid.pos.add(boid.vel.mul(dt));
            boid.pos = wrap_position(boid.pos, self.cfg.world_size);
        }

        for (i, boid) in self.boids.iter_mut().enumerate() {
            if newly_infected[i] {
                boid.state = HealthState::Infected;
                boid.infected_time = 0.0;
            }
            if boid.state == HealthState::Infected {
                boid.infected_time += dt;
                if boid.infected_time >= self.cfg.infectious_period {
                    boid.state = HealthState::Recovered;
                }
            }
        }
    }

    fn rebuild_grid(&mut self) {
        self.grid.clear();
        for (i, b) in self.boids.iter().enumerate() {
            self.grid.insert(i, b.pos);
        }
    }

    fn seed_infections(&mut self) {
        let count = self.cfg.initial_infected.min(self.boids.len());
        for _ in 0..count {
            let idx = (self.rng.next_f32() * self.boids.len() as f32) as usize;
            let boid = &mut self.boids[idx];
            boid.state = HealthState::Infected;
            boid.infected_time = 0.0;
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
