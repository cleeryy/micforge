//! Individual DSP blocks. All operate in place on `&mut [f32]` mono samples.

use crate::{Smoother, db_to_lin, lin_to_db};
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------- high-pass
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HighPass {
    pub enabled: bool,
    #[serde(rename = "frequencyHz")]
    pub frequency_hz: f32,
    #[serde(skip)]
    prev_in: f32,
    #[serde(skip)]
    prev_out: f32,
}

impl HighPass {
    pub fn new(frequency_hz: f32) -> Self {
        Self {
            enabled: true,
            frequency_hz,
            prev_in: 0.0,
            prev_out: 0.0,
        }
    }
    pub fn reset(&mut self) {
        self.prev_in = 0.0;
        self.prev_out = 0.0;
    }
    pub fn process(&mut self, buf: &mut [f32], sample_rate: f32) {
        if !self.enabled {
            return;
        }
        let rc = 1.0 / (2.0 * std::f32::consts::PI * self.frequency_hz.max(1.0));
        let dt = 1.0 / sample_rate;
        let a = rc / (rc + dt);
        for s in buf.iter_mut() {
            let x = *s;
            let y = a * (self.prev_out + x - self.prev_in);
            self.prev_in = x;
            self.prev_out = y;
            *s = y;
        }
    }
}

// ------------------------------------------------------- noise suppress stub
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoiseSuppress {
    pub enabled: bool,
}

impl NoiseSuppress {
    pub fn new() -> Self {
        Self { enabled: true }
    }
    /// Pass-through stub. Real neural suppression lands behind the
    /// `nnnoiseless` feature flag (see crate docs).
    pub fn process(&mut self, _buf: &mut [f32], _sample_rate: f32) {
        #[cfg(feature = "nnnoiseless")]
        compile_error!("nnnoiseless integration not yet implemented");
    }
}

impl Default for NoiseSuppress {
    fn default() -> Self {
        Self::new()
    }
}

// ------------------------------------------------------------------- gate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gate {
    pub enabled: bool,
    #[serde(rename = "thresholdDb")]
    pub threshold_db: f32,
    #[serde(rename = "attackMs")]
    pub attack_ms: f32,
    #[serde(rename = "releaseMs")]
    pub release_ms: f32,
    #[serde(skip)]
    env: f32,
    #[serde(skip)]
    gain: f32,
}

impl Gate {
    pub fn new(threshold_db: f32, attack_ms: f32, release_ms: f32) -> Self {
        Self {
            enabled: true,
            threshold_db,
            attack_ms,
            release_ms,
            env: 0.0,
            gain: 0.0,
        }
    }
    pub fn reset(&mut self) {
        self.env = 0.0;
        self.gain = 0.0;
    }
    pub fn process(&mut self, buf: &mut [f32], sample_rate: f32) {
        if !self.enabled {
            return;
        }
        let atk = attack_release_coef(self.attack_ms, sample_rate);
        let rel = attack_release_coef(self.release_ms, sample_rate);
        let thr = db_to_lin(self.threshold_db);
        for s in buf.iter_mut() {
            let a = s.abs();
            let c = if a > self.env { atk } else { rel };
            self.env += c * (a - self.env);
            let target = if self.env >= thr { 1.0 } else { 0.0 };
            // Fast open (attack), slow close (release).
            let gc = if target > self.gain { atk } else { rel };
            self.gain += gc * (target - self.gain);
            *s *= self.gain;
        }
    }
}

// ------------------------------------------------------------- compressor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Compressor {
    pub enabled: bool,
    #[serde(rename = "thresholdDb")]
    pub threshold_db: f32,
    pub ratio: f32,
    #[serde(rename = "attackMs")]
    pub attack_ms: f32,
    #[serde(rename = "releaseMs")]
    pub release_ms: f32,
    #[serde(rename = "makeupDb")]
    pub makeup_db: f32,
    #[serde(skip)]
    smoother: Smoother,
}

