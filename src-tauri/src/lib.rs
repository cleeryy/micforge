//! MicForge Tauri backend.
//!
//! FROZEN command contract (frontend lane codes against this — do not rename):
//! - `list_devices` → `{ inputs: [{id,name}], outputs: [{id,name}] }`
//! - `start{input_id,output_id,preset}` → `{ latency_ms }`
//! - `stop{}` → `{}` (empty object)
//! - `apply_preset{name}` → `{ applied: true }`
//! - `set_block{block,enabled,params}` → `{ ok: true }`
//! - `get_status{}` → `{ running, peak_db, clip, latency_ms }`

use micforge_audio::{AudioDevice, Engine};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

pub struct AppState {
    pub engine: Mutex<Engine>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            engine: Mutex::new(Engine::new()),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

// ------------------------------------------------------------ shapes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceList {
    pub inputs: Vec<AudioDevice>,
    pub outputs: Vec<AudioDevice>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartResult {
    pub latency_ms: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplyResult {
    pub applied: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OkResult {
    pub ok: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Status {
    pub running: bool,
    pub peak_db: f32,
    pub clip: bool,
    pub latency_ms: f32,
}

// ------------------------------------------------------------ logic (unit-testable)
pub fn logic_list_devices() -> DeviceList {
    let (inputs, outputs) = micforge_audio::list_devices();
    DeviceList { inputs, outputs }
}

pub fn logic_start(
    state: &AppState,
    input_id: &str,
    output_id: &str,
    preset: &str,
) -> Result<StartResult, String> {
    let value = resolve_preset(preset)?;
    let mut engine = state.engine.lock().map_err(|e| e.to_string())?;
    let latency_ms = engine.start(input_id, output_id, &value)?;
    Ok(StartResult { latency_ms })
}

pub fn logic_stop(state: &AppState) -> serde_json::Value {
    if let Ok(mut engine) = state.engine.lock() {
        engine.stop();
    }
    serde_json::json!({})
}

pub fn logic_apply_preset(state: &AppState, name: &str) -> Result<ApplyResult, String> {
    let value = resolve_preset(name)?;
    let mut engine = state.engine.lock().map_err(|e| e.to_string())?;
    engine.chain.apply_preset_json(&value)?;
    engine.preset_name = name.to_string();
    Ok(ApplyResult { applied: true })
}

pub fn logic_set_block(
    state: &AppState,
    block: &str,
    enabled: bool,
    params: serde_json::Value,
) -> Result<OkResult, String> {
    let mut engine = state.engine.lock().map_err(|e| e.to_string())?;
    engine.chain.set_block(block, enabled, &params)?;
    Ok(OkResult { ok: true })
}

pub fn logic_get_status(state: &AppState) -> Status {
    match state.engine.lock() {
        Ok(engine) => Status {
            running: engine.running,
            peak_db: engine.peak_db,
            clip: engine.clip,
            latency_ms: engine.total_latency_ms(),
        },
        Err(_) => Status {
            running: false,
            peak_db: f32::NEG_INFINITY,
            clip: false,
            latency_ms: 0.0,
        },
    }
}

/// Resolve a preset reference: a preset name (`blue-yeti`, `blue-yeti.json`),
/// an inline JSON object string, or fallback to the generic builtin.
pub fn resolve_preset(preset: &str) -> Result<serde_json::Value, String> {
    let trimmed = preset.trim();
    // Inline JSON object passes straight through (validated below).
    if trimmed.starts_with('{') {
        let v: serde_json::Value =
            serde_json::from_str(trimmed).map_err(|e| format!("bad preset JSON: {e}"))?;
        // Validate shape early so errors surface on start/apply, not at render.
        micforge_core::Preset::from_json(&v).map_err(|e| format!("bad preset: {e}"))?;
        return Ok(v);
    }
    let stem = trimmed.strip_suffix(".json").unwrap_or(trimmed);
    for candidate in [
        format!("presets/{stem}.json"),
        format!("../presets/{stem}.json"),
        format!("../../presets/{stem}.json"),
    ] {
        let path = std::path::Path::new(&candidate);
        if path.exists() {
            let preset = micforge_core::Preset::from_file(path)?;
            return serde_json::to_value(&preset).map_err(|e| e.to_string());
        }
    }
    // "generic" (or unknown names in tests) fall back to the builtin default.
    let builtin = micforge_core::Preset::builtin_default();
    if stem == "generic" || stem.is_empty() {
        return serde_json::to_value(&builtin).map_err(|e| e.to_string());
    }
    Err(format!(
        "unknown preset '{stem}' (looked in presets/, ../presets/)"
    ))
}

// ------------------------------------------------------------ tauri commands
#[tauri::command]
fn list_devices() -> DeviceList {
    logic_list_devices()
}

#[tauri::command]
fn start(
    state: tauri::State<'_, AppState>,
    input_id: String,
    output_id: String,
    preset: String,
) -> Result<StartResult, String> {
    logic_start(&state, &input_id, &output_id, &preset)
}

#[tauri::command]
fn stop(state: tauri::State<'_, AppState>) -> serde_json::Value {
    logic_stop(&state)
}

#[tauri::command]
fn apply_preset(state: tauri::State<'_, AppState>, name: String) -> Result<ApplyResult, String> {
    logic_apply_preset(&state, &name)
}

#[tauri::command]
fn set_block(
    state: tauri::State<'_, AppState>,
    block: String,
    enabled: bool,
    params: serde_json::Value,
) -> Result<OkResult, String> {
    logic_set_block(&state, &block, enabled, params)
}

#[tauri::command]
fn get_status(state: tauri::State<'_, AppState>) -> Status {
    logic_get_status(&state)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            list_devices,
            start,
            stop,
            apply_preset,
            set_block,
            get_status
        ])
        .run(tauri::generate_context!())
        .expect("error while running MicForge");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn api_list_devices_shape() {
        let d = logic_list_devices();
        let v = serde_json::to_value(&d).unwrap();
        assert!(v.get("inputs").unwrap().is_array());
        assert!(v.get("outputs").unwrap().is_array());
    }

    #[test]
    fn api_start_stop_status_cycle() {
        let s = AppState::new();
        let r = logic_start(&s, "mic", "cable", "generic").unwrap();
        assert!(r.latency_ms > 0.0);
        let st = logic_get_status(&s);
        assert!(st.running);
        assert!(st.latency_ms > 0.0);
        let stop_v = logic_stop(&s);
        assert_eq!(stop_v, serde_json::json!({}));
        assert!(!logic_get_status(&s).running);
    }

    #[test]
    fn api_apply_preset_and_set_block() {
        let s = AppState::new();
        let r = logic_apply_preset(&s, "generic").unwrap();
        assert!(r.applied);
        let r = logic_set_block(&s, "gate", false, serde_json::json!({})).unwrap();
        assert!(r.ok);
        assert!(logic_set_block(&s, "reverb", true, serde_json::json!({})).is_err());
    }

    #[test]
    fn api_start_rejects_unknown_preset() {
        let s = AppState::new();
        assert!(logic_start(&s, "mic", "cable", "no-such-preset").is_err());
    }

    #[test]
    fn api_status_shape_keys() {
        let s = AppState::new();
        let v = serde_json::to_value(logic_get_status(&s)).unwrap();
        for k in ["running", "peak_db", "clip", "latency_ms"] {
            assert!(v.get(k).is_some(), "missing key {k}");
        }
    }
}
