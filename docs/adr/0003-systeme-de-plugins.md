# ADR 0003 — Système de plugins (transforms)

* **Statut** : accepté (implémentation différée à la Phase 3)
* **Date** : 2026-07-28
* **Phase** : 0 (décision), 3 (mise en œuvre)

## Contexte

`CLAUDE.md` §3 arrête le principe : un transform est un module WebAssembly sandboxé exécuté par
Extism (runtime wasmtime), décrit par un manifeste TOML, avec un accès réseau exclusivement médié
par une host function appliquant allowlist de domaines, rate limit, timeout et taille maximale de
réponse.

Trois options ont été considérées pour la Phase 3 :

1. **Extism / WASM** — isolation réelle, plugins écrits en Rust, TypeScript, Go, Python ou Zig.
2. **Sous-processus natifs** (modèle Maltego) — simple, mais aucune isolation : un transform
   malveillant a les droits de l'analyste, ce qui est inacceptable pour un outil manipulant des
   dossiers d'enquête.
3. **Scripting embarqué** (Rhai, Lua) — sûr et léger, mais enferme les contributeurs dans un seul
   langage et ne couvre pas les besoins de parsing lourds.

## Décision

**Extism 1.30 (wasmtime transitif).** Vérifié le 2026-07-28 : dernière publication 2026-06-04,
~203 000 téléchargements récents, écosystème de PDK multi-langages actif.

Règles retenues :

1. `wasmtime` n'est **pas** une dépendance directe : il est fourni par `extism`. Le déclarer
   séparément expose à un conflit de versions (même famille de problème que le conflit
   `libsqlite3-sys` rencontré au moment de la mise en place du workspace).
2. **Aucun accès réseau direct depuis le sandbox.** Pas de socket brut, pas d'accès filesystem hors
   répertoire alloué. Tout passe par une host function de l'hôte.
3. Le manifeste TOML déclare les domaines autorisés ; un appel hors allowlist doit être **bloqué et
   testé comme tel** — c'est le critère d'acceptation de la Phase 3 dans `CLAUDE.md` §12.
4. Les transforms natifs du socle (DNS, WHOIS, Certificate Transparency, WhatsMyName) sont compilés
   en Rust pour la performance mais **exposent le même trait `Transform`** que les plugins WASM. Une
   asymétrie ici produirait deux chemins de code et deux niveaux de garantie.
5. Registre local, signature Ed25519 vérifiée à l'installation.

## Conséquences

* Le trait `Transform` et l'orchestrateur (DAG, annulation, rate limiting par domaine) doivent être
  définis **avant** d'écrire le premier transform natif, sous peine de devoir réécrire les
  transforms du socle quand le runtime WASM arrivera.
* Le SDK de plugins sera publié sous **Apache-2.0** (et non AGPL-3.0) conformément à la contrainte
  C5 : le cœur reste copyleft, l'écriture de plugins tiers reste libre de contrainte.
* Rien de tout cela n'est implémenté en Phase 0 : cette décision existe pour que les phases 1 et 2
  ne prennent pas de dépendance incompatible avec elle.
