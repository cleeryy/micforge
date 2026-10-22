//! Fixed-order DSP chain with per-block enable and preset apply.

use crate::blocks::EqBand;
use crate::{Compressor, DeEsser, Gate, HighPass, Limiter, NoiseSuppress, ParametricEq};
use crate::preset::Preset;

/// Fixed processing order: HP → NS → Gate → Comp → EQ → DeEsser → Limiter.
#[derive(Debug, Clone)]
pub struct DspChain {
    pub sample_rate: f32,
    pub hp: HighPass,
    pub ns: NoiseSuppress,
    pub gate: Gate,
    pub comp: Compressor,
    pub eq: ParametricEq,
    pub deesser: DeEsser,
    pub limiter: Limiter,
}

impl DspChain {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            sample_rate,
            hp: HighPass::new(80.0),
            ns: NoiseSuppress::new(),
            gate: Gate::new(-40.0, 5.0, 120.0),
            comp: Compressor::new(-18.0, 3.0, 10.0, 150.0, 4.0),
            eq: ParametricEq::new(
                [
                    EqBand { frequency_hz: 120.0, gain_db: -2.0, q: 1.0 },
                    EqBand { frequency_hz: 500.0, gain_db: 0.0, q: 1.0 },
                    EqBand { frequency_hz: 3000.0, gain_db: 2.0, q: 1.0 },
                    EqBand { frequency_hz: 8000.0, gain_db: -1.0, q: 1.0 },
                ],
                sample_rate,
            ),
            deesser: DeEsser::new(6500.0, -24.0),
            limiter: Limiter::new(-1.0, 50.0),
        }
    }

    pub fn reset(&mut self) {
        self.hp.reset();
        self.gate.reset();
        self.comp.reset(self.sample_rate);
        self.eq.rebuild(self.sample_rate);
        self.deesser.reset();
        self.limiter.reset();
    }

    pub fn process(&mut self, buf: &mut [f32]) {
        let sr = self.sample_rate;
        self.hp.process(buf, sr);
        self.ns.process(buf, sr);
        self.gate.process(buf, sr);
        self.comp.process(buf, sr);
        self.eq.process(buf, sr);
        self.deesser.process(buf, sr);
        self.limiter.process(buf, sr);
    }

    /// Apply a preset: copies every block's params + enable flags, resets state.
    pub fn apply_preset(&mut self, p: &Preset) {
        self.hp.enabled = p.highpass.enabled;
        self.hp.frequency_hz = p.highpass.frequency_hz;
        self.ns.enabled = p.noise.enabled;
        self.gate.enabled = p.gate.enabled;
        self.gate.threshold_db = p.gate.threshold_db;
        self.gate.attack_ms = p.gate.attack_ms;
        self.gate.release_ms = p.gate.release_ms;
        self.comp.enabled = p.comp.enabled;
        self.comp.threshold_db = p.comp.threshold_db;
        self.comp.ratio = p.comp.ratio.max(1.0);
        self.comp.attack_ms = p.comp.attack_ms;
        self.comp.release_ms = p.comp.release_ms;
        self.comp.makeup_db = p.comp.makeup_db;
        self.eq.enabled = p.eq.enabled;
        self.eq.bands = p.eq.bands;
        self.eq.rebuild(self.sample_rate);
        self.deesser.enabled = p.deesser.enabled;
        self.deesser.frequency_hz = p.deesser.frequency_hz;
        self.deesser.threshold_db = p.deesser.threshold_db;
        self.limiter.enabled = p.limiter.enabled;
        self.limiter.ceiling_db = p.limiter.ceiling_db;
        self.limiter.release_ms = p.limiter.release_ms;
        self.reset();
    }

    pub fn apply_preset_json(&mut self, v: &serde_json::Value) -> Result<String, String> {
        let p = Preset::from_json(v).map_err(|e| e.to_string())?;
        let name = p.name.clone();
        self.apply_preset(&p);
        Ok(name)
    }

    /// Rough DSP-only latency estimate in ms (excludes device buffers).
    pub fn latency_estimate_ms(&self) -> f32 {
        let mut ms = 0.1 + 1.0 + 1.0 + 0.5 + 1.0 + 1.0; // hp+gate+comp+eq+deesser+limiter
        if self.ns.enabled {
            ms += 10.0; // frame-based suppressor hop
        }
        ms
    }

    /// Enable/disable (and optionally retune) a single block by name.
    /// Block names: highpass, noise, gate, comp, eq, deesser, limiter.
    pub fn set_block(
        &mut self,
        block: &str,
        enabled: bool,
        params: &serde_json::Value,
    ) -> Result<(), String> {
        let get_f = |key: &str, cur: f32| {
            params
                .get(key)
                .and_then(|v| v.as_f64())
                .map(|v| v as f32)
                .unwrap_or(cur)
        };
        match block {
            "highpass" => {
                self.hp.enabled = enabled;
                self.hp.frequency_hz = get_f("frequencyHz", self.hp.frequency_hz);
            }
            "noise" | "rnnoise" => {
                self.ns.enabled = enabled;
            }
            "gate" => {
                self.gate.enabled = enabled;
                self.gate.threshold_db = get_f("thresholdDb", self.gate.threshold_db);
                self.gate.attack_ms = get_f("attackMs", self.gate.attack_ms);
                self.gate.release_ms = get_f("releaseMs", self.gate.release_ms);
            }
            "comp" => {
                self.comp.enabled = enabled;
                self.comp.threshold_db = get_f("thresholdDb", self.comp.threshold_db);
                self.comp.ratio = get_f("ratio", self.comp.ratio).max(1.0);
                self.comp.attack_ms = get_f("attackMs", self.comp.attack_ms);
                self.comp.release_ms = get_f("releaseMs", self.comp.release_ms);
                self.comp.makeup_db = get_f("makeupDb", self.comp.makeup_db);
            }
            "eq" => {
                self.eq.enabled = enabled;
                if let Some(bands) = params.get("bands").and_then(|v| v.as_array()) {
                    for (i, b) in bands.iter().take(4).enumerate() {
                        if let Some(f) = b.get("frequencyHz").and_then(|v| v.as_f64()) {
                            self.eq.bands[i].frequency_hz = f as f32;
                        }
                        if let Some(g) = b.get("gainDb").and_then(|v| v.as_f64()) {
                            self.eq.bands[i].gain_db = g as f32;
                        }
                        if let Some(q) = b.get("q").and_then(|v| v.as_f64()) {
                            self.eq.bands[i].q = q as f32;
                        }
                    }
                    self.eq.rebuild(self.sample_rate);
                }
            }
            "deesser" => {
                self.deesser.enabled = enabled;
                self.deesser.frequency_hz = get_f("frequencyHz", self.deesser.frequency_hz);
                self.deesser.threshold_db = get_f("thresholdDb", self.deesser.threshold_db);
            }
            "limiter" => {
                self.limiter.enabled = enabled;
                self.limiter.ceiling_db = get_f("ceilingDb", self.limiter.ceiling_db);
                self.limiter.release_ms = get_f("releaseMs", self.limiter.release_ms);
            }
            other => return Err(format!("unknown block: {other}")),
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::preset::Preset;

    const SR: f32 = 48000.0;

    fn sine(freq: f32, amp: f32, n: usize) -> Vec<f32> {
        (0..n)
            .map(|i| amp * (2.0 * std::f32::consts::PI * freq * i as f32 / SR).sin())
            .collect()
    }
    fn rms(v: &[f32]) -> f32 {
        (v.iter().map(|x| x * x).sum::<f32>() / v.len().max(1) as f32).sqrt()
    }
    fn peak(v: &[f32]) -> f32 {
        v.iter().map(|x| x.abs()).fold(0.0, f32::max)
    }

    #[test]
    fn limiter_enforces_ceiling() {
        let mut lim = Limiter::new(-1.0, 50.0);
        let mut buf = vec![2.0f32; 4800];
        lim.process(&mut buf, SR);
        let ceil = 10.0f32.powf(-1.0 / 20.0);
        assert!(
            peak(&buf) <= ceil + 1e-4,
            "peak {} exceeds ceiling {ceil}",
            peak(&buf)
        );
    }

    #[test]
    fn limiter_passthrough_when_disabled() {
        let mut lim = Limiter::new(-1.0, 50.0);
        lim.enabled = false;
        let mut buf = vec![2.0f32; 480];
        lim.process(&mut buf, SR);
        assert!((peak(&buf) - 2.0).abs() < 1e-6);
    }

    #[test]
    fn gate_silences_quiet_signal() {
        let mut g = Gate::new(-40.0, 5.0, 120.0);
        let mut buf = sine(440.0, 0.001, 4800); // ≈ -60 dB, below threshold
        g.process(&mut buf, SR);
        let tail = &buf[2400..];
        assert!(
            rms(tail) < 0.0005,
            "gate should close on quiet signal, rms={}",
            rms(tail)
        );
    }

    #[test]
    fn gate_opens_on_loud_signal() {
        let mut g = Gate::new(-40.0, 5.0, 120.0);
        let mut buf = sine(440.0, 0.5, 4800);
        g.process(&mut buf, SR);
        let tail = &buf[2400..];
        assert!(
            rms(tail) > 0.2,
            "gate should open on loud signal, rms={}",
            rms(tail)
        );
    }

    #[test]
    fn gate_pure_silence_stays_silent() {
        let mut g = Gate::new(-40.0, 5.0, 120.0);
        let mut buf = vec![0.0f32; 4800];
        g.process(&mut buf, SR);
        assert!(peak(&buf) == 0.0);
    }

    #[test]
    fn compressor_applies_ratio() {
        // Steady 0 dBFS DC: thr -18 dB, ratio 3 → out ≈ -12 dB (0.251 lin), makeup 0.
        let mut c = Compressor::new(-18.0, 3.0, 1.0, 50.0, 0.0);
        let mut buf = vec![1.0f32; 48000];
        c.process(&mut buf, SR);
        let tail_peak = peak(&buf[24000..]);
        let expected = 10.0f32.powf(-12.0 / 20.0);
        assert!(
            (tail_peak - expected).abs() < 0.05,
            "peak {tail_peak} != expected {expected}"
        );
    }

    #[test]
    fn compressor_boosts_with_makeup() {
        let mut c = Compressor::new(-18.0, 3.0, 1.0, 50.0, 6.0);
        let mut buf = vec![1.0f32; 48000];
        c.process(&mut buf, SR);
        let tail_peak = peak(&buf[24000..]);
        let expected = 10.0f32.powf((-12.0 + 6.0) / 20.0);
        assert!((tail_peak - expected).abs() < 0.06);
    }

    #[test]
    fn eq_cut_attenuates_stop_band() {
        let mk = || {
            ParametricEq::new(
                [
                    EqBand { frequency_hz: 120.0, gain_db: 0.0, q: 1.0 },
                    EqBand { frequency_hz: 500.0, gain_db: 0.0, q: 1.0 },
                    EqBand { frequency_hz: 3000.0, gain_db: 0.0, q: 1.0 },
                    EqBand { frequency_hz: 8000.0, gain_db: -12.0, q: 1.0 },
                ],
                SR,
            )
        };
        let input = sine(8000.0, 0.5, 48000);
        let dry = rms(&input[24000..]);
        let mut wet = input.clone();
        mk().process(&mut wet, SR);
        let wet_rms = rms(&wet[24000..]);
        let att_db = 20.0 * (wet_rms / dry).log10();
        assert!(att_db < -6.0, "expected >6 dB cut, got {att_db:.1} dB");
    }

    #[test]
    fn eq_flat_is_transparent() {
        let mut eq = ParametricEq::new(
            [
                EqBand { frequency_hz: 120.0, gain_db: 0.0, q: 1.0 },
                EqBand { frequency_hz: 500.0, gain_db: 0.0, q: 1.0 },
                EqBand { frequency_hz: 3000.0, gain_db: 0.0, q: 1.0 },
                EqBand { frequency_hz: 8000.0, gain_db: 0.0, q: 1.0 },
            ],
            SR,
        );
        let mut buf = sine(1000.0, 0.5, 4800);
        let before = rms(&buf);
        eq.process(&mut buf, SR);
        let after = rms(&buf[2400..]);
        let diff_db = (20.0 * (after / before).log10()).abs();
        assert!(diff_db < 1.0, "flat EQ should be ~transparent ({diff_db:.2} dB)");
    }

    #[test]
    fn highpass_removes_dc() {
        let mut hp = HighPass::new(80.0);
        let mut buf = vec![1.0f32; 48000];
        hp.process(&mut buf, SR);
        assert!(
            buf.last().unwrap().abs() < 0.05,
            "DC should decay, got {}",
            buf.last().unwrap()
        );
    }

    #[test]
    fn deesser_tames_hf_spares_lf() {
        let mut ds = DeEsser::new(6500.0, -24.0);
        let mut hf = sine(8000.0, 0.5, 24000);
        ds.process(&mut hf, SR);
        let hf_ratio = rms(&hf[12000..]) / 0.5 * std::f32::consts::SQRT_2;
        let mut ds2 = DeEsser::new(6500.0, -24.0);
        let mut lf = sine(300.0, 0.5, 24000);
        ds2.process(&mut lf, SR);
        let lf_ratio = rms(&lf[12000..]) / 0.5 * std::f32::consts::SQRT_2;
        assert!(hf_ratio < 0.9, "HF should be ducked ({hf_ratio:.2})");
        assert!(lf_ratio > 0.95, "LF should pass ({lf_ratio:.2})");
    }

    #[test]
    fn chain_order_matches_manual_sequence() {
        // With a fixed order, chain output must equal manual block-by-block processing.
        let mut chain = DspChain::new(SR);
        let preset = Preset::builtin_default();
        chain.apply_preset(&preset);
        let input = sine(1000.0, 0.4, 4800);
        let mut via_chain = input.clone();
        chain.process(&mut via_chain);

        let mut chain2 = DspChain::new(SR);
        chain2.apply_preset(&preset);
        let mut manual = input.clone();
        let sr = chain2.sample_rate;
        chain2.hp.process(&mut manual, sr);
        chain2.ns.process(&mut manual, sr);
        chain2.gate.process(&mut manual, sr);
        chain2.comp.process(&mut manual, sr);
        chain2.eq.process(&mut manual, sr);
        chain2.deesser.process(&mut manual, sr);
        chain2.limiter.process(&mut manual, sr);
        for (a, b) in via_chain.iter().zip(manual.iter()) {
            assert!((a - b).abs() < 1e-6);
        }
    }

    #[test]
    fn preset_round_trip() {
        let p = Preset::builtin_default();
        let v = serde_json::to_value(&p).unwrap();
        let q = Preset::from_json(&v).unwrap();
        assert_eq!(p, q);
    }

    #[test]
    fn preset_legacy_rnnoise_key() {
        let v = serde_json::json!({
            "version": 1, "name": "legacy",
            "highpass": { "enabled": true, "frequencyHz": 80 },
            "rnnoise": { "enabled": false },
            "gate": { "enabled": false, "thresholdDb": -40.0, "attackMs": 5.0, "releaseMs": 120.0 },
            "comp": { "enabled": false, "thresholdDb": -18.0, "ratio": 3.0, "attackMs": 10.0, "releaseMs": 150.0, "makeupDb": 0.0 },
            "eq": { "enabled": false, "bands": [
                { "frequencyHz": 120.0, "gainDb": 0.0, "q": 1.0 },
                { "frequencyHz": 500.0, "gainDb": 0.0, "q": 1.0 },
                { "frequencyHz": 3000.0, "gainDb": 0.0, "q": 1.0 },
                { "frequencyHz": 8000.0, "gainDb": 0.0, "q": 1.0 } ]},
            "deesser": { "enabled": false, "frequencyHz": 6500.0, "thresholdDb": -24.0 },
            "limiter": { "enabled": false, "ceilingDb": -1.0, "releaseMs": 50.0 }
        });
        let p = Preset::from_json(&v).unwrap();
        assert!(!p.noise.enabled);
    }

    #[test]
    fn preset_apply_sets_enable_flags() {
        let mut chain = DspChain::new(SR);
        let mut v = serde_json::to_value(Preset::builtin_default()).unwrap();
        v["gate"]["enabled"] = serde_json::json!(false);
        v["limiter"]["enabled"] = serde_json::json!(false);
        chain.apply_preset_json(&v).unwrap();
        assert!(!chain.gate.enabled);
        assert!(!chain.limiter.enabled);
        assert!(chain.comp.enabled);
    }

    #[test]
    fn latency_estimate_grows_with_noise_block() {
        let mut chain = DspChain::new(SR);
        chain.ns.enabled = false;
        let off = chain.latency_estimate_ms();
        chain.ns.enabled = true;
        let on = chain.latency_estimate_ms();
        assert!(off > 0.0 && on > off, "off={off} on={on}");
    }

    #[test]
    fn set_block_unknown_name_errors() {
        let mut chain = DspChain::new(SR);
        assert!(chain.set_block("reverb", true, &serde_json::json!({})).is_err());
    }

    #[test]
    fn set_block_toggles_and_tunes() {
        let mut chain = DspChain::new(SR);
        chain
            .set_block("gate", false, &serde_json::json!({"thresholdDb": -30.0}))
            .unwrap();
        assert!(!chain.gate.enabled);
        assert!((chain.gate.threshold_db - -30.0).abs() < 1e-6);
    }
}
