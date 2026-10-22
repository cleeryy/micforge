# Blue Yeti — Réglages physiques recommandés

L'app corrige le signal, mais un mauvais réglage physique ne se rattrape pas.
Base de départ avant d'ouvrir Yeti Voice :

## Position du micro

- **Mode cardioïde** : sélecteur arrière sur le symbole « cœur » (3e position).
  Parle **face au logo Blue**, pas par le dessus.
- Distance bouche-micro : **15–20 cm**, légèrement sur le côté pour éviter les plosives.
- Gain : **au minimum**, puis monte progressivement. Un gain trop fort sature
  et aucun DSP ne répare une saturation numérique.

## Fréquence d'échantillonnage

- Règle le Yeti en **48 kHz / 16 bits** (Panneau de configuration Son →
  Enregistrement → Yeti → Propriétés → Qualité studio).
- La chaîne DSP de l'app travaille en 48 kHz : tout autre taux force un
  rééchantillonnage inutile.

## Check-list rapide

1. Cardioïde + face au logo, 15–20 cm.
2. Gain mini puis ajusté (le peak-meter de l'app doit culminer vers −12 dBFS en voix normale).
3. 48 kHz côté Windows.
4. Coupe le monitoring direct du Yeti (bouton volume casque) si tu t'écoutes via l'app.
