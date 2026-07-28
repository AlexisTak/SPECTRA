# Lancer l'application

## Prérequis

- Node 20+ et npm
- Rust stable (`rustup`)
- Windows : WebView2 (préinstallé sur Windows 11)

```bash
npm install
```

## Développement

```bash
npm run tauri:dev
```

Tauri démarre le serveur Next.js lui-même (`beforeDevCommand`) puis ouvre la
fenêtre. Le premier lancement compile le backend Rust : compter 1 à 2 minutes.
Les suivants sont quasi instantanés.

## Production

```bash
npm run tauri:build
```

Produit un exécutable et un installeur dans `src-tauri/target/release/bundle/`.

## Vérifier que tout fonctionne

L'écran d'accueil interroge le backend au chargement et affiche le résultat :

- **« Backend joignable »** — la chaîne React → IPC → Rust → SQLite fonctionne.
- **« Backend injoignable »** — l'erreur brute est affichée. Dans un navigateur
  ordinaire (`npm run dev` seul) c'est le comportement **attendu** : les
  commandes n'existent que dans la fenêtre Tauri.

La base est créée au premier démarrage dans le répertoire de données
applicatives, pas dans le dossier du projet :

| Système | Emplacement |
|---|---|
| Windows | `%APPDATA%\com.cekarna.enquetes\casetrack.db` |
| macOS | `~/Library/Application Support/com.cekarna.enquetes/` |
| Linux | `~/.local/share/com.cekarna.enquetes/` |

## Pannes courantes

### « Another next dev server is already running »

Un serveur Next.js d'une session précédente occupe le port 3000. Le message
indique le PID à terminer :

```bash
# Windows
taskkill /PID <pid> /F
```

### La fenêtre reste blanche

Vérifier que `out/index.html` existe :

```bash
npm run build && ls out/index.html
```

En `tauri:build`, `out/` est régénéré automatiquement. Si le dossier ne contient
qu'un répertoire `_next` sans `index.html`, c'est qu'aucune page n'a été
générée — vérifier que `app/page.tsx` existe.

### « unknown field » dans la configuration Tauri

`tauri.conf.json` contient de la syntaxe Tauri 1. En Tauri 2, la section
`plugins` ne prend plus `{"enabled": true}` : les plugins s'initialisent dans
`lib.rs` via `.plugin(...)`, et `plugins` ne sert qu'à leur configuration.

### Le worker de layout est bloqué

Symptôme : le graphe ne se dispose jamais, sans erreur visible. La bibliothèque
crée son worker depuis une URL `blob:` ; la CSP doit contenir
`worker-src 'self' blob:`. Sans cette directive le blocage est silencieux.

## Politique de sécurité de contenu

Deux CSP distinctes dans `tauri.conf.json` :

- `csp` — production. Stricte : `script-src 'self'`, sans `unsafe-inline` ni
  `unsafe-eval`.
- `devCsp` — développement uniquement. Tolère `unsafe-inline`/`unsafe-eval` et
  le serveur Next.js, exigés par le rechargement à chaud.

Cette séparation est délibérée : l'application affiche du contenu web collecté,
et une CSP permissive en production serait une porte d'entrée XSS directe vers
`invoke()`, donc vers toute la base d'enquête.
