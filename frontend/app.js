"use strict";
/* MicForge vanilla frontend — static, no build step.
 * Talks to the frozen backend API ONLY via window.__TAURI__.core.invoke:
 *   list_devices, start, stop, apply_preset, set_block, get_status.
 * Without Tauri (plain browser) a mock backend simulates devices + meter. */

const $ = (id) => document.getElementById(id);
const els = {
  preset: $("presetSelect"), viewSimple: $("viewSimple"), viewPro: $("viewPro"),
  pro: $("proSection"), onboard: $("onboardCard"), obMic: $("obMicState"),
  obCable: $("obCableState"), obAck: $("onboardAck"), obRecheck: $("onboardRecheck"),
  obDismiss: $("onboardDismiss"), cableBanner: $("cableBanner"),
  cableReview: $("cableReview"), micMissing: $("micMissing"), micRecheck: $("micRecheck"),
  mute: $("muteBtn"), meter: $("meter"), fill: $("meterFill"), clip: $("clipLed"),
  peak: $("peakLabel"), latency: $("latencyLabel"), start: $("startBtn"), stop: $("stopBtn"),
  inputs: $("inputSelect"), outputs: $("outputSelect"), rescan: $("rescanBtn"),
  chain: $("chain"), chainLatency: $("chainLatency"), micTest: $("micTestBtn"),
  micTestResult: $("micTestResult"), badge: $("backendBadge"),
};
const sliders = [
  { el: $("slVolume"), out: $("slVolumeVal") },
  { el: $("slClarity"), out: $("slClarityVal") },
  { el: $("slNoise"), out: $("slNoiseVal") },
];

/* ---- DSP chain model (fixed order). Block ids mirror preset JSON keys. ---- */
const BLOCKS = [
  { id: "highpass", name: "High-pass", enabled: true,
    params: [{ k: "frequencyHz", label: "Cutoff (Hz)", min: 20, max: 300, step: 1, v: 80 }] },
  { id: "rnnoise", name: "Noise suppression", enabled: true,
    params: [{ k: "strength", label: "Strength (%)", min: 0, max: 100, step: 1, v: 60 }] },
  { id: "gate", name: "Gate", enabled: true,
    params: [{ k: "thresholdDb", label: "Threshold (dB)", min: -80, max: 0, step: 1, v: -40 },
             { k: "attackMs", label: "Attack (ms)", min: 0.1, max: 100, step: 0.5, v: 5 }] },
  { id: "comp", name: "Compressor", enabled: true,
    params: [{ k: "thresholdDb", label: "Threshold (dB)", min: -60, max: 0, step: 1, v: -18 },
             { k: "ratio", label: "Ratio (:1)", min: 1, max: 20, step: 0.5, v: 3 },
             { k: "attackMs", label: "Attack (ms)", min: 0.1, max: 200, step: 1, v: 10 }] },
  { id: "eq", name: "EQ", enabled: true,
    params: [{ k: "lowDb", label: "Low 120 Hz (dB)", min: -12, max: 12, step: 0.5, v: -2 },
             { k: "midDb", label: "Mid 500 Hz (dB)", min: -12, max: 12, step: 0.5, v: 0 },
             { k: "presenceDb", label: "Presence 3 kHz (dB)", min: -12, max: 12, step: 0.5, v: 2 },
             { k: "airDb", label: "Air 8 kHz (dB)", min: -12, max: 12, step: 0.5, v: -1 }] },
  { id: "deesser", name: "De-esser", enabled: true,
    params: [{ k: "thresholdDb", label: "Sensitivity (dB)", min: -60, max: 0, step: 1, v: -24 },
             { k: "frequencyHz", label: "Frequency (Hz)", min: 3000, max: 10000, step: 100, v: 6500 }] },
  { id: "limiter", name: "Limiter", enabled: true,
    params: [{ k: "ceilingDb", label: "Ceiling (dB)", min: -12, max: 0, step: 0.5, v: -1 },
             { k: "releaseMs", label: "Release (ms)", min: 1, max: 500, step: 1, v: 50 }] },
];

/* ---- Backend: real Tauri invoke or demo mock ---- */
const tauriInvoke = (window.__TAURI__ && window.__TAURI__.core && window.__TAURI__.core.invoke)
  ? window.__TAURI__.core.invoke.bind(window.__TAURI__.core) : null;