impl Compressor {
    pub fn new(threshold_db: f32, ratio: f32, attack_ms: f32, release_ms: f32, makeup_db: f32) -> Self {
        Self {
            enabled: true,
            threshold_db,
            ratio: ratio.max(1.0),
            attack_ms,
            release_ms,
            makeup_db,
            smoother: Smoother::new(attack_ms, release_ms, 48000.0),
        }
    }
    pub fn reset(&mut self, sample_rate: f32) {
        self.smoother = Smoother::new(self.attack_ms, self.release_ms, sample_rate);
    }
    pub fn process(&mut self, buf: &mut [f32], sample_rate: f32) {
        if !self.enabled {
            return;
        }
        self.smoother.set_times(self.attack_ms, self.release_ms, sample_rate);
        let makeup = db_to_lin(self.makeup_db);
        for s in buf.iter_mut() {
            let in_db = lin_to_db(s.abs());
            let over = (in_db - self.threshold_db).max(0.0);
            let target_db = self.threshold_db + over / self.ratio.max(1.0) - in_db;
            let target_lin = db_to_lin(target_db);
            let g = self.smoother.next(target_lin);
            *s *= g * makeup;
        }
    }
}

// ------------------------------------------------------- parametric EQ 4-band
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct EqBand {
    #[serde(rename = "frequencyHz")]
    pub frequency_hz: f32,
    #[serde(rename = "gainDb")]
    pub gain_db: f32,
    pub q: f32,
}

#[derive(Debug, Clone)]
struct Biquad {
    b0: f32, b1: f32, b2: f32, a1: f32, a2: f32,
    x1: f32, x2: f32, y1: f32, y2: f32,
}

