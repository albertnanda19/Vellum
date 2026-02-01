BEGIN;

CREATE SCHEMA IF NOT EXISTS vellum;

CREATE TABLE IF NOT EXISTS vellum.vellum_runs (
    id UUID PRIMARY KEY,
    started_at TIMESTAMPTZ NOT NULL,
    finished_at TIMESTAMPTZ,
    mode TEXT NOT NULL CHECK (mode IN ('dry-run', 'apply')),
    status TEXT NOT NULL CHECK (status IN ('running', 'success', 'failed')),
    db_name TEXT NOT NULL,
    db_user TEXT NOT NULL,
    client_host TEXT,
    vellum_version TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS vellum.vellum_migrations (
    id BIGSERIAL PRIMARY KEY,
    version TEXT NOT NULL,
    name TEXT NOT NULL,
    checksum TEXT NOT NULL,
    applied_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    execution_time_ms INTEGER NOT NULL,
    success BOOLEAN NOT NULL,
    error_code TEXT,
    error_message TEXT,
    run_id UUID NOT NULL REFERENCES vellum.vellum_runs(id) ON DELETE CASCADE,
    UNIQUE (version),
    UNIQUE (checksum)
);

CREATE TABLE IF NOT EXISTS vellum.vellum_statements (
    id BIGSERIAL PRIMARY KEY,
    migration_id BIGINT NOT NULL
        REFERENCES vellum.vellum_migrations(id)
        ON DELETE CASCADE,
    ordinal INTEGER NOT NULL,
    statement_hash TEXT NOT NULL,
    statement_kind TEXT NOT NULL,
    transactional BOOLEAN NOT NULL,
    execution_time_ms INTEGER,
    success BOOLEAN,
    error_message TEXT
);

CREATE TABLE IF NOT EXISTS vellum.vellum_schema_snapshots (
    id BIGSERIAL PRIMARY KEY,
    run_id UUID NOT NULL
        REFERENCES vellum.vellum_runs(id)
        ON DELETE CASCADE,
    snapshot_type TEXT NOT NULL CHECK (snapshot_type IN ('before', 'after')),
    schema_hash TEXT NOT NULL,
    snapshot JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS vellum.vellum_locks (
    lock_key TEXT PRIMARY KEY,
    acquired_at TIMESTAMPTZ NOT NULL,
    owner_run_id UUID NOT NULL
);

CREATE TABLE IF NOT EXISTS vellum.vellum_metadata (
    key TEXT PRIMARY KEY,
    value JSONB NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_migrations_run_id
    ON vellum.vellum_migrations(run_id);

CREATE INDEX IF NOT EXISTS idx_statements_migration_id
    ON vellum.vellum_statements(migration_id);

CREATE INDEX IF NOT EXISTS idx_schema_snapshots_run_id
    ON vellum.vellum_schema_snapshots(run_id);

COMMIT;
