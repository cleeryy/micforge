# Yeti Voice — Cadrage visuel (UI/UX uniquement)

## 1. Vue Simple (par défaut)

But : régler en 10 secondes, sans notion technique.

De haut en bas :
1. En-tête : « Yeti Voice », liste « Préréglage » (Claire / Direct / Visio) + bouton « Mode avancé ».
2. Zone voix : gros bouton « Son coupé / Son activé », vumètre vertical, témoin rouge « Saturation ! » en cas d'écrêtage.
3. Trois grands curseurs : « Volume », « Clarté », « Anti-bruit », chacun avec valeur en % et aide courte (« Plus fort », « Voix plus nette », « Coupe le bruit »).
4. Pied : listes « Micro d'entrée » (Yeti attendu) et « Sortie vers applis » (entrée CABLE attendue).

États :
- Tout présent : niveau vert animé quand on parle.
- Câble absent : bandeau orange « Sortie vers applis introuvable » + bouton « Revoir le premier démarrage » ; sortie grisée, curseurs actifs.
- Yeti absent : message « Micro Yeti non détecté » + bouton « Revérifier ».

## 2. Vue Avancée

Même base + chaîne fixe, dans l'ordre : coupe-bas 80 Hz → réduction de bruit → porte de bruit → compresseur → égaliseur 4 bandes + courbe → dé-esseur → limiteur.

Chaque bloc : une ligne avec interrupteur « Activé », nom français, 2–3 réglages (porte : « Seuil », « Ouverture » ; compresseur : « Force », « Attaque » ; égaliseur : 4 gains + courbe ; dé-esseur : « Sensibilité » ; limiteur : « Plafond »). Unités simples (Hz, dB, ms).

Pied de chaîne : mention « Retard estimé : ~X ms » + bouton « Tester mon micro (5 s) » (enregistre, rejoue, affiche le niveau maximal).

## 3. Premier démarrage

Assistant en 3 écrans :
1. « Micro » : cherche le Yeti, coche verte si trouvé, sinon aide au branchement.
2. « Sortie vers applis » : cherche l'entrée CABLE, sinon bouton « Ouvrir la page d'installation » + « Revérifier ».
3. « Réglages du micro » : rappel illustré — gain du Yeti au minimum, motif en cœur (cardioïde), 48 kHz sous Windows. Case « J'ai compris » pour finir.

Escamotable, mais le rappel revient tant que Yeti ou câble manque.

## 4. Principes

- Sombre, dense mais lisible : fond anthracite, texte clair, un seul accent orange, grands chiffres, contrastes forts.
- Clavier accessible : tout au clavier (Tab, flèches, Espace pour couper le son), focus visible, équivalent texte de chaque couleur.
- Français par défaut. Lexique imposé : « Son coupé » (jamais Mute), « Niveau » (jamais peak-meter), « Saturation » (jamais clip), « Préréglage » (jamais preset), « Curseur », « Interrupteur », « Porte de bruit », « Coupe-bas », « Égaliseur ».