impl Biquad {
    fn peaking(freq: f32, gain_db: f32, q: f32, sr: f32) -> Self {
        let a = 10.0_f32.powf(gain_db / 40.0);
        let w0 = 2.0 * std::f32::consts::PI * freq / sr;
        let alpha = w0.sin() / (2.0 * q.max(0.1));
        let (c, s) = (w0.cos(), w0.sin());
        let b0 = 1.0 + alpha * a;
        let b1 = -2.0 * c;
        let b2 = 1.0 - alpha * a;
        let a0 = 1.0 + alpha / a;
        let a1 = -2.0 * c;
        let a2 = 1.0 - alpha / a;
        let _ = s;
        Self {
            b0: b0 / a0, b1: b1 / a0, b2: b2 / a0,
            a1: a1 / a0, a2: a2 / a0,
            x1: 0.0, x2: 0.0, y1: 0.0, y2: 0.0,
        }
    }
    fn step(&mut self, x: f32) -> f32 {
        let y = self.b0 * x + self.b1 * self.x1 + self.b2 * self.x2
            - self.a1 * self.y1
            - self.a2 * self.y2;
        self.x2 = self.x1;
        self.x1 = x;
        self.y2 = self.y1;
        self.y1 = y;
        y
    }
    fn reset(&mut self) {
        self.x1 = 0.0; self.x2 = 0.0; self.y1 = 0.0; self.y2 = 0.0;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParametricEq {
    pub enabled: bool,
    pub bands: [EqBand; 4],
    #[serde(skip)]
    filters: Vec<Biquad>,
    #[serde(skip)]
    sample_rate: f32,
}

impl ParametricEq {
    pub fn new(bands: [EqBand; 4], sample_rate: f32) -> Self {
        let mut eq = Self {
            enabled: true,
            bands,
            filters: Vec::new(),
            sample_rate,
        };
        eq.rebuild(sample_rate);
        eq
    }
    pub fn rebuild(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        self.filters = self
            .bands
            .iter()
            .map(|b| Biquad::peaking(b.frequency_hz, b.gain_db, b.q, sample_rate))
            .collect();
    }
    pub fn reset(&mut self) {
        for f in &mut self.filters {
            f.reset();
        }
    }
    pub fn process(&mut self, buf: &mut [f32], sample_rate: f32) {
        if !self.enabled {
            return;
        }
        if (sample_rate - self.sample_rate).abs() > f32::EPSILON
            || self.filters.len() != 4
        {
            self.rebuild(sample_rate);
        }
        for s in buf.iter_mut() {
            let mut y = *s;
            for f in self.filters.iter_mut() {
                y = f.step(y);
            }
            *s = y;
        }
    }
}

// ---------------------------------------------------------------- de-esser
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeEsser {
    pub enabled: bool,
    #[serde(rename = "frequencyHz")]
    pub frequency_hz: f32,
    #[serde(rename = "thresholdDb")]
    pub threshold_db: f32,
    #[serde(skip)]
    hp_prev_in: f32,
    #[serde(skip)]
    hp_prev_out: f32,
    #[serde(skip)]
    smoother: Smoother,
}

impl DeEsser {
    pub fn new(frequency_hz: f32, threshold_db: f32) -> Self {
        Self {
            enabled: true,
            frequency_hz,
            threshold_db,
            hp_prev_in: 0.0,
            hp_prev_out: 0.0,
            smoother: Smoother::new(2.0, 60.0, 48000.0),
        }
    }
    pub fn reset(&mut self) {
        self.hp_prev_in = 0.0;
        self.hp_prev_out = 0.0;
        self.smoother.reset(1.0);
    }
    pub fn process(&mut self, buf: &mut [f32], sample_rate: f32) {
        if !self.enabled {
            return;
        }
        self.smoother.set_times(2.0, 60.0, sample_rate);
        if self.smoother.value == 0.0 {
            self.smoother.reset(1.0);
        }
        let rc = 1.0 / (2.0 * std::f32::consts::PI * self.frequency_hz.max(1.0));
        let dt = 1.0 / sample_rate;
        let a = rc / (rc + dt);
        let thr = db_to_lin(self.threshold_db);
        for s in buf.iter_mut() {
            let x = *s;
            let hf = a * (self.hp_prev_out + x - self.hp_prev_in);
            self.hp_prev_in = x;
            self.hp_prev_out = hf;
            let over = (hf.abs() / thr.max(1e-6)).min(4.0);
            let target = if over > 1.0 { 1.0 / over } else { 1.0 };
            let g = self.smoother.next(target);
            *s = x * g;
        }
    }
}

// ---------------------------------------------------------------- limiter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Limiter {
    pub enabled: bool,
    #[serde(rename = "ceilingDb")]
    pub ceiling_db: f32,
    #[serde(rename = "releaseMs")]
    pub release_ms: f32,
    #[serde(skip)]
    gain: f32,
}

impl Limiter {
    pub fn new(ceiling_db: f32, release_ms: f32) -> Self {
        Self {
            enabled: true,
            ceiling_db,
            release_ms,
            gain: 1.0,
        }
    }
    pub fn reset(&mut self) {
        self.gain = 1.0;
    }
    pub fn ceiling_lin(&self) -> f32 {
        db_to_lin(self.ceiling_db)
    }
    pub fn process(&mut self, buf: &mut [f32], sample_rate: f32) {
        if !self.enabled {
            return;
        }
        let ceil = self.ceiling_lin();
        let rel = attack_release_coef(self.release_ms, sample_rate);
        for s in buf.iter_mut() {
            let a = (s.abs() * self.gain).max(1e-9);
            if a > ceil {
                self.gain = ceil / s.abs().max(1e-9);
            } else {
                // Release back toward unity.
                self.gain += rel * (1.0 - self.gain);
            }
            *s *= self.gain;
            // Hard safety clamp (never exceed ceiling).
            if *s > ceil {
                *s = ceil;
            } else if *s < -ceil {
                *s = -ceil;
            }
        }
    }
}

fn attack_release_coef(ms: f32, sr: f32) -> f32 {
    let t = (ms.max(0.01) / 1000.0) * sr;
    1.0 - (-1.0 / t).exp()
}
