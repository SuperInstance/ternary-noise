/// A ternary state: -1, 0, or +1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Ternary {
    Neg,
    Zero,
    Pos,
}

impl Ternary {
    pub fn value(self) -> i8 {
        match self {
            Ternary::Neg => -1,
            Ternary::Zero => 0,
            Ternary::Pos => 1,
        }
    }

    pub fn from_value(v: i8) -> Self {
        match v {
            -1 => Ternary::Neg,
            0 => Ternary::Zero,
            1 => Ternary::Pos,
            _ => panic!("Ternary value must be -1, 0, or 1, got {}", v),
        }
    }

    pub fn clamp(v: i8) -> Self {
        match v {
            ..=-1 => Ternary::Neg,
            0 => Ternary::Zero,
            1.. => Ternary::Pos,
        }
    }

    /// Sum of a slice of ternary values.
    pub fn sum(values: &[Ternary]) -> i64 {
        values.iter().map(|t| t.value() as i64).sum()
    }

    /// Total energy: sum of absolute values.
    pub fn energy(values: &[Ternary]) -> i64 {
        values.iter().map(|t| t.value().unsigned_abs() as i64).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ternary_values() {
        assert_eq!(Ternary::Neg.value(), -1);
        assert_eq!(Ternary::Zero.value(), 0);
        assert_eq!(Ternary::Pos.value(), 1);
    }

    #[test]
    fn from_value() {
        assert_eq!(Ternary::from_value(-1), Ternary::Neg);
        assert_eq!(Ternary::from_value(0), Ternary::Zero);
        assert_eq!(Ternary::from_value(1), Ternary::Pos);
    }

    #[test]
    #[should_panic]
    fn from_value_invalid() {
        Ternary::from_value(2);
    }

    #[test]
    fn clamp_values() {
        assert_eq!(Ternary::clamp(-5), Ternary::Neg);
        assert_eq!(Ternary::clamp(-1), Ternary::Neg);
        assert_eq!(Ternary::clamp(0), Ternary::Zero);
        assert_eq!(Ternary::clamp(1), Ternary::Pos);
        assert_eq!(Ternary::clamp(7), Ternary::Pos);
    }

    #[test]
    fn sum_ternary() {
        let vals = vec![Ternary::Pos, Ternary::Neg, Ternary::Zero, Ternary::Pos];
        assert_eq!(Ternary::sum(&vals), 1);
    }

    #[test]
    fn energy_ternary() {
        let vals = vec![Ternary::Pos, Ternary::Neg, Ternary::Zero];
        assert_eq!(Ternary::energy(&vals), 2);
    }
}
