CREATE TABLE IF NOT EXISTS cases (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    legal_basis TEXT,
    purpose TEXT,
    retention_days INTEGER,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS entities (
    id TEXT PRIMARY KEY,
    kind TEXT NOT NULL,
    canonical_value TEXT NOT NULL,
    display_label TEXT NOT NULL,
    properties TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL,
    merged_from TEXT NOT NULL DEFAULT '[]'
);

CREATE TABLE IF NOT EXISTS observations (
    id TEXT PRIMARY KEY,
    subject TEXT NOT NULL REFERENCES entities(id),
    predicate TEXT NOT NULL,
    value TEXT NOT NULL,
    source TEXT NOT NULL,
    method TEXT NOT NULL,
    observed_at TEXT NOT NULL,
    valid_from TEXT,
    valid_to TEXT,
    confidence_source TEXT NOT NULL,
    confidence_info INTEGER NOT NULL,
    provenance TEXT NOT NULL,
    raw_hash TEXT,
    operator TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS relations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    source TEXT NOT NULL REFERENCES entities(id),
    target TEXT NOT NULL REFERENCES entities(id),
    kind TEXT NOT NULL,
    properties TEXT NOT NULL DEFAULT '{}'
);

CREATE INDEX IF NOT EXISTS idx_observations_subject ON observations(subject);
CREATE INDEX IF NOT EXISTS idx_relations_source ON relations(source);
CREATE INDEX IF NOT EXISTS idx_relations_target ON relations(target);
CREATE INDEX IF NOT EXISTS idx_entities_kind ON entities(kind);
CREATE INDEX IF NOT EXISTS idx_entities_canonical ON entities(canonical_value);

CREATE TABLE IF NOT EXISTS audit_log (
    id TEXT PRIMARY KEY,
    case_id TEXT NOT NULL,
    action TEXT NOT NULL,
    entity_kind TEXT NOT NULL,
    entity_id TEXT,
    actor TEXT NOT NULL,
    sequence INTEGER NOT NULL UNIQUE,
    payload TEXT NOT NULL DEFAULT '{}',
    hash TEXT NOT NULL,
    previous_hash TEXT NOT NULL,
    timestamp TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_audit_case ON audit_log(case_id);
CREATE INDEX IF NOT EXISTS idx_audit_sequence ON audit_log(sequence);
