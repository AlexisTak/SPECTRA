# Guide utilisateur — SPECTRA

## Présentation

SPECTRA est une plateforme d'investigation OSINT **100 % locale**. Aucune donnée d'enquête ne quitte votre machine. Le logiciel fonctionne entièrement hors ligne, sauf lors des collectes OSINT explicitement déclenchées par l'analyste.

## Installation

### Linux

Téléchargez l'AppImage ou le paquet `.deb` / `.rpm` depuis la page Release GitHub. Rendez l'AppImage exécutable :

```bash
chmod +x SPECTRA_*.AppImage
./SPECTRA_*.AppImage
```

### macOS

Ouvrez le `.dmg` et glissez SPECTRA dans le dossier Applications. Au premier lancement, macOS peut afficher un avertissement de sécurité : faites un clic droit → Ouvrir pour l'autoriser.

### Windows

Exécutez le `.msi` ou l'installateur `.exe` (NSIS). SPECTRA s'installe dans `C:\Program Files\SPECTRA`.

## Premiers pas

1. **Créer un dossier** : `Dossiers > Nouveau dossier`. Renseignez la base légale et la finalité du traitement (RGPD).
2. **Ajouter un sélecteur** : dans un dossier, utilisez `Recherche` ou `OSINT` pour entrer un identifiant (pseudo, email, téléphone, domaine…).
3. **Lancer une collecte** : les transforms OSINT s'exécutent en arrière-plan. Le panneau de tâches montre chaque requête HTTP en cours.
4. **Explorer le graphe** : les entités et relations découvertes apparaissent dans la vue `Graphe`. Sélectionnez, zoomez, déplacez.
5. **Générer un rapport** : `Rapports > Nouveau rapport`. Choisissez le format (Markdown, HTML, PDF).

## Raccourcis clavier

| Raccourci | Action |
|---|---|
| `Ctrl + K` | Palette de commandes |
| `Ctrl + N` | Nouveau dossier |
| `Ctrl + F` | Recherche globale |
| `Ctrl + G` | Aller au Graphe |
| `Esc` | Fermer la modale / panneau actif |
| `Ctrl + Molette` | Zoom dans le graphe |
| `Espace + Glisser` | Pan dans le graphe |

## Fonctionnalités IA (optionnel)

Si Ollama est installé localement (`http://localhost:11434`), SPECTRA détecte automatiquement le backend et propose :

- **Extraction d'entités (NER)** depuis du texte libre collé dans les notes.
- **Résumé de dossier** pour préparer un rapport.
- **Suggestions de pivot** : "Au vu de ce nœud, ces 3 pistes sont pertinentes".
- **Détection de doublons** par similarité sémantique.

**Important** : toute sortie de modèle est marquée `INFERRED` (provenance inférée). Elle n'est pas une preuve et est exclue des rapports par défaut. Un humain doit valider chaque proposition avant intégration au graphe.

## Conformité RGPD

SPECTRA intègre la conformité dès la conception :

- **Base légale et finalité** obligatoires à la création de chaque dossier.
- **Durée de conservation** paramétrable avec alerte à échéance.
- **Minimisation** : avertissement quand un transform collecte des catégories sensibles (art. 9 RGPD).
- **Journal d'audit** hash-chaîné, exportable pour répondre à une demande d'accès CNIL.
- **Chiffrement au repos** : le fichier `.spectra` est chiffré XChaCha20-Poly1305, clé dérivée Argon2id.

## Mises à jour

SPECTRA embarque un mécanisme de mise à jour auto-hébergé. Par défaut, il est désactivé. Pour l'activer, configurez un serveur statique (S3, Nginx, GitHub Pages) pointant vers le fichier `latest.json` généré par `tauri build`, et renseignez la clé publique Ed25519 dans les paramètres.

---

*SPECTRA est publié sous licence AGPL-3.0.*
