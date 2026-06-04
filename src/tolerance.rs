use crate::{Ternary, NoiseModel, NoiseModelType, NoisyEnvironment, noisy_env::measure_snr};

/// Result of a noise tolerance test at a specific SNR.
#[derive(Debug, Clone)]
pub struct ToleranceResult {
    pub snr_db: f64,
    pub fitness: f64,
    pub error_rate: f64,
    pub actual_snr: f64,
}

/// Test how much noise a strategy (represented as a clean ternary signal)
/// can handle before fitness drops below a threshold.
pub struct NoiseToleranceTest;

impl NoiseToleranceTest {
    /// Run a single test at a given SNR and return fitness metrics.
    pub fn test(clean: &[Ternary], noise_type: NoiseModelType, snr_db: f64, seed: u64) -> ToleranceResult {
        let model = Self::make_model(noise_type, seed);
        let mut env = NoisyEnvironment::new(model, snr_db);
        let noisy = env.inject(clean);

        let fitness = Self::compute_fitness(clean, &noisy);
        let error_rate = Self::compute_error_rate(clean, &noisy);
        let actual_snr = measure_snr(clean, &noisy);

        ToleranceResult {
            snr_db,
            fitness,
            error_rate,
            actual_snr,
        }
    }

    /// Sweep SNR from `snr_high` down to `snr_low` and collect results.
    pub fn sweep(
        clean: &[Ternary],
        noise_type: NoiseModelType,
        snr_high: f64,
        snr_low: f64,
        steps: usize,
        seed: u64,
    ) -> Vec<ToleranceResult> {
        let step_size = (snr_high - snr_low) / steps.max(1) as f64;
        (0..=steps)
            .map(|i| {
                let snr = snr_high - i as f64 * step_size;
                Self::test(clean, noise_type, snr, seed.wrapping_add(i as u64))
            })
            .collect()
    }

    /// Find the SNR at which fitness drops below `threshold` via binary search.
    pub fn find_tolerance(
        clean: &[Ternary],
        noise_type: NoiseModelType,
        threshold: f64,
        seed: u64,
    ) -> f64 {
        let mut high_snr = 50.0;
        let mut low_snr = -30.0;

        for _ in 0..50 {
            let mid = (high_snr + low_snr) / 2.0;
            let result = Self::test(clean, noise_type, mid, seed);
            if result.fitness >= threshold {
                high_snr = mid;
            } else {
                low_snr = mid;
            }
        }

        (high_snr + low_snr) / 2.0
    }

    fn compute_fitness(clean: &[Ternary], noisy: &[Ternary]) -> f64 {
        if clean.is_empty() {
            return 1.0;
        }
        let matches = clean.iter().zip(noisy.iter())
            .filter(|(c, n)| c == n)
            .count();
        matches as f64 / clean.len() as f64
    }

    fn compute_error_rate(clean: &[Ternary], noisy: &[Ternary]) -> f64 {
        1.0 - Self::compute_fitness(clean, noisy)
    }

    fn make_model(noise_type: NoiseModelType, seed: u64) -> NoiseModel {
        match noise_type {
            NoiseModelType::Gaussian => NoiseModel::gaussian(seed),
            NoiseModelType::Uniform => NoiseModel::uniform(seed),
            NoiseModelType::Impulse => NoiseModel::impulse(seed, 0.2),
            NoiseModelType::Structured => NoiseModel::structured(seed, 3),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_signal() -> Vec<Ternary> {
        [Ternary::Pos, Ternary::Zero, Ternary::Neg, Ternary::Pos, Ternary::Zero]
            .into_iter().cycle().take(100).collect()
    }

    #[test]
    fn test_high_snr_high_fitness() {
        let clean = sample_signal();
        let result = NoiseToleranceTest::test(&clean, NoiseModelType::Gaussian, 40.0, 42);
        assert!(result.fitness > 0.9, "fitness at 40 dB should be high: {}", result.fitness);
    }

    #[test]
    fn test_low_snr_low_fitness() {
        let clean = sample_signal();
        let result = NoiseToleranceTest::test(&clean, NoiseModelType::Gaussian, -20.0, 42);
        assert!(result.fitness < 0.6, "fitness at -20 dB should be low: {}", result.fitness);
    }

    #[test]
    fn sweep_returns_correct_count() {
        let clean = sample_signal();
        let results = NoiseToleranceTest::sweep(&clean, NoiseModelType::Uniform, 30.0, -10.0, 8, 42);
        assert_eq!(results.len(), 9); // steps + 1
    }

    #[test]
    fn sweep_fitness_decreases() {
        let clean = sample_signal();
        let results = NoiseToleranceTest::sweep(&clean, NoiseModelType::Uniform, 30.0, -10.0, 8, 42);
        let first = results.first().unwrap().fitness;
        let last = results.last().unwrap().fitness;
        assert!(first >= last, "fitness should decrease with lower SNR");
    }

    #[test]
    fn find_tolerance_returns_reasonable_snr() {
        let clean = sample_signal();
        let snr = NoiseToleranceTest::find_tolerance(&clean, NoiseModelType::Gaussian, 0.8, 42);
        assert!(snr > -30.0 && snr < 50.0, "tolerance SNR out of range: {}", snr);
    }
}
