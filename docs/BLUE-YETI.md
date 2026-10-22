# Blue Yeti — Recommended physical settings

The app fixes the signal, but a bad physical setup cannot be recovered.
Starting point before opening Yeti Voice:

## Mic position

- **Cardioid mode**: rear selector on the "heart" symbol (3rd position).
  Speak **into the Blue logo side**, not from the top.
- Mouth-to-mic distance: **15–20 cm**, slightly off-axis to avoid plosives.
- Gain: **at minimum**, then raise gradually. Too much gain saturates
  and no DSP can repair digital saturation.

## Sample rate

- Set the Yeti to **48 kHz / 16-bit** (Sound Control Panel →
  Recording → Yeti → Properties → Studio quality).
- The app's DSP chain runs at 48 kHz: any other rate forces a
  pointless resampling step.

## Quick checklist

1. Cardioid + facing the logo, 15–20 cm.
2. Gain low then adjusted (the app's peak meter should peak around −12 dBFS for normal speech).
3. 48 kHz on the Windows side.
4. Turn off the Yeti's direct monitoring (headphone volume knob) if you monitor yourself through the app.
