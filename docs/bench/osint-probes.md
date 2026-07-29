# Sondes OSINT — vérification réelle

**Date :** 2026-07-29
**Méthode :** `cargo run -p spectra-probe --example live_check -- <pseudo>`
**Réseau :** connexion domestique, France, sans proxy

## Résultats

Deux campagnes : un compte public connu (`torvalds`) et un pseudo aléatoire qui
n'existe nulle part (`zzqxvnptrwbk9438`).

| Sonde | Contrôle | `torvalds` | `zzqxvnptrwbk9438` |
|---|---|---|---|
| GitHub | sain | **trouvé** | absent |
| GitLab | bloqué | **trouvé** | bloqué |
| Hacker News | sain | **trouvé** | absent |
| Docker Hub | sain | **trouvé** | absent |
| crates.io | sain | absent | absent |
| Reddit | bloqué | bloqué | bloqué |
| npm | bloqué | bloqué | bloqué |
| PyPI | bloqué | bloqué | bloqué |

**Faux positifs mesurés : 0 sur 8** (objectif < 3 %).
**Faux négatifs :** non mesurés — il faudrait un jeu de comptes de contrôle sur
chaque plateforme, ce qui n'existe pas encore.

L'écart entre les colonnes `torvalds` et le pseudo aléatoire est la seule chose
qui compte : une sonde qui répondrait « trouvé » dans les deux colonnes serait
inutile, et c'est exactement ce que le contrôle détecte.

## Ce que la sonde de contrôle a trouvé

Au premier essai, **PyPI répondait « existe » pour un pseudo aléatoire**. Sans
ce mécanisme, elle aurait produit un faux positif sur *toutes* les recherches —
le défaut le plus grave possible dans un outil d'enquête.

Diagnostic, vérifié à la main :

```
$ curl -s -o /dev/null -w "%{http_code}" https://pypi.org/user/6DPBTkLyrM7d/
200
$ curl -s https://pypi.org/user/6DPBTkLyrM7d/ | grep -oE "<title>[^<]*"
<title>Client Challenge
```

PyPI sert une page anti-bot **avec un code HTTP 200**. La règle « 200 = le
compte existe » était donc structurellement fausse, et aucune lecture du code
ne l'aurait montrée. Même situation pour npm, qui renvoie 403 à toute requête
non-navigateur.

Les deux sondes sont désormais marquées `Blocked` plutôt que corrigées par
contournement : le §2 du cahier des charges exclut explicitement toute
impersonation TLS ou évasion anti-bot en v1. Un site qui nous bloque est un
site sur lequel on ne conclut pas.

## Sondes bloquées : 4 sur 8

GitLab, Reddit, npm et PyPI répondent derrière Cloudflare ou un pare-feu
applicatif. C'est une limite assumée, pas un échec technique : contourner
ouvrirait une surface de maintenance permanente et une zone grise sur les CGU.

À noter : GitLab est bloqué au contrôle mais répond correctement sur un compte
réel. Le blocage n'est donc pas systématique — il dépend probablement du
volume ou de la réputation de l'adresse IP.

## Reproduire

```bash
# Compte public connu
cargo run -p spectra-probe --example live_check -- torvalds

# Pseudo qui n'existe pas — aucun résultat attendu
cargo run -p spectra-probe --example live_check -- zzqxvnptrwbk9438
```

L'exemple sort sur le réseau : il n'est **pas** un test. La suite de tests ne
dépend d'aucun site tiers (CLAUDE.md §11) — `cargo test` valide seulement la
structure du snapshot embarqué (`src-tauri/tests/osint_probes.rs`, 7 tests).

## Limites de ce relevé

1. **8 sondes, pas 700.** Le convertisseur WhatsMyName existe mais aucun
   dataset amont n'est encore intégré. La couverture réelle est celle du
   snapshot embarqué, et l'interface le dit désormais explicitement.
2. **Un seul pseudo positif testé.** Le §12 demande 20 pseudos publics
   contrôlés ; le taux de faux positifs annoncé porte sur un échantillon trop
   petit pour être statistiquement solide.
3. **Aucune mesure de faux négatifs.** Une sonde qui répondrait toujours
   « absent » passerait ce relevé sans être détectée.
4. **Une seule exécution, une seule IP.** Le taux de blocage dépend de la
   réputation réseau et varierait ailleurs.
