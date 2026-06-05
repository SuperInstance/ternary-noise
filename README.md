# ternary-noise

Study the effect of noise on ternary agent systems — how much noise can they tolerate before conservation laws break?

## Overview

This crate provides tools for analyzing noise tolerance in ternary decision systems where agents operate in a {-1, 0, +1} state space. It answers a fundamental question: **at what noise level do conservation laws governing ternary dynamics break down?**

## Core Concepts

### Ternary States
Agents exist in one of three states: `-1`, `0`, or `+1`. Conservation laws govern the sum and distribution of states across the system.

### Noise Models
- **Gaussian noise**: Continuous perturbation drawn from a normal distribution (truncated to ternary range)
- **Uniform noise**: Equal-probability perturbation within a bounded range
- **Impulse noise**: Sparse, large-magnitude perturbations (salt-and-pepper style)
- **Structured noise**: Correlated perturbations that preserve some spatial structure

### Conservation Laws
In a clean ternary system, certain invariants hold:
1. **Sum conservation**: The total ternary energy is bounded
2. **Sign balance**: The ratio of positive to negative states follows predictable patterns
3. **Transition monotonicity**: State transitions follow ordered rules

Noise disrupts these invariants. This crate measures exactly how much.

## Methodology

### 1. Noise Injection
Noise is injected into ternary decisions at a configurable Signal-to-Noise Ratio (SNR). The SNR is expressed in decibels:

```
SNR_dB = 10 * log10(signal_power / noise_power)
```

Higher SNR = less noise. Lower SNR = more noise.

### 2. Tolerance Testing
For a given strategy, we sweep SNR from high (clean) to low (noisy) and measure:
- **Fitness**: How well the strategy performs its intended function
- **Conservation adherence**: Whether invariants still hold
- **Decision quality**: Effective accuracy of ternary decisions

### 3. Breakpoint Detection
The **conservation breakpoint** is the noise level where a conservation law first fails. This is found via binary search between a passing and failing SNR.

### 4. Signal-to-Noise Measurement
Effective SNR is measured by comparing the ternary decision stream against the clean (noise-free) reference:
- Treat clean decisions as the signal
- Deviations from clean as noise
- Compute power ratio

### 5. Denoising
Simple denoising strategies can recover some signal quality:
- **Majority filter**: Windowed majority vote among neighbors
- **Median filter**: Windowed median of ternary values

## Quick Start

```rust
use ternary_noise::*;

// Create a noise model
let noise = NoiseModel::gaussian(42);

// Create a noisy environment at 10 dB SNR
let env = NoisyEnvironment::new(noise, 10.0);

// Generate clean ternary decisions
let clean: Vec<Ternary> = vec![Ternary::Pos, Ternary::Neg, Ternary::Zero, Ternary::Pos];

// Inject noise
let noisy = env.inject(&clean);

// Measure effective SNR
let snr = SignalToNoise::measure(&clean, &noisy);

// Denoise
let denoised = DenoisingStrategy::majority_filter(&noisy, 3);

// Find conservation breakpoint
let breakpoint = ConservationBreakpoint::find(&clean, NoiseModelType::Gaussian, 0.9);
```

## API

- `Ternary` — The {-1, 0, +1} state type
- `NoiseModel` — Configurable noise generators
- `NoisyEnvironment` — Environment with noise injection at target SNR
- `NoiseToleranceTest` — Sweep noise levels and measure fitness
- `ConservationBreakpoint` — Binary search for conservation failure point
- `SignalToNoise` — Measure effective SNR of ternary streams
- `DenoisingStrategy` — Majority and median filters for ternary signals

## License

MIT

## See Also
- **ternary-signal** — related
- **ternary-entropy** — related
- **ternary-complexity** — related
- **ternary-denoise** — related
- **ternary-conservation** — related

