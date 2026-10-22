//! Preset model: JSON-serializable DSP settings + notes.

use crate::blocks::EqBand;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HighPassCfg {
    pub enabled: bool,
    #[serde(rename = "frequencyHz", default = "default_hp")]
    pub frequency_hz: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoiseCfg {
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateCfg {
    pub enabled: bool,
    #[serde(rename = "thresholdDb")]
    pub threshold_db: f32,
    #[serde(rename = "attackMs")]
    pub attack_ms: f32,
    #[serde(rename = "releaseMs")]
    pub release_ms: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompCfg {
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EqCfg {
    pub enabled: bool,
    pub bands: [EqBand; 4],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeEsserCfg {
    pub enabled: bool,
    #[serde(rename = "frequencyHz")]
    pub frequency_hz: f32,
    #[serde(rename = "thresholdDb")]
    pub threshold_db: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LimiterCfg {
    pub enabled: bool,
    #[serde(rename = "ceilingDb")]
    pub ceiling_db: f32,
    #[serde(rename = "releaseMs")]
    pub release_ms: f32,
}

fn default_hp() -> f32 {
    80.0
}

/// A full DSP preset. The `noise` section accepts the legacy `rnnoise` key.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Preset {
    pub version: u32,
    pub name: String,
    pub highpass: HighPassCfg,
    #[serde(alias = "rnnoise")]
    pub noise: NoiseCfg,
    pub gate: GateCfg,
    pub comp: CompCfg,
    pub eq: EqCfg,
    pub deesser: DeEsserCfg,
    pub limiter: LimiterCfg,
    #[serde(default)]
    pub notes: String,
}

impl Preset {
    pub fn from_json(v: &serde_json::Value) -> Result<Self, serde_json::Error> {
        serde_json::from_value(v.clone())
    }

    pub fn from_file(path: &std::path::Path) -> Result<Self, String> {
        let text = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
        serde_json::from_str(&text).map_err(|e| e.to_string())
    }

    pub fn builtin_default() -> Self {
        serde_json::from_value(serde_json::json!({
            "version": 1,
            "name": "generic",
            "highpass": { "enabled": true, "frequencyHz": 80 },
            "noise": { "enabled": true },
            "gate": { "enabled": true, "thresholdDb": -40.0, "attackMs": 5.0, "releaseMs": 120.0 },
            "comp": { "enabled": true, "thresholdDb": -18.0, "ratio": 3.0, "attackMs": 10.0, "releaseMs": 150.0, "makeupDb": 4.0 },
            "eq": { "enabled": true, "bands": [
                { "frequencyHz": 120.0, "gainDb": -2.0, "q": 1.0 },
                { "frequencyHz": 500.0, "gainDb": 0.0, "q": 1.0 },
                { "frequencyHz": 3000.0, "gainDb": 2.0, "q": 1.0 },
                { "frequencyHz": 8000.0, "gainDb": -1.0, "q": 1.0 }
            ]},
            "deesser": { "enabled": true, "frequencyHz": 6500.0, "thresholdDb": -24.0 },
            "limiter": { "enabled": true, "ceilingDb": -1.0, "releaseMs": 50.0 },
            "notes": "Generic starting point for any microphone."
        }))
        .expect("builtin default preset is valid")
    }
}

impl PartialEq for HighPassCfg {
    fn eq(&self, o: &Self) -> bool {
        self.enabled == o.enabled && (self.frequency_hz - o.frequency_hz).abs() < 1e-6
    }
}
impl PartialEq for NoiseCfg {
    fn eq(&self, o: &Self) -> bool {
        self.enabled == o.enabled
    }
}
impl PartialEq for GateCfg {
    fn eq(&self, o: &Self) -> bool {
        self.enabled == o.enabled
            && (self.threshold_db - o.threshold_db).abs() < 1e-6
            && (self.attack_ms - o.attack_ms).abs() < 1e-6
            && (self.release_ms - o.release_ms).abs() < 1e-6
    }
}
impl PartialEq for CompCfg {
    fn eq(&self, o: &Self) -> bool {
        self.enabled == o.enabled
            && (self.threshold_db - o.threshold_db).abs() < 1e-6
            && (self.ratio - o.ratio).abs() < 1e-6
            && (self.attack_ms - o.attack_ms).abs() < 1e-6
            && (self.release_ms - o.release_ms).abs() < 1e-6
            && (self.makeup_db - o.makeup_db).abs() < 1e-6
    }
}
impl PartialEq for EqCfg {
    fn eq(&self, o: &Self) -> bool {
        self.enabled == o.enabled
            && self.bands.iter().zip(o.bands.iter()).all(|(a, b)| {
                (a.frequency_hz - b.frequency_hz).abs() < 1e-6
                    && (a.gain_db - b.gain_db).abs() < 1e-6
                    && (a.q - b.q).abs() < 1e-6
            })
    }
}
impl PartialEq for DeEsserCfg {
    fn eq(&self, o: &Self) -> bool {
        self.enabled == o.enabled
            && (self.frequency_hz - o.frequency_hz).abs() < 1e-6
            && (self.threshold_db - o.threshold_db).abs() < 1e-6
    }
}
impl PartialEq for LimiterCfg {
    fn eq(&self, o: &Self) -> bool {
        self.enabled == o.enabled
            && (self.ceiling_db - o.ceiling_db).abs() < 1e-6
            && (self.release_ms - o.release_ms).abs() < 1e-6
    }
}
