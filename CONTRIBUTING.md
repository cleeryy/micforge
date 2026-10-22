# Contribuer à Yeti Voice

Merci ! Quelques règles simples :

1. **PR petites et ciblées** : une feature / un fix par PR.
2. **Core testé** : toute logique dans `BlueYeti.Core` doit être couverte par xunit.
   Lance `dotnet test` avant de pousser (le projet WPF ne build que sur Windows).
3. **Pas de binaire/driver** dans le repo : le câble virtuel reste externe (voir README).
4. **Presets** : tout nouveau preset JSON doit respecter le schéma (`version`,
   `highpass`, `rnnoise`, `gate`, `comp`, `eq`, `deesser`, `limiter`, `notes`)
   et passer `python3 -m json.tool`.
5. **Français** pour les docs utilisateur, code et commentaires en anglais ou français.
6. Licence : toute contribution est sous AGPL-3.0-only.
