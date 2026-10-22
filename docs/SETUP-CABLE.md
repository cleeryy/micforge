# Câble audio virtuel — Installation (VB-CABLE)

Yeti Voice ne bundle **aucun driver audio** (choix volontaire + légal).
Le son traité sort vers un câble virtuel que tes apps utilisent comme micro.

## Installation de VB-CABLE (externe)

1. Télécharge VB-CABLE sur le site officiel : <https://vb-audio.com/Cable/>
2. Décompresse, clic droit sur `VBCABLE_Setup_x64.exe` → **Exécuter en tant qu'administrateur**.
3. Redémarre si demandé.
4. Vérifie : Paramètres Son → Lecture → « CABLE Input », Enregistrement → « CABLE Output ».

## Détection dans Yeti Voice

- Au démarrage, l'app liste les sorties WASAPI : choisis **« CABLE Input (VB-Audio Virtual Cable) »**.
- Bouton **Re-scanner** : reliste les périphériques (utile après installation du driver
  ou reconnexion USB du Yeti sans redémarrer l'app).

## Fallback VAC

Si VB-CABLE ne convient pas (latence, besoin multi-câbles), toute alternative
exposant un périphérique WASAPI de lecture fonctionne :
[Virtual Audio Cable](https://vac.muzychenko.net/) (payant),
[Voicemeeter](https://vb-audio.com/Voicemeeter/) (même éditeur).
Sélectionne simplement son entrée comme sortie dans l'app.

## Routage type

```
Blue Yeti ──WASAPI──▶ Yeti Voice (DSP) ──WASAPI──▶ CABLE Input
                                                      │
Discord/OBS : micro = « CABLE Output » ◀──────────────┘
```
