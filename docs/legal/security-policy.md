# Politique de sécurité — SPECTRA

## Versions supportées

| Version | Supportée |
|---|---|
| 0.2.x | ✅ Oui |
| 0.1.x | ❌ Non (développement initial) |

## Signalement de vulnérabilités

SPECTRA prend la sécurité très au sérieux. Si vous découvrez une vulnérabilité, merci de suivre la procédure responsable de divulgation :

1. **Ne pas ouvrir une issue publique** immédiatement.
2. Envoyez un email chiffré à `security@spectra-osint.org` (adresse indicative) avec :
   - Une description détaillée de la vulnérabilité.
   - Les étapes de reproduction.
   - L'impact estimé.
   - Vos coordonnées et préférence de crédit (anonyme, pseudonyme, nom réel).
3. Nous répondrons dans un délai de **72 heures** pour accuser réception.
4. Nous travaillerons avec vous pour évaluer la gravité, préparer un correctif et planifier la divulgation coordonnée.

## Mesures de sécurité en place

### Build et dépendances

- `cargo deny` et `cargo audit` en CI pour détecter les dépendances vulnérables.
- Interdiction de dépendances sous licence incompatible ou non libre.
- `npm audit --audit-level=high` en CI.

### Code Rust

- `#![deny(warnings)]` sur les crates `spectra-*`.
- `clippy::pedantic` activé.
- Aucun `unsafe` sans commentaire `// SAFETY:` justifié.
- Fuzzing (`cargo-fuzz`) sur les parseurs externes (HTML, JSON de sources tierces).

### Application Tauri

- CSP strict, `dangerousDisableAssetCspModification` interdit.
- Permissions Tauri au minimum nécessaire (capabilities explicites).
- Aucune commande Tauri n'accepte de chemin arbitraire sans validation.
- Chiffrement au repos du dossier d'enquête (XChaCha20-Poly1305 + Argon2id).

### OPSEC de l'analyste

- Profils réseau isolés (direct / proxy / SOCKS5 / Tor).
- Rotation de User-Agent et jar de cookies isolé par dossier.
- Détection de fuite Tor (refuse de collecter si le circuit n'est pas établi).
- Respect de `robots.txt` par défaut.

## Historique des corrections

| Date | Version | Description | CVE |
|---|---|---|---|
| — | — | Aucune vulnérabilité publiquement signalée à ce jour | — |

---

*Dernière mise à jour : 2026-07-29*
