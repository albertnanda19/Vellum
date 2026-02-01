# Vellum

Transactional, type-safe, and auditable PostgreSQL migrations for production workflows.

## 1. Project Title & Tagline

**Vellum** — a transactional, infrastructure-first PostgreSQL migration tool written in Rust.

## 2. Overview

Vellum is a CLI tool for applying SQL migrations to PostgreSQL databases with a strong focus on operational safety.
It targets teams and individual engineers who need migration execution to be:

- predictable in CI/CD,
- safe under concurrency,
- auditable after the fact,
- and resistant to partial/half-applied state.

Vellum is designed for real infrastructure workflows: it is non-interactive, deterministic, and aims to produce stable output suitable for both humans and automation.

## 3. Why Vellum?

Database migrations are risky because they often run at the exact moment you can least afford surprises: during deploys.
Many migration systems fail in one (or more) of these ways:

- **Partial migration risk**: statements apply incrementally and can leave the database in an unknown intermediate state when something fails.
- **Weak concurrency behavior**: multiple deploy jobs (or engineers) can run migrations simultaneously, creating race conditions.
- **Non-real dry-runs**: “dry-run” sometimes only checks file presence/order, rather than executing inside the database engine.
- **Poor auditability**: once migrations run, it can be hard to answer “what ran, when, and why did it fail?”

Vellum addresses these by making transactional execution, advisory locking, and audit logging first-class behaviors.

## 4. Key Features

- **Transactional migration execution**
  Migrations run inside database transactions to avoid partial state where possible.

- **Advisory locking (concurrency safe)**
  Uses PostgreSQL advisory locks to prevent concurrent migration runs.

- **Dry-run execution (rollback-only)**
  Executes migrations in a rollback-only mode to validate behavior without applying changes.

- **AST-based SQL parsing (pg_query)**
  Uses PostgreSQL’s parser (via `pg_query`) to parse statements, enabling safer statement handling than naive splitting.

- **Checksum & drift detection**
  Detects changes to already-applied migrations via checksums.

- **Statement-level audit logging**
  Records execution details for traceability.

- **Deterministic migration ordering**
  Ensures stable and predictable ordering of migrations.

## 5. Design Principles & Core Concepts

- **Atomicity over convenience**
  Prefer all-or-nothing behavior where PostgreSQL semantics allow.

- **Fail fast & loud**
  Stop immediately on unsafe or invalid state; present actionable errors.

- **Deterministic behavior**
  Ordering and output should not depend on timing or environment quirks.

- **Single source of truth**
  Migration history is derived from authoritative state (migration files + database audit tables).

- **No partial state**
  Avoid leaving the database in an ambiguous migration state.

- **Infrastructure-first UX**
  Output is concise, stable, and designed for CI logs.

- **Safety > flexibility**
  Vellum intentionally avoids “smart magic” that can hide risk.

## 6. Architecture Overview (high-level, non-diagram)

At a high level, Vellum is composed of:

- **CLI layer** (`vellum` binary)
  Parses arguments, resolves configuration (database URL), and prints user-facing reports/errors.

- **Execution engine**
  Applies migrations (or performs dry-run execution) using transactional semantics and reports the outcome.

- **Locking layer**
  Acquires PostgreSQL advisory locks to guarantee a single active migration runner per database.

- **Migration discovery**
  Loads and orders migrations from the local `migrations/` directory.

- **Database audit tables**
  Stores applied migrations, run metadata, checksums, and statement-level execution data.

## 7. Installation & Setup

Starting from an empty folder:

### Clone the repository

SSH:

```bash
git clone git@github.com:albertnanda19/Vellum.git
cd Vellum
```

HTTPS:

```bash
git clone https://github.com/albertnanda19/Vellum.git
cd Vellum
```

### Install Rust toolchain

