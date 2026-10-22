# Yeti Voice

**Règle ton Blue Yeti à 100 % sur Windows.** Application open-source (AGPL-3.0-only)
de traitement micro en temps réel : branche ton Blue Yeti, applique une chaîne DSP
(réduction de bruit, gate, compresseur, EQ, de-esser, limiteur) et route le son
traité vers tes apps (Discord, OBS, visio…).

## Principe

- App 100 % **user-mode** : capture WASAPI du micro via [NAudio](https://github.com/naudio/NAudio),
  traitement DSP en C# (.NET 8), lecture vers un **câble audio virtuel externe**.
- Le câble virtuel **n'est pas fourni** avec l'app : installe
  [VB-CABLE](https://vb-audio.com/Cable/) (gratuit, driver signé) séparément,
  puis sélectionne « CABLE Input » comme sortie dans Yeti Voice.
- Voir `docs/SETUP-CABLE.md` pour l'installation et `docs/BLUE-YETI.md` pour les
  réglages physiques du micro.

## Prérequis

- Windows 10/11 (x64)
- [.NET 8 SDK](https://dotnet.microsoft.com/download/dotnet/8.0) (pour builder)

## Build / Publish portable

```powershell
dotnet build yeti-voice.sln
dotnet publish src/BlueYeti.App/BlueYeti.App.csproj -c Release -r win-x64 --self-contained -o publish
```

Le dossier `publish/` est portable : copie-le où tu veux, aucun installeur requis.
Installeur Inno Setup (optionnel) : voir `installer/setup.iss`.

## Scope MVP

- [x] Sélection entrée (Blue Yeti) / sortie (câble virtuel), re-scan des devices
- [x] Chaîne DSP : high-pass, RNNoise, noise gate, compresseur, EQ 4 bandes, de-esser, limiteur
- [x] Presets JSON : `yeti-clear`, `yeti-stream`, `yeti-meeting`
- [x] Peak-meter temps réel
- [ ] RNNoise natif (actuellement stub — TODO)
- [ ] Auto-gain, monitoring latence

## Hors-scope

- Redistribution d'un driver / câble virtuel (interdit : l'utilisateur l'installe lui-même)
- macOS / Linux (WASAPI = Windows uniquement)
- Effets voix (pitch, robot) — DSP correctif uniquement

## Structure

```
src/BlueYeti.Core/    # Moteur audio + chaîne DSP + modèles (net8.0, testable)
src/BlueYeti.App/     # UI WPF (net8.0-windows) : sélecteurs, peak-meter
tests/BlueYeti.Core.Tests/  # Tests xunit du Core
presets/              # Presets JSON livrés avec l'app
docs/                 # DESIGN.md (maquette), BLUE-YETI.md, SETUP-CABLE.md
installer/            # Script Inno Setup
```

## Licence

AGPL-3.0-only — voir `LICENSE`. Copyright (c) 2026 cleeryy.
