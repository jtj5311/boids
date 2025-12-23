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

impl HealthState {
    fn idx(self) -> usize {
        match self {
            HealthState::Susceptible => 0,
            HealthState::Infected => 1,
            HealthState::Recovered => 2,
        }
    }
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
    pub infection_radius: f32,
    pub infection_beta: f32,
    pub infectious_period: f32,
    pub initial_infected: usize,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct SirCounts {
    pub susceptible: usize,
    pub infected: usize,
    pub recovered: usize,
}

pub const FEATURE_SIZE: usize = 14;
pub const HIDDEN_SIZE: usize = 16;

pub struct Simulation {
    pub boids: Vec<Boid>,
    grid: SpatialHash,
    cfg: SimConfig,
    rng: Lcg,
    policies: [NnPolicy; 3],
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
        cfg.separation_radius = cfg.separation_radius.min(cfg.neighbor_radius).max(0.5);
        cfg.infection_radius = cfg.infection_radius.max(1.0);
        cfg.infectious_period = cfg.infectious_period.max(0.1);
        let mut sim = Self {
            boids,
            grid: SpatialHash::new(cfg.neighbor_radius.max(cfg.infection_radius)),
            cfg,
            rng,
            policies: [
                NnPolicy::new(FEATURE_SIZE, HIDDEN_SIZE),
                NnPolicy::new(FEATURE_SIZE, HIDDEN_SIZE),
                NnPolicy::new(FEATURE_SIZE, HIDDEN_SIZE),
            ],
        };
        for policy in &mut sim.policies {
            policy.randomize(&mut sim.rng, 0.6);
        }
        sim.seed_infections();
        sim
    }

    pub fn set_motion_params(
        &mut self,
        neighbor_radius: f32,
        separation_radius: f32,
        max_speed: f32,
        max_force: f32,
    ) {
        self.cfg.neighbor_radius = neighbor_radius.max(1.0);
        self.cfg.separation_radius = separation_radius
            .min(self.cfg.neighbor_radius)
            .max(0.5);
        self.cfg.max_speed = max_speed.max(1.0);
        self.cfg.max_force = max_force.max(1.0);
        self.grid
            .set_cell_size(self.cfg.neighbor_radius.max(self.cfg.infection_radius));
    }

    pub fn set_infection_params(
        &mut self,
        infection_radius: f32,
        infection_beta: f32,
        infectious_period: f32,
    ) {
        self.cfg.infection_radius = infection_radius.max(1.0);
        self.cfg.infection_beta = infection_beta.max(0.0);
        self.cfg.infectious_period = infectious_period.max(0.1);
        self.grid
            .set_cell_size(self.cfg.neighbor_radius.max(self.cfg.infection_radius));
    }

    pub fn set_world_size(&mut self, size: Vec2f) {
        self.cfg.world_size = size;
    }

