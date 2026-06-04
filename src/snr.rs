use crate::Ternary;

/// Measure effective signal-to-noise ratio of ternary decisions.
pub struct SignalToNoise;

impl SignalToNoise {
    /// Measure SNR in dB between clean and noisy ternary signals.
    pub fn measure(clean: &[Ternary], noisy: &[Ternary]) -> f64 {
        crate::noisy_env::measure_snr(clean, noisy)
    }

    /// Measure the bit error rate (fraction of differing elements).
    pub fn error_rate(clean: &[Ternary], noisy: &[Ternary]) -> f64 {
        assert_eq!(clean.len(), noisy.len());
        if clean.is_empty() {
            return 0.0;
        }
        let errors = clean.iter().zip(noisy.iter())
            .filter(|(c, n)| c != n)
            .count();
        errors as f64 / clean.len() as f64
    }

    /// Measure the mean squared error between clean and noisy.
    pub fn mse(clean: &[Ternary], noisy: &[Ternary]) -> f64 {
        assert_eq!(clean.len(), noisy.len());
        if clean.is_empty() {
            return 0.0;
        }
        let sum_sq: f64 = clean.iter().zip(noisy.iter())
            .map(|(c, n)| ((c.value() - n.value()) as f64).powi(2))
            .sum();
        sum_sq / clean.len() as f64
    }

    /// Signal power of a ternary signal.
    pub fn signal_power(signal: &[Ternary]) -> f64 {
        if signal.is_empty() {
            return 0.0;
        }
        signal.iter()
            .map(|t| (t.value() as f64).powi(2))
            .sum::<f64>() / signal.len() as f64
    }

    /// Noise power (difference between clean and noisy).
    pub fn noise_power(clean: &[Ternary], noisy: &[Ternary]) -> f64 {
        if clean.is_empty() {
            return 0.0;
        }
        clean.iter().zip(noisy.iter())
            .map(|(c, n)| ((c.value() - n.value()) as f64).powi(2))
            .sum::<f64>() / clean.len() as f64
    }

    /// Peak signal-to-noise ratio (PSNR) — max possible MSE is 4 (distance -1 to +1 = 2, squared = 4).
    pub fn psnr(clean: &[Ternary], noisy: &[Ternary]) -> f64 {
        let m = Self::mse(clean, noisy);
        if m < 1e-15 {
            return f64::INFINITY;
        }
        10.0 * (4.0 / m).log10() // peak_signal^2 = 4
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_rate_identical() {
        let v = vec![Ternary::Pos, Ternary::Neg, Ternary::Zero];
        assert_eq!(SignalToNoise::error_rate(&v, &v), 0.0);
    }

    #[test]
    fn error_rate_all_wrong() {
        let clean = vec![Ternary::Pos, Ternary::Pos];
        let noisy = vec![Ternary::Neg, Ternary::Neg];
        assert_eq!(SignalToNoise::error_rate(&clean, &noisy), 1.0);
    }

    #[test]
    fn mse_identical() {
        let v = vec![Ternary::Pos, Ternary::Zero];
        assert_eq!(SignalToNoise::mse(&v, &v), 0.0);
    }

    #[test]
    fn mse_known() {
        let clean = vec![Ternary::Pos];
        let noisy = vec![Ternary::Neg];
        // (1 - (-1))^2 = 4
        assert_eq!(SignalToNoise::mse(&clean, &noisy), 4.0);
    }

    #[test]
    fn signal_power_calc() {
        let v = vec![Ternary::Pos, Ternary::Zero]; // 1^2 + 0^2 = 1, avg = 0.5
        assert_eq!(SignalToNoise::signal_power(&v), 0.5);
    }

    #[test]
    fn psnr_identical() {
        let v = vec![Ternary::Pos];
        assert!(SignalToNoise::psnr(&v, &v).is_infinite());
    }
}
