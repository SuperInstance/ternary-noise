use crate::{Ternary, NoiseModel, NoiseModelType, NoisyEnvironment};

/// Conservation law checks for ternary systems.
#[derive(Debug, Clone)]
pub struct ConservationState {
    /// Sum of all ternary values.
    pub sum: i64,
    /// Total energy (sum of absolute values).
    pub energy: i64,
    /// Count of each state: (neg, zero, pos).
    pub counts: (usize, usize, usize),
}

impl ConservationState {
    pub fn from_slice(values: &[Ternary]) -> Self {
        let mut neg = 0usize;
        let mut zero = 0usize;
        let mut pos = 0usize;
        for t in values {
            match t {
                Ternary::Neg => neg += 1,
                Ternary::Zero => zero += 1,
                Ternary::Pos => pos += 1,
            }
        }
        ConservationState {
            sum: Ternary::sum(values),
            energy: Ternary::energy(values),
            counts: (neg, zero, pos),
        }
    }

    /// Check if sum is conserved (within tolerance).
    pub fn sum_conserved(&self, reference: &ConservationState, tolerance: f64) -> bool {
        if reference.energy == 0 {
            return self.energy == 0;
        }
        let diff = (self.sum - reference.sum).unsigned_abs() as f64;
        diff <= tolerance * reference.energy as f64
    }

    /// Check if energy is conserved (within tolerance).
    pub fn energy_conserved(&self, reference: &ConservationState, tolerance: f64) -> bool {
        let diff = (self.energy - reference.energy).unsigned_abs() as f64;
        diff <= tolerance * reference.energy.max(1) as f64
    }

    /// Check if sign balance is conserved: pos/(pos+neg) ratio within tolerance.
    pub fn sign_balance_conserved(&self, reference: &ConservationState, tolerance: f64) -> bool {
        let ref_total = (reference.counts.0 + reference.counts.2).max(1) as f64;
        let self_total = (self.counts.0 + self.counts.2).max(1) as f64;
        let ref_ratio = reference.counts.2 as f64 / ref_total;
        let self_ratio = self.counts.2 as f64 / self_total;
        (self_ratio - ref_ratio).abs() <= tolerance
    }

    /// All conservation laws hold.
    pub fn all_conserved(&self, reference: &ConservationState, tolerance: f64) -> bool {
        self.sum_conserved(reference, tolerance)
            && self.energy_conserved(reference, tolerance)
            && self.sign_balance_conserved(reference, tolerance)
    }
}

/// Find the noise level where a conservation law first fails.
pub struct ConservationBreakpoint;

impl ConservationBreakpoint {
    /// Binary search for the SNR where conservation breaks.
    /// Returns the approximate SNR (dB) at which conservation fails.
    pub fn find(clean: &[Ternary], noise_type: NoiseModelType, tolerance: f64) -> f64 {
        let reference = ConservationState::from_slice(clean);
        let mut high_snr = 60.0;
        let mut low_snr = -40.0;

        for _ in 0..50 {
            let mid = (high_snr + low_snr) / 2.0;
            let model = Self::make_model(noise_type, 42);
            let mut env = NoisyEnvironment::new(model, mid);
            let noisy = env.inject(clean);
            let state = ConservationState::from_slice(&noisy);

            if state.all_conserved(&reference, tolerance) {
                low_snr = mid; // still conserved, push lower
            } else {
                high_snr = mid; // broken, push higher
            }
        }

        // Return the breakpoint (where it starts failing)
        low_snr
    }

    /// Check if conservation holds at a given SNR.
    pub fn check_at_snr(
        clean: &[Ternary],
        noise_type: NoiseModelType,
        snr_db: f64,
        tolerance: f64,
        seed: u64,
    ) -> bool {
        let reference = ConservationState::from_slice(clean);
        let model = Self::make_model(noise_type, seed);
        let mut env = NoisyEnvironment::new(model, snr_db);
        let noisy = env.inject(clean);
        let state = ConservationState::from_slice(&noisy);
        state.all_conserved(&reference, tolerance)
    }

    /// Detailed conservation report at a given SNR.
    pub fn report(
        clean: &[Ternary],
        noise_type: NoiseModelType,
        snr_db: f64,
        tolerance: f64,
        seed: u64,
    ) -> ConservationReport {
        let reference = ConservationState::from_slice(clean);
        let model = Self::make_model(noise_type, seed);
        let mut env = NoisyEnvironment::new(model, snr_db);
        let noisy = env.inject(clean);
        let state = ConservationState::from_slice(&noisy);

        ConservationReport {
            snr_db,
            sum_conserved: state.sum_conserved(&reference, tolerance),
            energy_conserved: state.energy_conserved(&reference, tolerance),
            sign_balance_conserved: state.sign_balance_conserved(&reference, tolerance),
            reference_sum: reference.sum,
            noisy_sum: state.sum,
            reference_energy: reference.energy,
            noisy_energy: state.energy,
        }
    }

    fn make_model(noise_type: NoiseModelType, seed: u64) -> NoiseModel {
        match noise_type {
            NoiseModelType::Gaussian => NoiseModel::gaussian(seed),
            NoiseModelType::Uniform => NoiseModel::uniform(seed),
            NoiseModelType::Impulse => NoiseModel::impulse(seed, 0.15),
            NoiseModelType::Structured => NoiseModel::structured(seed, 4),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConservationReport {
    pub snr_db: f64,
    pub sum_conserved: bool,
    pub energy_conserved: bool,
    pub sign_balance_conserved: bool,
    pub reference_sum: i64,
    pub noisy_sum: i64,
    pub reference_energy: i64,
    pub noisy_energy: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn balanced_signal() -> Vec<Ternary> {
        let mut v = Vec::new();
        for i in 0..60 {
            match i % 3 {
                0 => v.push(Ternary::Neg),
                1 => v.push(Ternary::Zero),
                _ => v.push(Ternary::Pos),
            }
        }
        v
    }

    #[test]
    fn conservation_state_counts() {
        let vals = vec![Ternary::Pos, Ternary::Neg, Ternary::Zero, Ternary::Pos];
        let state = ConservationState::from_slice(&vals);
        assert_eq!(state.counts, (1, 1, 2));
        assert_eq!(state.sum, 1);
        assert_eq!(state.energy, 3);
    }

    #[test]
    fn sum_conserved_identical() {
        let vals = balanced_signal();
        let a = ConservationState::from_slice(&vals);
        let b = ConservationState::from_slice(&vals);
        assert!(a.sum_conserved(&b, 0.0));
    }

    #[test]
    fn conservation_holds_at_high_snr() {
        let clean = balanced_signal();
        assert!(ConservationBreakpoint::check_at_snr(&clean, NoiseModelType::Gaussian, 50.0, 0.2, 42));
    }

    #[test]
    fn conservation_breaks_at_low_snr() {
        let clean = balanced_signal();
        assert!(!ConservationBreakpoint::check_at_snr(&clean, NoiseModelType::Gaussian, -30.0, 0.05, 42));
    }

    #[test]
    fn find_breakpoint_reasonable() {
        let clean = balanced_signal();
        let bp = ConservationBreakpoint::find(&clean, NoiseModelType::Gaussian, 0.1);
        assert!(bp > -40.0 && bp < 60.0, "breakpoint out of range: {}", bp);
    }

    #[test]
    fn report_structure() {
        let clean = balanced_signal();
        let report = ConservationBreakpoint::report(&clean, NoiseModelType::Uniform, 20.0, 0.1, 42);
        assert!((report.snr_db - 20.0).abs() < 1e-10);
    }
}
