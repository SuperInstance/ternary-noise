use crate::Ternary;

/// Type of noise to apply.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoiseModelType {
    Gaussian,
    Uniform,
    Impulse,
    Structured,
}

/// A simple xorshift64 PRNG — no external deps needed.
#[derive(Debug, Clone)]
struct Rng {
    state: u64,
}

impl Rng {
    fn new(seed: u64) -> Self {
        assert!(seed != 0, "RNG seed must be non-zero");
        Rng { state: seed }
    }

    fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x
    }

    /// Uniform f64 in [0, 1).
    fn next_f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }

    /// Approximate standard-normal via Box-Muller (two uniforms).
    fn next_normal(&mut self) -> f64 {
        let u1 = self.next_f64().max(1e-15);
        let u2 = self.next_f64();
        let z = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos();
        z
    }
}

/// Configurable noise model for ternary systems.
#[derive(Debug, Clone)]
pub struct NoiseModel {
    pub model_type: NoiseModelType,
    rng: Rng,
    /// For impulse noise: probability that any single element is corrupted.
    pub impulse_prob: f64,
    /// For structured noise: correlation length (number of adjacent elements affected).
    pub struct_corr_len: usize,
}

impl NoiseModel {
    pub fn gaussian(seed: u64) -> Self {
        NoiseModel {
            model_type: NoiseModelType::Gaussian,
            rng: Rng::new(seed),
            impulse_prob: 0.0,
            struct_corr_len: 1,
        }
    }

    pub fn uniform(seed: u64) -> Self {
        NoiseModel {
            model_type: NoiseModelType::Uniform,
            rng: Rng::new(seed),
            impulse_prob: 0.0,
            struct_corr_len: 1,
        }
    }

    pub fn impulse(seed: u64, prob: f64) -> Self {
        NoiseModel {
            model_type: NoiseModelType::Impulse,
            rng: Rng::new(seed),
            impulse_prob: prob,
            struct_corr_len: 1,
        }
    }

    pub fn structured(seed: u64, correlation_len: usize) -> Self {
        NoiseModel {
            model_type: NoiseModelType::Structured,
            rng: Rng::new(seed),
            impulse_prob: 0.0,
            struct_corr_len: correlation_len.max(1),
        }
    }

    /// Generate a noise perturbation value in [-1, 1] scaled by `strength`.
    pub fn noise_sample(&mut self, strength: f64) -> f64 {
        let raw = match self.model_type {
            NoiseModelType::Gaussian => {
                // Clamp normal to [-3, 3] then scale to [-1, 1]
                let n = self.rng.next_normal();
                (n / 3.0).clamp(-1.0, 1.0)
            }
            NoiseModelType::Uniform => {
                2.0 * self.rng.next_f64() - 1.0
            }
            NoiseModelType::Impulse => {
                if self.rng.next_f64() < self.impulse_prob {
                    // Large jump: pick -1 or +1
                    if self.rng.next_f64() < 0.5 { -1.0 } else { 1.0 }
                } else {
                    0.0
                }
            }
            NoiseModelType::Structured => {
                2.0 * self.rng.next_f64() - 1.0
            }
        };
        raw * strength
    }

    /// Apply noise to a single ternary value.
    pub fn perturb(&mut self, t: Ternary, strength: f64) -> Ternary {
        let n = self.noise_sample(strength);
        let v = t.value() as f64 + n * 3.0; // scale noise to ternary range
        Ternary::clamp(v.round() as i8)
    }

    /// Apply noise to a slice of ternary values.
    pub fn perturb_slice(&mut self, values: &[Ternary], strength: f64) -> Vec<Ternary> {
        match self.model_type {
            NoiseModelType::Structured => self.perturb_structured(values, strength),
            _ => values.iter().map(|&t| self.perturb(t, strength)).collect(),
        }
    }

    fn perturb_structured(&mut self, values: &[Ternary], strength: f64) -> Vec<Ternary> {
        let mut result = values.to_vec();
        let len = values.len();
        if len == 0 {
            return result;
        }
        let corr = self.struct_corr_len.min(len);
        let mut i = 0;
        while i < len {
            // Generate one noise sample for the whole correlated block
            let n = self.noise_sample(strength);
            for j in 0..corr {
                if i + j < len {
                    let v = result[i + j].value() as f64 + n * 3.0;
                    result[i + j] = Ternary::clamp(v.round() as i8);
                }
            }
            i += corr;
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rng_non_zero_seed() {
        let mut rng = Rng::new(42);
        let v = rng.next_u64();
        assert_ne!(v, 0);
    }

    #[test]
    #[should_panic]
    fn rng_zero_seed_panics() {
        Rng::new(0);
    }

    #[test]
    fn gaussian_noise_range() {
        let mut model = NoiseModel::gaussian(42);
        for _ in 0..100 {
            let v = model.noise_sample(1.0);
            assert!(v >= -1.0 && v <= 1.0, "noise out of range: {}", v);
        }
    }

    #[test]
    fn uniform_noise_range() {
        let mut model = NoiseModel::uniform(42);
        for _ in 0..100 {
            let v = model.noise_sample(1.0);
            assert!(v >= -1.0 && v <= 1.0);
        }
    }

    #[test]
    fn impulse_noise_sparse() {
        let mut model = NoiseModel::impulse(42, 0.1);
        let mut hits = 0i32;
        for _ in 0..1000 {
            let v = model.noise_sample(1.0);
            if v != 0.0 { hits += 1; }
        }
        // Should be roughly 10% hits — allow wide margin
        assert!(hits > 50 && hits < 200, "impulse hits: {}", hits);
    }

    #[test]
    fn perturb_returns_ternary() {
        let mut model = NoiseModel::uniform(99);
        for _ in 0..50 {
            let t = model.perturb(Ternary::Zero, 1.0);
            assert!(matches!(t, Ternary::Neg | Ternary::Zero | Ternary::Pos));
        }
    }
}
