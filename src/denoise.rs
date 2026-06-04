use crate::Ternary;

/// Simple denoising strategies for ternary signals.
pub struct DenoisingStrategy;

impl DenoisingStrategy {
    /// Apply a majority filter with the given window size (must be odd).
    /// Each element is replaced by the majority vote of its window.
    pub fn majority_filter(signal: &[Ternary], window_size: usize) -> Vec<Ternary> {
        assert!(window_size > 0 && window_size % 2 == 1, "window size must be odd and positive");
        if signal.is_empty() {
            return Vec::new();
        }

        let half = window_size / 2;
        let mut result = Vec::with_capacity(signal.len());

        for i in 0..signal.len() {
            let start = i.saturating_sub(half);
            let end = (i + half + 1).min(signal.len());

            let mut neg = 0usize;
            let mut zero = 0usize;
            let mut pos = 0usize;
            for &t in &signal[start..end] {
                match t {
                    Ternary::Neg => neg += 1,
                    Ternary::Zero => zero += 1,
                    Ternary::Pos => pos += 1,
                }
            }

            let winner = if pos >= neg && pos >= zero {
                Ternary::Pos
            } else if neg >= pos && neg >= zero {
                Ternary::Neg
            } else {
                Ternary::Zero
            };
            result.push(winner);
        }

        result
    }

    /// Apply a median filter with the given window size (must be odd).
    /// Each element is replaced by the median of its window.
    pub fn median_filter(signal: &[Ternary], window_size: usize) -> Vec<Ternary> {
        assert!(window_size > 0 && window_size % 2 == 1, "window size must be odd and positive");
        if signal.is_empty() {
            return Vec::new();
        }

        let half = window_size / 2;
        let mut result = Vec::with_capacity(signal.len());

        for i in 0..signal.len() {
            let start = i.saturating_sub(half);
            let end = (i + half + 1).min(signal.len());

            let mut values: Vec<i8> = signal[start..end]
                .iter()
                .map(|t| t.value())
                .collect();
            values.sort();

            let median = values[values.len() / 2];
            result.push(Ternary::from_value(median));
        }

        result
    }

    /// Apply multiple passes of majority filter.
    pub fn majority_multi_pass(signal: &[Ternary], window_size: usize, passes: usize) -> Vec<Ternary> {
        let mut current = signal.to_vec();
        for _ in 0..passes {
            current = Self::majority_filter(&current, window_size);
        }
        current
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn majority_filter_clean_signal() {
        let signal = vec![Ternary::Pos, Ternary::Pos, Ternary::Pos];
        let filtered = DenoisingStrategy::majority_filter(&signal, 3);
        assert_eq!(filtered, signal);
    }

    #[test]
    fn majority_filter_removes_impulse() {
        // Single impulse in constant signal
        let signal = vec![
            Ternary::Pos, Ternary::Pos, Ternary::Neg, Ternary::Pos, Ternary::Pos,
        ];
        let filtered = DenoisingStrategy::majority_filter(&signal, 3);
        assert_eq!(filtered[2], Ternary::Pos);
    }

    #[test]
    fn median_filter_preserves_step() {
        let signal = vec![
            Ternary::Neg, Ternary::Neg, Ternary::Pos, Ternary::Pos, Ternary::Pos,
        ];
        let filtered = DenoisingStrategy::median_filter(&signal, 3);
        // Edges should stay, center should be Pos
        assert_eq!(filtered[0], Ternary::Neg);
        assert_eq!(filtered[4], Ternary::Pos);
    }

    #[test]
    #[should_panic]
    fn majority_filter_even_window_panics() {
        DenoisingStrategy::majority_filter(&[], 2);
    }

    #[test]
    fn majority_filter_empty() {
        let filtered = DenoisingStrategy::majority_filter(&[], 3);
        assert!(filtered.is_empty());
    }

    #[test]
    fn median_filter_empty() {
        let filtered = DenoisingStrategy::median_filter(&[], 3);
        assert!(filtered.is_empty());
    }

    #[test]
    fn multi_pass_reduces_noise() {
        // Noisy signal with scattered impulses
        let mut signal = vec![Ternary::Zero; 20];
        signal[5] = Ternary::Pos;
        signal[10] = Ternary::Neg;
        signal[15] = Ternary::Pos;

        let filtered = DenoisingStrategy::majority_multi_pass(&signal, 3, 2);
        // After multi-pass, most should be Zero
        let zeros = filtered.iter().filter(|&&t| t == Ternary::Zero).count();
        assert!(zeros >= 17, "expected most to be zero, got {}/20", zeros);
    }
}
