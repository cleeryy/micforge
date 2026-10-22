//! Audio engine: capture 48 kHz mono → [`DspChain`] → selected output.
//!
//! On production paths this uses cpal streams; in tests (and headless Linux CI)
//! the synthetic loopback path exercises the same chain without hardware.

use cpal::traits::{DeviceTrait, HostTrait};
use micforge_core::{DspChain, Preset};
use serde::{Deserialize, Serialize};

pub const SAMPLE_RATE: u32 = 48000;
pub const CHANNELS: u16 = 1;
/// Render quantum in frames; 480 frames @ 48 kHz = 10 ms, 960 = 20 ms.
pub const FRAMES: usize = 480;
pub const MAX_FRAMES: usize = 960;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioDevice {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct Engine {
    pub chain: DspChain,
    pub running: bool,
    pub input_id: String,
    pub output_id: String,
    pub preset_name: String,
    pub peak_db: f32,
    pub clip: bool,
    /// Extra device-buffer latency on top of [`DspChain::latency_estimate_ms`].
    pub buffer_latency_ms: f32,
}

impl Engine {
    pub fn new() -> Self {
        Self {
            chain: DspChain::new(SAMPLE_RATE as f32),
            running: false,
            input_id: String::new(),
            output_id: String::new(),
            preset_name: "generic".into(),
            peak_db: f32::NEG_INFINITY,
            clip: false,
            buffer_latency_ms: FRAMES as f32 / SAMPLE_RATE as f32 * 1000.0,
        }
    }

    pub fn total_latency_ms(&self) -> f32 {
        self.chain.latency_estimate_ms() + self.buffer_latency_ms
    }

    /// Start processing with the given preset JSON value.
    pub fn start(
        &mut self,
        input_id: &str,
        output_id: &str,
        preset: &serde_json::Value,
    ) -> Result<f32, String> {
        let name = self.chain.apply_preset_json(preset)?;
        self.input_id = input_id.to_string();
        self.output_id = output_id.to_string();
        self.preset_name = name;
        self.running = true;
        self.peak_db = f32::NEG_INFINITY;
        self.clip = false;
        // Real cpal stream hookup happens in the Tauri runtime (Windows).
        // On Linux CI / headless, the engine runs in synthetic loopback mode.
        Ok(self.total_latency_ms())
    }

    pub fn start_with_preset(&mut self, input_id: &str, output_id: &str, preset: &Preset) -> f32 {
        let v = serde_json::to_value(preset).expect("preset serializes");
        self.start(input_id, output_id, &v).expect("builtin preset applies")
    }

    pub fn stop(&mut self) {
        self.running = false;
    }

    /// Process one quantum through the chain, tracking peak/clip meters.
    /// Used by the real stream callback and by the synthetic loopback tests.
    pub fn render(&mut self, buf: &mut [f32]) {
        assert!(
            buf.len() <= MAX_FRAMES,
            "quantum {} exceeds max {MAX_FRAMES}",
            buf.len()
        );
        self.chain.process(buf);
        let peak = buf.iter().map(|x| x.abs()).fold(0.0, f32::max);
        if peak > 0.0 {
            let db = 20.0 * peak.log10();
            if db > self.peak_db {
                self.peak_db = db;
            }
        }
        if peak >= 0.999 {
            self.clip = true;
        }
    }

    /// Synthetic loopback: process `input` in [`FRAMES`]-sized quanta.
    pub fn process_synthetic(&mut self, input: &[f32]) -> Vec<f32> {
        let mut out = Vec::with_capacity(input.len());
        for chunk in input.chunks(FRAMES) {
            let mut buf = chunk.to_vec();
            self.render(&mut buf);
            out.extend_from_slice(&buf);
        }
        out
    }

    pub fn clear_clip(&mut self) {
        self.clip = false;
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}

/// Enumerate capture/render devices. Never fails: returns empty lists when
/// no host backend is available (headless CI), so the UI can show its
/// "device missing" states instead of crashing.
pub fn list_devices() -> (Vec<AudioDevice>, Vec<AudioDevice>) {
    let mut inputs = Vec::new();
    let mut outputs = Vec::new();
    let host = cpal::default_host();
    if let Ok(devs) = host.input_devices() {
        for d in devs {
            let name = d.name().unwrap_or_else(|_| "Unknown input".into());
            inputs.push(AudioDevice { id: name.clone(), name });
        }
    }
    if let Ok(devs) = host.output_devices() {
        for d in devs {
            let name = d.name().unwrap_or_else(|_| "Unknown output".into());
            outputs.push(AudioDevice { id: name.clone(), name });
        }
    }
    (inputs, outputs)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn preset_value() -> serde_json::Value {
        serde_json::to_value(Preset::builtin_default()).unwrap()
    }

    #[test]
    fn start_stop_transitions() {
        let mut e = Engine::new();
        assert!(!e.running);
        let lat = e.start("mic", "cable", &preset_value()).unwrap();
        assert!(e.running);
        assert!(lat > 0.0);
        assert_eq!(e.preset_name, "generic");
        e.stop();
        assert!(!e.running);
    }

    #[test]
    fn start_rejects_bad_preset() {
        let mut e = Engine::new();
        let bad = serde_json::json!({"version": 1});
        assert!(e.start("mic", "cable", &bad).is_err());
        assert!(!e.running);
    }

    #[test]
    fn synthetic_loopback_round_trip() {
        let mut e = Engine::new();
        e.start_with_preset("mic", "cable", &Preset::builtin_default());
        let input: Vec<f32> = (0..4800)
            .map(|i| 0.3 * (2.0 * std::f32::consts::PI * 440.0 * i as f32 / 48000.0).sin())
            .collect();
        let out = e.process_synthetic(&input);
        assert_eq!(out.len(), input.len());
        assert!(out.iter().any(|x| x.abs() > 0.01), "voice should pass through");
        assert!(e.peak_db > -60.0, "peak meter should track");
    }

    #[test]
    fn quantum_size_within_bounds() {
        assert!(FRAMES >= 480 && FRAMES <= MAX_FRAMES);
        assert_eq!(MAX_FRAMES, 960);
    }

    #[test]
    fn clip_flag_on_hot_signal() {
        let mut e = Engine::new();
        // Limiter disabled so the hot signal actually clips the meter.
        let mut v = preset_value();
        v["limiter"]["enabled"] = serde_json::json!(false);
        v["gate"]["enabled"] = serde_json::json!(false);
        v["comp"]["enabled"] = serde_json::json!(false);
        e.start("mic", "cable", &v).unwrap();
        let mut buf = vec![1.5f32; FRAMES];
        e.render(&mut buf);
        assert!(e.clip);
    }

    #[test]
    fn list_devices_never_panics() {
        let (ins, outs) = list_devices();
        // Headless CI may yield zero devices; just assert it returns.
        let _ = (ins.len(), outs.len());
    }
}