const mock = {
  running: false, muted: false, t: 0,
  devices: {
    inputs: [{ id: "mock-yeti", name: "Mock Yeti Stereo Microphone" }, { id: "mock-laptop", name: "Mock Laptop Microphone" }],
    outputs: [{ id: "mock-cable", name: "CABLE Input (VB-Audio Virtual Cable)" }, { id: "mock-spk", name: "Mock Speakers" }],
  },
};
async function invoke(cmd, args) {
  if (tauriInvoke) return tauriInvoke(cmd, args || {});
  await new Promise((r) => setTimeout(r, 30)); // feel async like IPC
  switch (cmd) {
    case "list_devices": return mock.devices;
    case "start": mock.running = true; return { latency_ms: 12.5 };
    case "stop": mock.running = false; return {};
    case "apply_preset": return {};
    case "set_block": return {};
    case "get_status": {
      mock.t += 0.1;
      const voice = mock.running && !mock.muted
        ? -22 + 10 * Math.sin(mock.t * 7) + 5 * Math.sin(mock.t * 23) : -Infinity;
      const peak = mock.running && !mock.muted ? Math.min(voice, -2) : -Infinity;
      return { running: mock.running, peak_db: peak, clip: peak > -1, latency_ms: 12.5 };
    }
    default: throw new Error("unknown command: " + cmd);
  }
}

let state = { running: false, muted: false, micTest: null };

function fmtDb(db) {
  return (!isFinite(db) || db <= -60) ? "−∞ dB" : db.toFixed(1) + " dB";
}
function updateMeter(peakDb, clip, latencyMs) {
  const pct = !isFinite(peakDb) ? 0 : Math.max(0, Math.min(100, ((peakDb + 60) / 60) * 100));
  els.fill.style.width = pct.toFixed(1) + "%";
  els.meter.setAttribute("aria-valuenow", String(isFinite(peakDb) ? Math.round(peakDb) : -60));
  els.meter.setAttribute("aria-valuetext", "Peak " + fmtDb(peakDb) + (clip ? ", clipping" : ""));
  els.clip.classList.toggle("on", !!clip);
  els.clip.setAttribute("aria-label", clip ? "Clipping! Reduce gain." : "No clipping");
  els.peak.textContent = "Peak: " + fmtDb(peakDb);
  const lat = latencyMs == null ? "— ms" : Number(latencyMs).toFixed(1) + " ms";
  els.latency.textContent = "Latency: " + lat;
  els.chainLatency.textContent = "Estimated latency: ~" + lat;
}

function setTransport(running) {
  state.running = running;
  els.start.disabled = running || els.micMissing.hidden === false;
  els.stop.disabled = !running;
  els.badge.textContent = tauriInvoke ? (running ? "Live" : "Connected") : (running ? "Live (demo)" : "Demo mode");
  els.badge.classList.toggle("live", running);
}

/* ---- Devices + onboarding states ---- */
function isCable(devs) {
  return devs.outputs.some((d) => /cable/i.test(d.name + " " + d.id));
}
function fillSelect(sel, list, fallbackLabel) {
  sel.innerHTML = "";
  if (!list.length) {
    const o = document.createElement("option");
    o.value = ""; o.textContent = fallbackLabel; sel.appendChild(o); return;
  }
  for (const d of list) {
    const o = document.createElement("option");
    o.value = d.id; o.textContent = d.name; sel.appendChild(o);
  }
}
async function refreshDevices() {
  let devs;
  try {
    devs = await invoke("list_devices");
  } catch (e) { console.error(e); return; }
  devs.inputs = devs.inputs || []; devs.outputs = devs.outputs || [];
  const preferIn = devs.inputs.find((d) => /yeti/i.test(d.name)) || devs.inputs[0];
  const preferOut = devs.outputs.find((d) => /cable/i.test(d.name + " " + d.id)) || devs.outputs[0];
  fillSelect(els.inputs, devs.inputs, "No microphone found");
  fillSelect(els.outputs, devs.outputs, "No output found");
  if (preferIn) els.inputs.value = preferIn.id;
  if (preferOut) els.outputs.value = preferOut.id;
  const micGone = devs.inputs.length === 0;
  const cableGone = !isCable(devs);
  els.micMissing.hidden = !micGone;
  els.cableBanner.hidden = !cableGone;
  els.start.disabled = state.running || micGone;
  els.obMic.textContent = micGone ? "Microphone: not found — plug it in." : "Microphone: found (" + (preferIn ? preferIn.name : "") + ").";
  els.obCable.textContent = cableGone ? "CABLE output: not found — install VB-CABLE." : "CABLE output: found.";
  if (!localStorage.getItem("micforge.onboarded") && (micGone || cableGone)) els.onboard.hidden = false;
}

