// Sérialisation JSON canonique : clés triées alphabétiquement à tous
// les niveaux, et clés `undefined` filtrées (comme JSON.stringify).
// Réutilisé par lib/audit.ts ET par le port Rust
// (compute_row_hash dans src-tauri/src/commands/audit.rs).
// DRY : tout changement ici DOIT être reflété dans le test Rust
// `audit::tests::test_canonical_hash_matches_ts`.

export function canonicalJson(value: unknown): string {
  if (value === null || typeof value !== 'object') return JSON.stringify(value)
  if (Array.isArray(value)) return '[' + value.map(canonicalJson).join(',') + ']'
  const obj = value as Record<string, unknown>
  // Filtre les clés `undefined` : on n'inclut que les valeurs définies.
  // C'est la sémantique de JSON.stringify sur un objet.
  const keys = Object.keys(obj)
    .filter((k) => obj[k] !== undefined)
    .sort()
  return (
    '{' +
    keys
      .map((k) => JSON.stringify(k) + ':' + canonicalJson(obj[k]))
      .join(',') +
    '}'
  )
}
