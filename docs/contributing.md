# Guide de contribution — SPECTRA

Merci d'envisager de contribuer à SPECTRA ! Ce document résume les règles de développement et de collaboration.

## Code de conduite

- Respectez les autres contributeurs.
- Privilégiez la clarté et la bienveillance dans les revues de code.
- SPECTRA est un outil d'investigation légale et éthique : toute contribution facilitant le harcèlement, la traque ou la surveillance non consentie sera refusée.

## Signaler un bug

1. Vérifiez qu'il n'existe pas déjà dans les issues GitHub.
2. Utilisez le modèle de bug report.
3. Incluez :
   - Version de SPECTRA (`Aide > À propos`)
   - Système d'exploitation
   - Étapes de reproduction minimales
   - Logs (`~/.config/SPECTRA/logs/`)

## Proposer une fonctionnalité

Ouvrez une issue avec le label `enhancement`. Décrivez :
- Le problème que vous résolvez.
- La solution envisagée (ou les options).
- L'impact sur la confidentialité et la sécurité.

## Environnement de développement

### Prérequis

- Rust 1.80+ (`rustup update stable`)
- Node.js 22+ (`nvm use 22`)
- pnpm ou npm
- Linux : `libwebkit2gtk-4.1-dev`, `libappindicator3-dev`, `librsvg2-dev`, `patchelf`

### Installation

```bash
git clone https://github.com/spectra-osint/spectra.git
cd spectra
npm install
cargo build --workspace
```

### Lancer l'application

```bash
npm run tauri:dev
```

### Tests

```bash
cargo test --workspace
npx vitest run
```

### Lints

```bash
cargo fmt --check -p spectra-core -p spectra-store -p spectra-audit -p spectra-report -p spectra-ai
cargo clippy --workspace --all-targets -- -D warnings
```

## Architecture du code

- **`spectra-core`** : modèle de domaine. Aucune I/O. ≥ 90 % de couverture.
- **`spectra-store`** : SQLite, migrations, chiffrement.
- **`spectra-audit`** : journal hash-chaîné.
- **`spectra-report`** : génération Markdown / HTML / PDF.
- **`spectra-ai`** : backends IA (Ollama) et garde-fous.
- **`src-tauri/`** : shell Tauri + commandes IPC.
- **`app/`** : frontend Next.js + React (canvas Sigma, tableaux, etc.).

## Style de code

- **Rust** : `cargo fmt`, `clippy::pedantic`, `#![deny(warnings)]` sur les crates spectra.
- **TypeScript** : `strict: true`, `noUncheckedIndexedAccess`, zéro `any`.
- **Commits** : Conventional Commits (`feat:`, `fix:`, `docs:`, `refactor:`, `test:`).
- **ADR** : toute décision structurante → fichier dans `docs/adr/`.

## Revue de code

Chaque PR doit :
- Passer la CI (build, tests, clippy, cargo-deny, npm audit).
- Être revue par au moins un mainteneur.
- Inclure des tests pour les nouvelles fonctionnalités.
- Mettre à jour la documentation si nécessaire.

## Traductions

SPECTRA supporte FR et EN. Les chaînes sont dans `app/locales/`. Pour ajouter une langue, créez un fichier `app/locales/{lang}.json` et ouvrez une PR.

## Licence

En contribuant, vous acceptez que votre travail soit publié sous les licences du projet :
- Cœur : AGPL-3.0
- SDK de plugins : Apache-2.0