/* ---- Simple sliders -> chain params ----
 * Volume -> comp.makeupDb (0..100% = 0..12 dB); Clarity -> eq.presenceDb (0..6 dB);
 * Noise reduction -> rnnoise enable + strength; gate threshold tightens past 70%. */
let sliderTimer = null;
function pushSimpleMapping() {
  const vol = Number(sliders[0].el.value), cla = Number(sliders[1].el.value), nz = Number(sliders[2].el.value);
  const eq = BLOCKS.find((b) => b.id === "eq");
  const rn = BLOCKS.find((b) => b.id === "rnnoise");
  const gate = BLOCKS.find((b) => b.id === "gate");
  const comp = BLOCKS.find((b) => b.id === "comp");
  eq.params.find((p) => p.k === "presenceDb").v = +(cla / 100 * 6).toFixed(1);
  rn.enabled = nz > 0; rn.params[0].v = nz;
  gate.params.find((p) => p.k === "thresholdDb").v = nz >= 70 ? -50 : -40;
  syncChainInputs();
  clearTimeout(sliderTimer);
  sliderTimer = setTimeout(async () => {
    try {
      const makeupDb = +(vol / 100 * 12).toFixed(1);
      await invoke("set_block", { block: "comp", enabled: comp.enabled, params: { makeupDb } });
      await invoke("set_block", { block: "eq", enabled: eq.enabled, params: { presenceDb: eq.params.find((p) => p.k === "presenceDb").v } });
      await invoke("set_block", { block: "rnnoise", enabled: rn.enabled, params: { strength: nz } });
      await invoke("set_block", { block: "gate", enabled: gate.enabled, params: { thresholdDb: gate.params.find((p) => p.k === "thresholdDb").v } });
    } catch (e) { console.error(e); }
  }, 150);
}

/* ---- Pro chain UI ---- */
function buildChain() {
  els.chain.innerHTML = "";
  for (const b of BLOCKS) {
    const wrap = document.createElement("div");
    wrap.className = "block";
    const head = document.createElement("div");
    head.className = "block-head";
    const sw = document.createElement("label");
    sw.className = "switch";
    const cb = document.createElement("input");
    cb.type = "checkbox"; cb.checked = b.enabled;
    cb.setAttribute("aria-label", b.name + " enabled");
    cb.dataset.block = b.id;
    cb.addEventListener("change", () => pushBlock(b));
    sw.append(cb, document.createTextNode("Enabled"));
    const name = document.createElement("strong");
    name.textContent = b.name;
    head.append(sw, name);
    const grid = document.createElement("div");
    grid.className = "params";
    for (const p of b.params) {
      const cell = document.createElement("div");
      cell.className = "param";
      const lab = document.createElement("label");
      const inp = document.createElement("input");
      inp.type = "number"; inp.min = String(p.min); inp.max = String(p.max); inp.step = String(p.step);
      inp.value = String(p.v); inp.dataset.block = b.id; inp.dataset.param = p.k;
      const lid = "p-" + b.id + "-" + p.k;
      inp.id = lid; lab.htmlFor = lid; lab.textContent = p.label;
      inp.addEventListener("change", () => {
        const v = Math.max(p.min, Math.min(p.max, Number(inp.value)));
        inp.value = String(v); p.v = v; pushBlock(b);
      });
      cell.append(lab, inp);
      grid.appendChild(cell);
    }
    wrap.append(head, grid);
    els.chain.appendChild(wrap);
  }
}
function syncChainInputs() {
  for (const b of BLOCKS) {
    const cb = els.chain.querySelector('input[type="checkbox"][data-block="' + b.id + '"]');
    if (cb) cb.checked = b.enabled;
    for (const p of b.params) {
      const inp = els.chain.querySelector('input[data-block="' + b.id + '"][data-param="' + p.k + '"]');
      if (inp && document.activeElement !== inp) inp.value = String(p.v);
    }
  }
}
let blockTimer = null;
function pushBlock(b) {
  const cb = els.chain.querySelector('input[type="checkbox"][data-block="' + b.id + '"]');
  b.enabled = cb ? cb.checked : b.enabled;
  const params = {};
  for (const p of b.params) params[p.k] = p.v;
  clearTimeout(blockTimer);
  blockTimer = setTimeout(async () => {
    try { await invoke("set_block", { block: b.id, enabled: b.enabled, params }); }
    catch (e) { console.error(e); }
  }, 150);
}

