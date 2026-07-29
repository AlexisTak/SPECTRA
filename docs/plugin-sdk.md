# SDK de plugins — SPECTRA

SPECTRA étend ses capacités via des **transforms sandboxés** en WebAssembly (WASM) via le runtime [Extism](https://extism.org/). Un transform peut être écrit dans n'importe quel langage disposant d'un PDK Extism (Rust, TypeScript, Go, Python, Zig…).

## Architecture d'un plugin

Un plugin est un dossier contenant :

```
mon-transform/
├── plugin.wasm          # binaire compilé
└── manifest.toml        # métadonnées et permissions
```

### Manifeste (`manifest.toml`)

```toml
name = "dns-resolution"
description = "Résolution DNS A/AAAA/MX/TXT"
version = "1.0.0"
authors = ["Votre Nom <votre@email.com>"]
license = "Apache-2.0"

[input]
kind = "Domain"

[output]
kinds = ["IpAddress", "Domain"]

[network]
allowlist = ["8.8.8.8", "1.1.1.1"]
max_response_size = "1 MiB"
respect_robots_txt = true
user_agent = "SPECTRA-Transform/1.0"

[quotas]
max_requests = 10
max_duration_sec = 30
```

## Interface WASM (Host Functions)

Le runtime SPECTRA expose les fonctions hôte suivantes aux plugins WASM :

| Fonction | Description | Limites |
|---|---|---|
| `http_get(url)` | Requête GET HTTP(S) | Domaine dans `allowlist`, rate limit par domaine |
| `http_post(url, body)` | Requête POST HTTP(S) | Mêmes limites |
| `dns_lookup(domain, type)` | Requête DNS via l'hôte | Type A, AAAA, MX, TXT, NS |
| `log(level, message)` | Journal structuré | `debug`, `info`, `warn`, `error` |
| `store_temp(key, value)` | Stockage temporaire inter-appels | 10 MiB max, purgé à la fin du transform |
| `emit_entity(entity)` | Émettre une entité découverte | Validé contre l'ontologie |
| `emit_observation(obs)` | Émettre une observation | Marquée `COLLECTED` |

## Cycle de vie d'un transform

1. **Installation** : le plugin est copié dans `~/.config/SPECTRA/plugins/` (vérifé Ed25519).
2. **Activation** : SPECTRA lit le manifeste et valide les permissions.
3. **Exécution** : l'hôte prépare l'entrée, invoque la fonction `transform` exportée, récupère les entités/observations.
4. **Post-traitement** : rate limiting, vérification de `robots.txt`, hachage des réponses brutes, ajout au dossier.

## Exemple minimal (Rust)

```rust
use extism_pdk::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Input {
    domain: String,
}

#[derive(Serialize, Deserialize)]
struct Output {
    ips: Vec<String>,
}

#[plugin_fn]
pub fn transform(Json(input): Json<Input>) -> FnResult<Json<Output>> {
    let resp = http::get(&format!("https://dns.google/resolve?name={}&type=A", input.domain))?;
    // … parsing …
    Ok(Json(Output { ips: vec![…] }))
}
```

## Signature et sécurité

Chaque plugin doit être signé avec une clé Ed25519. SPECTRA vérifie la signature à l'installation et refuse tout plugin non signé ou dont la signature est invalide.

```bash
tauri signer sign --private-key ~/.tauri/mykey plugin.wasm
```

Le registre de plugins local est un simple dossier. Il n'y a pas de marketplace centralisé : chaque organisation gère ses propres plugins.

## Transforms natifs

Le socle SPECTRA inclut des transforms natifs (DNS, WHOIS, Certificate Transparency, WhatsMyName) compilés en Rust pour la performance. Ils exposent la **même interface** que les plugins WASM et sont indiscernables pour l'analyste.

---

*Le SDK de plugins est publié sous licence Apache-2.0.*
