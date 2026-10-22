# Manual end-to-end test (Windows-first, any mic)

> Why manual? Automated end-to-end with a real microphone is **impossible
> in cloud CI**: GitHub-hosted runners have no audio hardware, no
> microphone, and no virtual audio cable. CI therefore proves only that
> the code builds, the Rust TDD suite passes, clippy is clean, and
> `presets/*.json` parse (see `.github/workflows/ci.yml`). **This
> document plus the TDD suite is the E2E path** — the full signal chain
> below must be run by hand on a real PC with a real mic.

## 0. What CI proves vs what you must test

| CI (automatic) | You, tomorrow (manual, this doc) |
|---|---|
| `cargo test --workspace` passes on Ubuntu + Windows | Real mic signal flows through the app |
| `cargo clippy -- -D warnings` clean | Voice sounds good in Discord/Teams |
| Presets parse as JSON | Presets sound different and sane |
| Tauri bundle builds on Windows | Installer/artifact runs on your PC |

## 1. Requirements

- Windows 10/11 (x64). Linux works for dev, but this checklist is Windows-first.
- Any USB/XLR microphone (a Blue Yeti is the reference, but nothing here is Yeti-specific).
- Rust stable toolchain + Node.js LTS (for `cargo tauri dev`), **or** the
  `MicForge-bundle-windows` artifact from the last green CI run.
- WebView2 runtime (preinstalled on Windows 11; on Windows 10 install it
  from Microsoft if the app window stays blank).
- VB-CABLE virtual audio driver (the app bundles **no driver**):
  run `VBCABLE_Setup_x64.exe` as administrator, reboot if asked.
  Playback tab should then show **CABLE Input**, Recording tab **CABLE Output**.

## 2. Force 48 kHz everywhere

Mismatched rates force pointless resampling in the DSP chain:

- [ ] Recording → your mic → Properties → Advanced: **48 000 Hz / 16-bit**.
- [ ] Playback → CABLE Input → Properties → Advanced: **48 000 Hz**.
- [ ] Recording → CABLE Output → Properties → Advanced: **48 000 Hz**.

## 3. Physical mic setup

- [ ] Gain low to start; raise only until normal speech peaks around
      **−12 dBFS** on the app's peak meter.
- [ ] Cardioid/polar pattern facing you if your mic has a selector
      (on a Blue Yeti: rear switch on cardioid, speak into the Blue logo
      side, 15–20 cm, slightly off-axis).
- [ ] Quiet room, mic off the desk or on a shock mount if possible.

## 4. Run the app

From the repo root:

```powershell
cargo tauri dev
```

Or install/launch the CI bundle artifact.

- [ ] **Device select**: input selector lists your physical mic, output
      selector lists **CABLE Input (VB-Audio Virtual Cable)**.
- [ ] If a device is missing, use **Rescan** (handy after installing the
      cable driver or replugging the mic).

## 5. Five-second mic test

- [ ] Select mic → CABLE Input, start processing.
- [ ] Speak normally for 5 s: peak meter moves, peaks ≈ −12 dBFS,
      no red clip indicator stuck on.
- [ ] Listen via CABLE Output (Sound settings → Recording → CABLE
      Output → Listen → play through your headphones): voice is clear,
      no stutter, gate opens/closes naturally on pauses.

## 6. Discord / Teams verification

- [ ] Discord → Voice settings → input device = **CABLE Output**;
      mic test bar moves when you speak.
- [ ] Same check in Teams/Zoom: input = **CABLE Output**.
- [ ] Ask a contact (or record a clip): no echo, no robotic artefacts.

## 7. Clip / latency sanity

- [ ] Shout once: limiter catches it (no harsh digital clipping),
      meter recovers immediately.
- [ ] Clap while monitoring: perceived delay stays conversational
      (WASAPI shared mode; exact figures depend on the machine).
- [ ] Silent room for 10 s: no hiss build-up, no gate chatter.

## 8. Preset switching

- [ ] Load each preset (`presets/*.json`): processing restarts without
      error, sound changes audibly (presence, gate tightness).
- [ ] Corrupt preset (e.g. empty `{}`) is rejected with a clear error,
      app keeps running on the previous preset.

## 9. Simple vs Pro views

- [ ] **Simple view**: a non-technical user can pick mic → cable,
      hit start, and see the level meter without touching anything else.
- [ ] **Pro view**: every DSP parameter from the loaded preset is
      reachable and adjustable; changes apply live without restart.

## Sign-off

All boxes ticked on (date / PC / mic model): _______________.
File issues with preset name + Windows build + mic model + log excerpt.
