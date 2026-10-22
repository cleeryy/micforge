# Yeti Voice — Visual scope (UI/UX only)

## 1. Simple view (default)

Goal: dial in your sound in 10 seconds, no technical jargon.

Top to bottom:
1. Header: "Yeti Voice", "Preset" list (Clear / Live / Meeting) + "Advanced mode" button.
2. Voice area: big "Muted / Unmuted" button, vertical level meter, red "Clipping!" indicator on overload.
3. Three large sliders: "Volume", "Clarity", "Noise reduction", each with a % value and a short hint ("Louder", "Clearer voice", "Cuts background noise").
4. Footer: "Input microphone" list (Yeti expected) and "Output to apps" list (CABLE input expected).

States:
- All present: green level animates while speaking.
- Cable missing: orange banner "Output to apps not found" + "Review first-run setup" button; output greyed out, sliders still active.
- Yeti missing: "Yeti microphone not detected" message + "Check again" button.

## 2. Advanced view

Same base + fixed chain, in order: 80 Hz high-pass → noise reduction → noise gate → compressor → 4-band equalizer + curve → de-esser → limiter.

Each block: one row with an "Enabled" switch, its name, 2–3 settings (gate: "Threshold", "Attack"; compressor: "Amount", "Attack"; equalizer: 4 gains + curve; de-esser: "Sensitivity"; limiter: "Ceiling"). Plain units (Hz, dB, ms).

Chain footer: "Estimated latency: ~X ms" note + "Test my mic (5 s)" button (records, plays back, shows peak level).

## 3. First run

3-screen wizard:
1. "Microphone": looks for the Yeti, green check if found, otherwise plug-in help.
2. "Output to apps": looks for the CABLE input, otherwise an "Open install page" button + "Check again".
3. "Mic setup": illustrated reminder — Yeti gain at minimum, heart (cardioid) pattern, 48 kHz in Windows. "Got it" checkbox to finish.

Skippable, but the reminder comes back while Yeti or cable is missing.

## 4. Principles

- Dark, dense but readable: charcoal background, light text, a single orange accent, big numbers, strong contrast.
- Keyboard accessible: everything via keyboard (Tab, arrows, Space to mute/unmute), visible focus, a text equivalent for every color cue.
- English by default.
