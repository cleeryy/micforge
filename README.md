# MicForge

**Forge any microphone into a studio-ready source — on Windows.** Open-source (AGPL-3.0-only)
real-time microphone processing app: pick any input mic, apply a DSP chain
(noise suppression, gate, compressor, EQ, de-esser, limiter) and route the
processed sound to your apps (Discord, OBS, video calls…). The Blue Yeti is
just one preset among others.

## How it works

- **Rust + Tauri v2** desktop app. Audio capture/render via `cpal`
  (WASAPI on Windows), DSP in pure Rust (`crates/micforge-core`, zero OS deps),
  thin Tauri command bridge in `src-tauri`, UI in `frontend/`.
- Processed audio is played to an **external virtual audio cable** which your
  apps then use as their microphone.
- The virtual cable is **not bundled** with the app: install
  [VB-CABLE](https://vb-audio.com/Cable/) (free, signed driver) separately,
  then select "CABLE Input" as the output in MicForge.

```
Mic [cpal capture 48 kHz mono, 480-frame quanta]
 └─> micforge-core DspChain (fixed order)
       high-pass → noise-suppress → gate → compressor
       → 4-band parametric EQ → de-esser → limiter
      └─> cpal render to virtual cable ─> Discord / OBS / calls
      └─> meters: peak_dB + clip flag ──> frontend via get_status
```

## Prerequisites

- Windows 10/11 (x64) for real use; Linux builds and runs the full test suite.
- [Rust stable](https://rustup.rs/) (1.80+), Node.js 20+ (frontend lane).
- A virtual cable driver installed separately (see note above).

## Commands

```powershell
cargo test                    # unit tests: DSP blocks + engine + Tauri API (Linux OK)
cargo tauri dev               # dev app on Windows (run from repo root)
cargo tauri build             # Windows installer/portable bundle
```

`cargo test` must stay green on Linux (synthetic loopback mode — no hardware needed).

## Cable note

VB-CABLE (or equivalent) is **external, never bundled**: audio drivers cannot
be redistributed from this repo for licensing/signature reasons. Install it
yourself, then pick its input endpoint as MicForge's output.

## Presets

| Preset      | For                        | Notes                                              |
| ----------- | -------------------------- | -------------------------------------------------- |
| `generic`   | Any microphone (default)   | Balanced starting point, +2 dB presence at 3 kHz   |
| `blue-yeti` | Blue Yeti USB mic          | Low physical gain, cardioid pattern, 48 kHz        |
| `laptop`    | Built-in laptop microphones| Stronger high-pass vs fan rumble, soft long gate   |

Preset files live in `presets/*.json` and validate with `python3 -m json.tool`.
The noise-suppression block is currently a pass-through stub; the real neural
stage lands behind the `nnnoiseless` feature flag.

## Layout

```
crates/micforge-core/   # Pure-Rust DSP: blocks + fixed-order chain + presets (TDD)
crates/micforge-audio/  # cpal capture 48 kHz mono → chain → output, synthetic loopback for tests
src-tauri/              # Tauri v2 app: frozen command API (see below), least-privilege capability
frontend/               # UI (other lane — do not touch from the backend)
presets/                # generic / blue-yeti / laptop JSON presets
docs/                   # DESIGN.md (reference), E2E manual (other lane)
```

### Tauri command API (frozen — do not rename)

- `list_devices` → `{ inputs: [{id,name}], outputs: [{id,name}] }`
- `start{input_id,output_id,preset}` → `{ latency_ms }`
- `stop{}` → `{}`
- `apply_preset{name}` → `{ applied: true }`
- `set_block{block,enabled,params}` → `{ ok: true }`
- `get_status{}` → `{ running, peak_db, clip, latency_ms }`

## License

AGPL-3.0-only — see `LICENSE`. Copyright (c) 2026 cleeryy.
