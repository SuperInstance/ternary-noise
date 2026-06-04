# Future Integration: ternary-noise

## Current State
Provides noise injection and tolerance testing for ternary strategies: `NoiseModel` with configurable types (Gaussian-like, uniform flip, burst), `NoisyEnvironment` injects noise at target SNR (dB), `NoiseToleranceTest` sweeps SNR to measure robustness, and SNR measurement utilities. Noise injection improves learning robustness by forcing strategies to generalize.

## Integration Opportunities

### With ternary-cell (Robustness Through Noise)
ternary-cell's predict phase can deliberately inject noise into predictions using ternary-noise's `NoisyEnvironment`. Noisy predictions force cells to maintain broader activation patterns (higher entropy), preventing premature convergence to a single ternary value. The `NoiseToleranceTest` sweep becomes a cell health metric: cells that tolerate more noise are more robust, cells that fail at low noise are brittle. `NoiseToleranceTest::sweep()` profiles each cell's noise tolerance curve.

### With ternary-adversarial (Adversarial vs. Random Noise)
ternary-adversarial perturbs trits strategically (maximize damage). ternary-noise perturbs randomly (Gaussian/uniform). Together they define the full robustness spectrum: random noise is the baseline, adversarial noise is the worst case. A strategy's robustness score combines both: it should tolerate moderate random noise (ternary-noise) and resist targeted adversarial perturbations (ternary-adversarial).

### With ternary-thermodynamics (Noise as Temperature)
ternary-thermodynamics' temperature parameter controls exploration. ternary-noise's SNR controls perturbation strength. They're the same thing: high temperature = low SNR = high noise = more exploration. The Boltzmann distribution at temperature T is equivalent to noisy strategy selection at SNR = f(T). This unifies thermodynamic and noise-based exploration under one framework.

## Potential in Mature Systems
In room-as-codespace, rooms receive noisy inputs from sensors (ESP32), network (ternary-protocol), and other rooms. ternary-noise provides the noise model and measurement tools. PLATO monitors each room's effective SNR and adjusts accordingly: low SNR rooms get error-correcting codes (ternary-codes), high SNR rooms run without overhead. During training, rooms use noise injection to build robustness, then operate clean in production.

## Cross-Pollination Ideas
- **ternary-steganography**: Noise provides cover for steganographic data — hidden messages are indistinguishable from natural noise.
- **ternary-streaming**: Real-time SNR monitoring on streaming windows — track signal quality degradation over time.
- **ternary-curriculum**: Progressive noise scheduling — start with high noise (easy to survive, forces exploration), gradually reduce noise as mastery increases.

## Dependencies for Next Steps
- Add noise injection to ternary-cell's predict phase as a configurable strategy
- Define combined robustness metric (random + adversarial noise tolerance)
- Implement SNR monitoring in PLATO room registry
- Benchmark noise injection overhead on ESP32
