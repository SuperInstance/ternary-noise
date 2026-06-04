use crate::{NoiseModel, Ternary};

/// An environment that injects noise at a target SNR (in dB).
#[derive(Debug, Clone)]
pub struct NoisyEnvironment {
    pub noise_model: NoiseModel,
    pub snr_db: f64,
}

impl NoisyEnvironment {
    pub fn new(noise_model: NoiseModel, snr_db: f64) -> Self {
        NoisyEnvironment { noise_model, snr_db }
    }

    /// Convert SNR in dB to a noise strength parameter.
    /// Higher SNR → less noise → lower strength.
    /// At 0 dB, strength = 1.0. At +∞ dB, strength → 0.
    pub fn noise_strength(&self) -> f64 {
        if self.snr_db > 100.0 {
            return 0.0;
        }
        // SNR_linear = 10^(snr_db/10)
        // strength = 1 / sqrt(SNR_linear) = 10^(-snr_db/20)
        let linear = 10f64.powf(self.snr_db / 10.0);
        1.0 / linear.sqrt().max(1e-10)
    }

    /// Inject noise into a clean ternary signal.
    pub fn inject(&mut self, clean: &[Ternary]) -> Vec<Ternary> {
        let strength = self.noise_strength();
        self.noise_model.perturb_slice(clean, strength)
    }
}

/// Measure actual SNR of a noisy signal relative to clean.
pub fn measure_snr(clean: &[Ternary], noisy: &[Ternary]) -> f64 {
    assert_eq!(clean.len(), noisy.len(), "signal lengths must match");
    if clean.is_empty() {
        return f64::INFINITY;
    }

    let signal_power: f64 = clean.iter()
        .map(|t| (t.value() as f64).powi(2))
        .sum::<f64>() / clean.len() as f64;

    let noise_power: f64 = clean.iter().zip(noisy.iter())
        .map(|(c, n)| ((c.value() - n.value()) as f64).powi(2))
        .sum::<f64>() / clean.len() as f64;

    if noise_power < 1e-15 {
        return f64::INFINITY;
    }

    10.0 * (signal_power / noise_power).log10()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::noise_model::NoiseModelType;

    #[test]
    fn noise_strength_high_snr() {
        let env = NoisyEnvironment::new(NoiseModel::gaussian(42), 40.0);
        assert!(env.noise_strength() < 0.02);
    }

    #[test]
    fn noise_strength_low_snr() {
        let env = NoisyEnvironment::new(NoiseModel::gaussian(42), -10.0);
        assert!(env.noise_strength() > 1.0);
    }

    #[test]
    fn noise_strength_zero_db() {
        let env = NoisyEnvironment::new(NoiseModel::gaussian(42), 0.0);
        assert!((env.noise_strength() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn inject_preserves_length() {
        let mut env = NoisyEnvironment::new(NoiseModel::gaussian(42), 10.0);
        let clean = vec![Ternary::Pos; 100];
        let noisy = env.inject(&clean);
        assert_eq!(noisy.len(), 100);
    }

    #[test]
    fn inject_all_ternary() {
        let mut env = NoisyEnvironment::new(NoiseModel::uniform(42), -5.0);
        let clean = vec![Ternary::Pos; 50];
        let noisy = env.inject(&clean);
        for t in &noisy {
            assert!(matches!(t, Ternary::Neg | Ternary::Zero | Ternary::Pos));
        }
    }

    #[test]
    fn measure_snr_identical() {
        let clean = vec![Ternary::Pos, Ternary::Neg, Ternary::Zero];
        assert!(measure_snr(&clean, &clean).is_infinite());
    }

    #[test]
    fn measure_snr_noisy() {
        let clean = vec![Ternary::Pos; 100];
        let noisy = vec![Ternary::Neg; 100];
        let snr = measure_snr(&clean, &noisy);
        // Completely wrong: noise power = signal power → 0 dB
        assert!(snr < 0.1, "expected ~0 dB, got {}", snr);
    }
}