/* ---- Status polling (~10 Hz) ---- */
async function poll() {
  try {
    const s = await invoke("get_status");
    updateMeter(s.peak_db, s.clip, s.latency_ms);
    if (typeof s.running === "boolean" && s.running !== state.running) setTransport(s.running);
    if (state.micTest) {
      if (isFinite(s.peak_db)) state.micTest.peak = Math.max(state.micTest.peak, s.peak_db);
    }
  } catch (e) { console.error(e); }
}

/* ---- Events ---- */
function bindEvents() {
  for (const s of sliders) {
    s.el.addEventListener("input", () => { s.out.textContent = s.el.value + "%"; pushSimpleMapping(); });
  }
  els.preset.addEventListener("change", async () => {
    try { await invoke("apply_preset", { name: els.preset.value }); } catch (e) { console.error(e); }
  });
  els.viewSimple.addEventListener("click", () => setView(false));
  els.viewPro.addEventListener("click", () => setView(true));
  els.start.addEventListener("click", async () => {
    try {
      const r = await invoke("start", {
        input_id: els.inputs.value || null, output_id: els.outputs.value || null, preset: els.preset.value,
      });
      setTransport(true);
      if (r && r.latency_ms != null) updateMeter(-60, false, r.latency_ms);
    } catch (e) { console.error(e); }
  });
  els.stop.addEventListener("click", async () => {
    try { await invoke("stop"); } catch (e) { console.error(e); }
    setTransport(false); updateMeter(-Infinity, false, null);
  });
  els.mute.addEventListener("click", () => {
    state.muted = !state.muted;
    if (!tauriInvoke) mock.muted = state.muted;
    els.mute.textContent = state.muted ? "Muted" : "Unmuted";
    els.mute.setAttribute("aria-pressed", String(state.muted));
  });
  els.rescan.addEventListener("click", refreshDevices);
  els.micRecheck.addEventListener("click", refreshDevices);
  els.cableReview.addEventListener("click", () => { els.onboard.hidden = false; els.onboard.querySelector("h2").focus?.(); });
  els.obRecheck.addEventListener("click", refreshDevices);
  els.obDismiss.addEventListener("click", () => {
    if (els.obAck.checked) localStorage.setItem("micforge.onboarded", "1");
    els.onboard.hidden = true;
  });
  els.micTest.addEventListener("click", () => {
    if (state.micTest) return;
    state.micTest = { peak: -Infinity };
    els.micTest.disabled = true;
    els.micTestResult.textContent = "Listening… speak for 5 s.";
    setTimeout(() => {
      const peak = state.micTest ? state.micTest.peak : -Infinity;
      state.micTest = null;
      els.micTest.disabled = false;
      els.micTestResult.textContent = "Mic test peak: " + fmtDb(peak) + ".";
    }, 5000);
  });
  document.addEventListener("keydown", (e) => {
    if (e.code === "Space" && e.target === document.body) { e.preventDefault(); els.mute.click(); }
  });
}
function setView(pro) {
  els.pro.hidden = !pro;
  els.viewPro.setAttribute("aria-pressed", String(pro));
  els.viewSimple.setAttribute("aria-pressed", String(!pro));
}

/* ---- Init ---- */
buildChain();
bindEvents();
if (localStorage.getItem("micforge.onboarded")) els.onboard.hidden = true;
else els.onboard.hidden = false;
refreshDevices();
setTransport(false);
setInterval(poll, 100);
poll();
