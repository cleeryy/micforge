# Virtual audio cable — Installation (VB-CABLE)

Yeti Voice bundles **no audio driver** (deliberate + legal choice).
The processed sound goes out to a virtual cable that your apps use as a microphone.

## Installing VB-CABLE (external)

1. Download VB-CABLE from the official site: <https://vb-audio.com/Cable/>
2. Unzip, right-click `VBCABLE_Setup_x64.exe` → **Run as administrator**.
3. Reboot if asked.
4. Verify: Sound settings → Playback → "CABLE Input", Recording → "CABLE Output".

## Detection in Yeti Voice

- On startup, the app lists WASAPI outputs: pick **"CABLE Input (VB-Audio Virtual Cable)"**.
- **Rescan** button: re-lists devices (handy after installing the driver
  or reconnecting the Yeti over USB without restarting the app).

## VAC fallback

If VB-CABLE does not fit (latency, multi-cable needs), any alternative
exposing a WASAPI playback device works:
[Virtual Audio Cable](https://vac.muzychenko.net/) (paid),
[Voicemeeter](https://vb-audio.com/Voicemeeter/) (same vendor).
Just select its input as the output in the app.

## Typical routing

```
Blue Yeti ──WASAPI──▶ Yeti Voice (DSP) ──WASAPI──▶ CABLE Input
                                                       │
Discord/OBS: mic = "CABLE Output" ◀───────────────────┘
```
