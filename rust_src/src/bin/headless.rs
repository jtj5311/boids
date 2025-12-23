#[path = "../sim.rs"]
mod sim;

use sim::{HealthState, HIDDEN_SIZE, NnPolicy, SimConfig, Simulation, Vec2f, FEATURE_SIZE};

struct Rand {
    state: u32,
}

impl Rand {
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

    fn normal(&mut self) -> f32 {
        let u1 = self.next_f32().max(1e-6);
        let u2 = self.next_f32();
        let r = (-2.0 * u1.ln()).sqrt();
        let theta = 2.0 * std::f32::consts::PI * u2;
        r * theta.cos()
    }
}

fn main() {
    let cfg = SimConfig {
        world_size: Vec2f::new(800.0, 600.0),
        max_speed: 160.0,
        max_force: 80.0,
        neighbor_radius: 60.0,
        separation_radius: 22.0,
        infection_radius: 18.0,
        infection_beta: 1.2,
        infectious_period: 6.0,
        initial_infected: 8,
    };

    let mut sim = Simulation::new(1200, cfg, 1337);
    let mut rng = Rand::new(4242);

    let mut policies = [
        sim.policy_for(HealthState::Susceptible).clone(),
        sim.policy_for(HealthState::Infected).clone(),
        sim.policy_for(HealthState::Recovered).clone(),
    ];

    for state in [
        HealthState::Susceptible,
        HealthState::Infected,
        HealthState::Recovered,
    ] {
        let (best, best_score) = cem_one_iteration(&cfg, &policies, state, &mut rng);
        let idx = state_idx(state);
        policies[idx] = best;
        println!("CEM {:?} best_score {:.1}", state, best_score);
    }

    sim.set_policy_for(HealthState::Susceptible, policies[0].clone());
    sim.set_policy_for(HealthState::Infected, policies[1].clone());
    sim.set_policy_for(HealthState::Recovered, policies[2].clone());

    let final_counts = sim_counts_after(&cfg, &policies, 1337);
    println!(
        "Final S/I/R after one-iter CEM: {}/{}/{}",
        final_counts.0, final_counts.1, final_counts.2
    );
}

fn cem_one_iteration(
    cfg: &SimConfig,
    policies: &[NnPolicy; 3],
    state: HealthState,
    rng: &mut Rand,
) -> (NnPolicy, f32) {
    let pop_size = 24;
    let elite = 6;
    let sigma = 0.35;

    let base = &policies[state_idx(state)];
    let mean = base.to_vec();

    let mut candidates: Vec<(Vec<f32>, f32)> = Vec::with_capacity(pop_size);
    for _ in 0..pop_size {
        let mut params = mean.clone();
        for p in &mut params {
            *p += rng.normal() * sigma;
        }
        let score = evaluate(cfg, policies, state, &params);
        candidates.push((params, score));
    }

    candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    let best_score = candidates[0].1;

    let mut mean_params = vec![0.0; mean.len()];
    for i in 0..elite {
        for (dst, src) in mean_params.iter_mut().zip(candidates[i].0.iter()) {
            *dst += *src;
        }
    }
    let inv = 1.0 / elite as f32;
    for v in &mut mean_params {
        *v *= inv;
    }

    let policy = NnPolicy::from_vec(FEATURE_SIZE, HIDDEN_SIZE, &mean_params);
    (policy, best_score)
}

fn evaluate(cfg: &SimConfig, policies: &[NnPolicy; 3], state: HealthState, params: &[f32]) -> f32 {
    let mut sim = Simulation::new(1200, *cfg, 9001);
    sim.set_policy_for(HealthState::Susceptible, policies[0].clone());
    sim.set_policy_for(HealthState::Infected, policies[1].clone());
    sim.set_policy_for(HealthState::Recovered, policies[2].clone());
    let candidate = NnPolicy::from_vec(FEATURE_SIZE, HIDDEN_SIZE, params);
    sim.set_policy_for(state, candidate);

    let steps = 600;
    let dt = 1.0 / 60.0;
    for _ in 0..steps {
        sim.step(dt);
    }
    let counts = sim.counts();
    match state {
        HealthState::Susceptible => counts.susceptible as f32,
        HealthState::Infected => (counts.infected + counts.recovered) as f32,
        HealthState::Recovered => (counts.susceptible + counts.recovered) as f32,
    }
}

fn sim_counts_after(cfg: &SimConfig, policies: &[NnPolicy; 3], seed: u32) -> (usize, usize, usize) {
    let mut sim = Simulation::new(1200, *cfg, seed);
    sim.set_policy_for(HealthState::Susceptible, policies[0].clone());
    sim.set_policy_for(HealthState::Infected, policies[1].clone());
    sim.set_policy_for(HealthState::Recovered, policies[2].clone());
    let steps = 600;
    let dt = 1.0 / 60.0;
    for _ in 0..steps {
        sim.step(dt);
    }
    let counts = sim.counts();
    (counts.susceptible, counts.infected, counts.recovered)
}

fn state_idx(state: HealthState) -> usize {
    match state {
        HealthState::Susceptible => 0,
        HealthState::Infected => 1,
        HealthState::Recovered => 2,
    }
}
