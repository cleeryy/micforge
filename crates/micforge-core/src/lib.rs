//! MicForge core DSP: pure Rust, zero OS dependencies.
//!
//! Fixed-order chain (see [`chain::DspChain`]):
//! high-pass → noise-suppress → gate → compressor → 4-band EQ → de-esser → limiter.

pub mod blocks;
pub mod chain;
pub mod preset;

pub use blocks::{Compressor, DeEsser, Gate, HighPass, Limiter, NoiseSuppress, ParametricEq};
pub use chain::DspChain;
pub use preset::Preset;

pub(crate) fn db_to_lin(db: f32) -> f32 {
    10.0_f32.powf(db / 20.0)
}

pub(crate) fn lin_to_db(lin: f32) -> f32 {
    20.0 * lin.max(1e-9).log10()
}

/// Smoothed gain follower: `target` is chased with attack (rising) / release (falling) times.
#[derive(Debug, Clone, Default)]
pub(crate) struct Smoother {
    attack_coef: f32,
    release_coef: f32,
    pub value: f32,
}

impl Smoother {
    pub fn new(attack_ms: f32, release_ms: f32, sample_rate: f32) -> Self {
        Self {
            attack_coef: coef(attack_ms, sample_rate),
            release_coef: coef(release_ms, sample_rate),
            value: 0.0,
        }
    }
    pub fn set_times(&mut self, attack_ms: f32, release_ms: f32, sample_rate: f32) {
        self.attack_coef = coef(attack_ms, sample_rate);
        self.release_coef = coef(release_ms, sample_rate);
    }
    pub fn next(&mut self, target: f32) -> f32 {
        let c = if target > self.value {
            self.attack_coef
        } else {
            self.release_coef
        };
        self.value += c * (target - self.value);
        self.value
    }
    pub fn reset(&mut self, v: f32) {
        self.value = v;
    }
}

fn coef(ms: f32, sr: f32) -> f32 {
    let t = (ms.max(0.01) / 1000.0) * sr;
    1.0 - (-1.0 / t).exp()
}
