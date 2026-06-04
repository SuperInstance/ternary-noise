//! # ternary-noise
//!
//! Study the effect of noise on ternary agent systems — how much noise can
//! they tolerate before conservation laws break?

mod ternary;
mod noise_model;
mod noisy_env;
mod tolerance;
mod conservation;
mod snr;
mod denoise;

pub use ternary::Ternary;
pub use noise_model::{NoiseModel, NoiseModelType};
pub use noisy_env::NoisyEnvironment;
pub use tolerance::NoiseToleranceTest;
pub use conservation::ConservationBreakpoint;
pub use snr::SignalToNoise;
pub use denoise::DenoisingStrategy;