    pub fn step(&mut self, dt: f32) {
        self.rebuild_grid();
        let mut accelerations = vec![Vec2f::default(); self.boids.len()];
        let mut newly_infected = vec![false; self.boids.len()];
        let infect_p = 1.0 - (-self.cfg.infection_beta * dt).exp();

        for i in 0..self.boids.len() {
            let (inputs, infected_contact) = self.features_for(i);
            let policy = &self.policies[self.boids[i].state.idx()];
            let accel = policy.forward(&inputs).mul(self.cfg.max_force);
            accelerations[i] = accel.limit(self.cfg.max_force);
            if self.boids[i].state == HealthState::Susceptible
                && infected_contact
                && self.rng.next_f32() < infect_p
            {
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

    pub fn counts(&self) -> SirCounts {
        let mut counts = SirCounts::default();
        for boid in &self.boids {
            match boid.state {
                HealthState::Susceptible => counts.susceptible += 1,
                HealthState::Infected => counts.infected += 1,
                HealthState::Recovered => counts.recovered += 1,
            }
        }
        counts
    }

    pub fn policy_for(&self, state: HealthState) -> &NnPolicy {
        &self.policies[state.idx()]
    }

    pub fn set_policy_for(&mut self, state: HealthState, policy: NnPolicy) {
        self.policies[state.idx()] = policy;
    }

    pub fn policy_param_count(&self) -> usize {
        self.policies[0].param_count()
    }

    fn features_for(&self, idx: usize) -> ([f32; FEATURE_SIZE], bool) {
        let boid = self.boids[idx];
        let mut align_sum = Vec2f::default();
        let mut cohesion_sum = Vec2f::default();
        let mut separation_sum = Vec2f::default();
        let mut count = 0;
        let mut sep_count = 0;
        let mut infected_count = 0;
        let mut nearest_infected_dist = f32::INFINITY;
        let mut nearest_infected_dir = Vec2f::default();
        let mut infected_contact = false;

        self.grid.for_each_neighbor(boid.pos, |j| {
            if idx == j {
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
                if other.state == HealthState::Infected {
                    infected_count += 1;
                    if dist < nearest_infected_dist && dist > 0.0 {
                        nearest_infected_dist = dist;
                        nearest_infected_dir = offset.div(dist);
                    }
                }
            }
            if other.state == HealthState::Infected && dist < self.cfg.infection_radius {
                infected_contact = true;
            }
        });

        let mut inputs = [0.0; FEATURE_SIZE];
        let speed = boid.vel.length();
        inputs[0] = boid.vel.x / self.cfg.max_speed;
        inputs[1] = boid.vel.y / self.cfg.max_speed;
        inputs[2] = (speed / self.cfg.max_speed).clamp(0.0, 1.0);

        if count > 0 {
            let align = align_sum.div(count as f32).div(self.cfg.max_speed);
            inputs[3] = align.x;
            inputs[4] = align.y;
            let center = cohesion_sum.div(count as f32);
            let cohesion = center.sub(boid.pos).div(self.cfg.neighbor_radius);
            inputs[5] = cohesion.x;
            inputs[6] = cohesion.y;
        }

        if sep_count > 0 {
            let sep = separation_sum.div(sep_count as f32);
            inputs[7] = sep.x;
            inputs[8] = sep.y;
        }

        let neighbor_norm = (count as f32 / 20.0).clamp(0.0, 1.0);
        inputs[9] = neighbor_norm;

        if nearest_infected_dist.is_finite() {
            inputs[10] = nearest_infected_dir.x;
            inputs[11] = nearest_infected_dir.y;
            inputs[12] = (nearest_infected_dist / self.cfg.infection_radius).clamp(0.0, 1.0);
        } else {
            inputs[12] = 1.0;
        }

        if count > 0 {
            inputs[13] = infected_count as f32 / count as f32;
        }

        (inputs, infected_contact)
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

#[derive(Clone, Debug)]
pub struct NnPolicy {
    input_size: usize,
    hidden_size: usize,
    w1: Vec<f32>,
    b1: Vec<f32>,
    w2: Vec<f32>,
    b2: Vec<f32>,
}

impl NnPolicy {
    pub fn new(input_size: usize, hidden_size: usize) -> Self {
        Self {
            input_size,
            hidden_size,
            w1: vec![0.0; input_size * hidden_size],
            b1: vec![0.0; hidden_size],
            w2: vec![0.0; hidden_size * 2],
            b2: vec![0.0; 2],
        }
    }

    pub fn randomize(&mut self, rng: &mut Lcg, scale: f32) {
        for w in &mut self.w1 {
            *w = (rng.next_f32() * 2.0 - 1.0) * scale;
        }
        for b in &mut self.b1 {
            *b = (rng.next_f32() * 2.0 - 1.0) * scale;
        }
        for w in &mut self.w2 {
            *w = (rng.next_f32() * 2.0 - 1.0) * scale;
        }
        for b in &mut self.b2 {
            *b = (rng.next_f32() * 2.0 - 1.0) * scale;
        }
    }

    pub fn forward(&self, input: &[f32; FEATURE_SIZE]) -> Vec2f {
        let mut hidden = vec![0.0; self.hidden_size];
        for h in 0..self.hidden_size {
            let mut acc = self.b1[h];
            let row = h * self.input_size;
            for i in 0..self.input_size {
                acc += self.w1[row + i] * input[i];
            }
            hidden[h] = acc.tanh();
        }

        let mut out = [0.0; 2];
        for o in 0..2 {
            let mut acc = self.b2[o];
            let row = o * self.hidden_size;
            for h in 0..self.hidden_size {
                acc += self.w2[row + h] * hidden[h];
            }
            out[o] = acc.tanh();
        }
        Vec2f::new(out[0], out[1])
    }

    pub fn param_count(&self) -> usize {
        self.w1.len() + self.b1.len() + self.w2.len() + self.b2.len()
    }

    pub fn to_vec(&self) -> Vec<f32> {
        let mut params = Vec::with_capacity(self.param_count());
        params.extend_from_slice(&self.w1);
        params.extend_from_slice(&self.b1);
        params.extend_from_slice(&self.w2);
        params.extend_from_slice(&self.b2);
        params
    }

    pub fn from_vec(input_size: usize, hidden_size: usize, params: &[f32]) -> Self {
        let w1_len = input_size * hidden_size;
        let b1_len = hidden_size;
        let w2_len = hidden_size * 2;
        let b2_len = 2;
        let expected = w1_len + b1_len + w2_len + b2_len;
        let mut offset = 0;
        let mut take = |n: usize| {
            let slice = &params[offset..offset + n];
            offset += n;
            slice.to_vec()
        };
        let w1 = take(w1_len);
        let b1 = take(b1_len);
        let w2 = take(w2_len);
        let b2 = take(b2_len);
        let _ = expected;
        Self {
            input_size,
            hidden_size,
            w1,
            b1,
            w2,
            b2,
        }
    }
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
