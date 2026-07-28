# ADR 0004 — Backend IA local

* **Statut** : accepté (implémentation différée à la Phase 6)
* **Date** : 2026-07-28
* **Phase** : 0 (décision), 6 (mise en œuvre)

## Contexte

`CLAUDE.md` §3 impose une IA locale, optionnelle et jamais bloquante : Ollama par défaut via
`ollama-rs`, `mistral.rs` en repli « zéro installation », embeddings via `fastembed-rs`, le tout
derrière un trait `LlmBackend`.

L'implémentation héritée (Cekarna) appelle Ollama **depuis TypeScript** par `fetch()` direct
(`lib/ollama.ts`, `lib/tauri-bridge.ts:411-432`), sans trait, sans repli et sans marquage de
provenance des sorties.

## Décision

1. **Le trait `LlmBackend` vit dans `spectra-ai`, côté Rust.** Aucun appel de modèle depuis le
   frontend : le backend est le seul à parler au modèle, ce qui permet d'appliquer uniformément les
   garde-fous (timeout, marquage de provenance, journalisation) et de tenir la contrainte C1
   (maîtrise de ce qui sort de la machine).
2. **Implémentation par défaut : `ollama-rs` 0.3.6** (vérifié le 2026-07-28, publié le 2026-07-24,
   ~143 000 téléchargements récents — le crate le plus actif du domaine).
3. **Embeddings : `fastembed` 5.17.3** (le crate s'appelle `fastembed`, pas `fastembed-rs`), qui ne
   dépend pas d'Ollama et permet la recherche sémantique même sans modèle de génération installé.
4. **`mistral.rs` en repli** : évalué en Phase 6 seulement. Le coût (taille du binaire, temps de
   compilation, support GPU) doit être mesuré avant de s'engager.
5. **Dégradation propre obligatoire.** Sans Ollama, chaque fonction IA renvoie « indisponible » et
   l'application reste 100 % fonctionnelle — critère d'acceptation de la Phase 6.

## Garde-fous non négociables

Toute sortie de modèle est marquée `provenance = INFERRED` dans le modèle d'observation, affichée de
manière visuellement distincte, **jamais fusionnée automatiquement** dans le graphe, et exclue par
défaut des rapports. Un raisonnement de LLM n'est pas une preuve.

Ces garde-fous ne sont pas une préoccupation de Phase 6 : ils contraignent le champ `provenance` de
l'`Observation`, donc la **Phase 1**. C'est la raison pour laquelle cet ADR est écrit maintenant.

## Conséquences

* `spectra-core` doit exposer `Provenance { Collected, Asserted, Inferred }` dès la Phase 1.
* Le code Ollama existant dans `lib/*.ts` sera remplacé, pas étendu : il constitue une dette connue,
  documentée dans `audit.md`.
* `sqlite-vec` (0.1.9, pré-1.0) sera nécessaire pour l'index vectoriel : il reste derrière une
  abstraction et hors du chemin critique du stockage (voir ADR 0001).
