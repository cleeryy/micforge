# Yeti Voice

**Tune your Blue Yeti 100% on Windows.** Open-source (AGPL-3.0-only)
real-time microphone processing app: plug in your Blue Yeti, apply a DSP chain
(noise reduction, gate, compressor, EQ, de-esser, limiter) and route the
processed sound to your apps (Discord, OBS, video calls…).

## How it works

- 100% **user-mode** app: WASAPI capture of the mic via [NAudio](https://github.com/naudio/NAudio),
  DSP processing in C# (.NET 8), playback to an **external virtual audio cable**.
- The virtual cable is **not bundled** with the app: install
  [VB-CABLE](https://vb-audio.com/Cable/) (free, signed driver) separately,
  then select "CABLE Input" as the output in Yeti Voice.
- See `docs/SETUP-CABLE.md` for installation and `docs/BLUE-YETI.md` for the
  physical mic settings.

## Prerequisites

- Windows 10/11 (x64)
- [.NET 8 SDK](https://dotnet.microsoft.com/download/dotnet/8.0) (to build)

## Build / Portable publish

```powershell
dotnet build yeti-voice.sln
dotnet publish src/BlueYeti.App/BlueYeti.App.csproj -c Release -r win-x64 --self-contained -o publish
```

The `publish/` folder is portable: copy it anywhere, no installer required.
Optional Inno Setup installer: see `installer/setup.iss`.

## MVP scope

- [x] Input (Blue Yeti) / output (virtual cable) selection, device re-scan
- [x] DSP chain: high-pass, RNNoise, noise gate, compressor, 4-band EQ, de-esser, limiter
- [x] JSON presets: `yeti-clear`, `yeti-stream`, `yeti-meeting`
- [x] Real-time peak meter
- [ ] Native RNNoise (currently stub — TODO)
- [ ] Auto-gain, latency monitoring

## Out of scope

- Redistributing a virtual cable / driver (forbidden: the user installs it themselves)
- macOS / Linux (WASAPI is Windows-only)
- Voice effects (pitch, robot) — corrective DSP only

## Layout

```
src/BlueYeti.Core/    # Audio engine + DSP chain + models (net8.0, testable)
src/BlueYeti.App/     # WPF UI (net8.0-windows): selectors, peak meter
tests/BlueYeti.Core.Tests/  # xunit tests for the Core
presets/              # JSON presets shipped with the app
docs/                 # DESIGN.md (mockup), BLUE-YETI.md, SETUP-CABLE.md
installer/            # Inno Setup script
```

## License

AGPL-3.0-only — see `LICENSE`. Copyright (c) 2026 cleeryy.