Install Rust using rustup:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup update
```

### Build the project

```bash
cargo build
```

### Configure the database URL

Vellum supports both command-line override and environment-based configuration.

- Preferred env var: `VELLUM_DATABASE_URL`
- Fallback env var: `DATABASE_URL`
- CLI override flag: `--database-url`

You can use a `.env` file (recommended for local development). Create `.env` at repo root:

```bash
cat > .env <<'EOF'
VELLUM_DATABASE_URL="postgres://postgres:password@localhost:5432/vellum"
EOF
```

Vellum will automatically load `.env` at startup.

### Ensure PostgreSQL is running

Vellum requires a reachable PostgreSQL instance.

Example quick check:

```bash
psql "postgres://postgres:password@localhost:5432/vellum" -c 'select 1;'
```

## 8. Usage Guide

Vellum currently exposes two primary subcommands.

### `vellum status`

Shows migration state for the target database.

```bash
vellum status
```

Behind the scenes, Vellum:

- connects to the database,
- discovers local migrations from `migrations/`,
- reads audit tables to compute applied vs pending,
- prints a stable summary.

### `vellum migrate`

Applies pending migrations.

```bash
vellum migrate
```

Behind the scenes, Vellum:

- connects to the database,
- discovers local migrations from `migrations/`,
- acquires an advisory lock,
- applies migrations transactionally,
- records audit information.

### `vellum migrate --dry-run`

Validates migrations without applying changes.

```bash
vellum migrate --dry-run
```

Behind the scenes, Vellum:

- connects to the database,
- acquires an advisory lock,
- executes migrations in rollback-only mode,
- ensures migrations are executable and consistent,
- writes no schema changes.

### Running from source

If you have not installed the binary, you can run it via Cargo:

```bash
cargo run -- status
cargo run -- migrate --dry-run
cargo run -- migrate
```

### Installing the binary locally

```bash
cargo install --path .
```

Then you can run:

```bash
vellum status
```

## 9. Example Output

### `vellum status`

```text
----------------------------------------
Vellum Status
----------------------------------------
Database            : vellum

Applied migrations  : 12
Pending migrations  : 2
Last migration      : 00012_add_index_users
Last run status     : success
----------------------------------------
→ Run `vellum migrate` to apply pending migrations
```

### `vellum migrate --dry-run`

```text
----------------------------------------
Vellum Migration (dry-run)
----------------------------------------
Database            : vellum

✔ Connected to database
✔ Advisory lock acquired
→ Validating 2 migrations

✔ All migrations are valid
✔ No changes were applied
----------------------------------------
```

### `vellum migrate`

```text
----------------------------------------
Vellum Migration
----------------------------------------
Database            : vellum
Mode                : apply

✔ Connected to database
✔ Advisory lock acquired
→ Applying 2 migrations

• 00013_add_column_orders -> OK (12ms)
• 00014_backfill_orders   -> OK (48ms)

✔ Migration completed successfully
----------------------------------------
```

## 10. Safety & Guarantees

Vellum is designed to provide the following guarantees:

- **No partial migration (where PostgreSQL semantics allow)**
  Migrations are executed transactionally to reduce the chance of half-applied state.

- **No concurrent execution**
  Advisory locks prevent multiple runners from migrating the same database concurrently.

- **No silent failure**
  Errors are presented with a clear title, reason, and a concrete next step.

- **No schema drift unnoticed**
  Checksum mismatch detection surfaces modified migrations that were previously applied.

## 11. Comparison (High-level)

This section is intentionally high-level and focuses on approach.

- **Flyway**
  Widely used migration framework with many deployment integrations. Vellum focuses on a narrower scope: PostgreSQL-only with strong transactional and audit semantics.

- **Diesel (migrations)**
  Common in Rust projects using Diesel ORM. Vellum is SQL-first and is designed to work without requiring an ORM.

- **SQLx migrations**
  Integrates migrations into SQLx-based applications. Vellum separates execution into a dedicated infra CLI with a deterministic UX and explicit audit/run tracking.

## 12. Limitations

To keep the project operationally conservative, Vellum has clear constraints:

- **PostgreSQL only**
- **No web UI / dashboard**
- **No interactive mode**
- **No automatic rollback strategy beyond transactional semantics**
  (If a statement is not transaction-safe, PostgreSQL may not be able to roll it back.)
- **No data migration helpers**
  Vellum executes SQL; data backfills and online migration patterns must be implemented explicitly.

## 13. Development & Contribution Notes

Vellum is currently early-stage.

- The focus is correctness, safety, and predictable behavior.
- Features are added conservatively; changes that affect safety or determinism require strong justification.

Contributions are welcome:

- Open an issue describing the change and the operational motivation.
- Keep UX changes predictable and non-interactive.
- Avoid adding features that require always-on services or external dependencies.

## 14. Author

Albert Mangiri
