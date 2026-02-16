package db

const schemaSQL = `
-- ============================================================================
-- PHASES (replaces STATUS.md + status/trackers/*.md)
-- ============================================================================

CREATE TABLE IF NOT EXISTS phases (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    path TEXT UNIQUE NOT NULL,
    name TEXT NOT NULL,
    category TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',
    completed_date TEXT,
    description TEXT,
    test_count INTEGER DEFAULT 0,
    test_target INTEGER,
    acceptance_criteria TEXT,
    blockers TEXT,
    dependencies TEXT,
    files_modified TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_phases_category ON phases(category);
CREATE INDEX IF NOT EXISTS idx_phases_status ON phases(status);
CREATE INDEX IF NOT EXISTS idx_phases_completed_date ON phases(completed_date);

-- ============================================================================
-- CATEGORIES (replaces status/trackers/*.md)
-- ============================================================================

CREATE TABLE IF NOT EXISTS categories (
    id INTEGER PRIMARY KEY,
    name TEXT UNIQUE NOT NULL,
    display_name TEXT NOT NULL,
    completed INTEGER NOT NULL DEFAULT 0,
    total INTEGER NOT NULL,
    percentage INTEGER NOT NULL DEFAULT 0,
    status TEXT NOT NULL DEFAULT 'pending',
    status_notes TEXT,
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- ============================================================================
-- DECISIONS (replaces docs/decision-logs/**/*.md)
-- ============================================================================

CREATE TABLE IF NOT EXISTS decisions (
    id TEXT PRIMARY KEY,
    component TEXT NOT NULL,
    title TEXT NOT NULL,
    decision TEXT NOT NULL,
    rationale TEXT NOT NULL,
    alternatives TEXT,
    consequences TEXT,
    date TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'accepted',
    superseded_by TEXT,
    related_phases TEXT,
    tags TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_decisions_component ON decisions(component);
CREATE INDEX IF NOT EXISTS idx_decisions_date ON decisions(date);
CREATE INDEX IF NOT EXISTS idx_decisions_status ON decisions(status);

-- ============================================================================
-- FEATURES (replaces docs/features/**/*.md)
-- ============================================================================

CREATE TABLE IF NOT EXISTS features (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT UNIQUE NOT NULL,
    display_name TEXT NOT NULL,
    version TEXT NOT NULL,
    status TEXT NOT NULL,
    description TEXT,
    implementation_notes TEXT,
    related_phases TEXT,
    spec_path TEXT,
    api_path TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_features_version ON features(version);
CREATE INDEX IF NOT EXISTS idx_features_status ON features(status);

-- ============================================================================
-- SPECS (tracks specification docs)
-- ============================================================================

CREATE TABLE IF NOT EXISTS specs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    path TEXT UNIQUE NOT NULL,
    name TEXT NOT NULL,
    section TEXT NOT NULL,
    title TEXT NOT NULL,
    summary TEXT,
    last_validated TEXT,
    validation_status TEXT,
    related_features TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_specs_section ON specs(section);

-- ============================================================================
-- API_DOCS (tracks API documentation)
-- ============================================================================

CREATE TABLE IF NOT EXISTS api_docs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    path TEXT UNIQUE NOT NULL,
    module TEXT NOT NULL,
    name TEXT NOT NULL,
    title TEXT NOT NULL,
    functions_count INTEGER DEFAULT 0,
    last_validated TEXT,
    validation_status TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- ============================================================================
-- METADATA (replaces STATUS.md header + global config)
-- ============================================================================

CREATE TABLE IF NOT EXISTS metadata (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- ============================================================================
-- AUDIT_LOG (change history)
-- ============================================================================

CREATE TABLE IF NOT EXISTS audit_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp TEXT NOT NULL DEFAULT (datetime('now')),
    action TEXT NOT NULL,
    entity_type TEXT NOT NULL,
    entity_id TEXT NOT NULL,
    old_data TEXT,
    changes TEXT NOT NULL,
    commit_sha TEXT,
    agent TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_audit_timestamp ON audit_log(timestamp);
CREATE INDEX IF NOT EXISTS idx_audit_entity ON audit_log(entity_type, entity_id);

-- ============================================================================
-- PARITY_CHECKS (tracks code/spec/docs/tests parity validation)
-- ============================================================================

CREATE TABLE IF NOT EXISTS parity_checks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp TEXT NOT NULL DEFAULT (datetime('now')),
    check_type TEXT NOT NULL,
    status TEXT NOT NULL,
    issues_count INTEGER NOT NULL DEFAULT 0,
    issues TEXT,
    summary TEXT,
    duration_ms INTEGER
);

CREATE INDEX IF NOT EXISTS idx_parity_timestamp ON parity_checks(timestamp);
CREATE INDEX IF NOT EXISTS idx_parity_status ON parity_checks(status);

-- ============================================================================
-- TEST_COVERAGE (tracks test counts and coverage)
-- ============================================================================

CREATE TABLE IF NOT EXISTS test_coverage (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp TEXT NOT NULL DEFAULT (datetime('now')),
    category TEXT NOT NULL,
    test_count INTEGER NOT NULL,
    passing INTEGER NOT NULL,
    failing INTEGER NOT NULL,
    coverage_percent REAL,
    details TEXT
);

CREATE INDEX IF NOT EXISTS idx_test_coverage_timestamp ON test_coverage(timestamp);
CREATE INDEX IF NOT EXISTS idx_test_coverage_category ON test_coverage(category);

-- ============================================================================
-- VIEWS (convenience queries)
-- ============================================================================

CREATE VIEW IF NOT EXISTS v_progress AS
SELECT
    c.name AS category,
    c.display_name,
    c.completed,
    c.total,
    c.percentage,
    c.status,
    c.status_notes
FROM categories c
ORDER BY c.id;

CREATE VIEW IF NOT EXISTS v_active_phases AS
SELECT
    p.id,
    p.path,
    p.name,
    p.category,
    p.description,
    p.test_count,
    p.test_target
FROM phases p
WHERE p.status IN ('in_progress', 'blocked')
ORDER BY p.category, p.name;

CREATE VIEW IF NOT EXISTS v_recent_decisions AS
SELECT
    d.id,
    d.component,
    d.title,
    d.date,
    d.status
FROM decisions d
ORDER BY d.date DESC
LIMIT 20;

CREATE VIEW IF NOT EXISTS v_parity_summary AS
SELECT
    check_type,
    status,
    COUNT(*) as check_count,
    MAX(timestamp) as last_check
FROM parity_checks
GROUP BY check_type, status
ORDER BY last_check DESC;

-- ============================================================================
-- TRIGGERS (Automatic Updates)
-- ============================================================================

CREATE TRIGGER IF NOT EXISTS update_category_progress
AFTER UPDATE ON phases
WHEN NEW.status = 'completed' AND OLD.status != 'completed'
BEGIN
    UPDATE categories
    SET
        completed = (
            SELECT COUNT(*)
            FROM phases
            WHERE category = NEW.category AND status = 'completed'
        ),
        percentage = (
            SELECT ROUND(CAST(COUNT(*) AS REAL) / total * 100)
            FROM phases
            WHERE category = NEW.category AND status = 'completed'
        ),
        status = CASE
            WHEN (SELECT COUNT(*) FROM phases WHERE category = NEW.category AND status = 'completed') = total
            THEN 'complete'
            WHEN (SELECT COUNT(*) FROM phases WHERE category = NEW.category AND status = 'completed') > 0
            THEN 'active'
            ELSE 'pending'
        END,
        updated_at = datetime('now')
    WHERE name = NEW.category;

    UPDATE metadata
    SET value = (SELECT COUNT(*) FROM phases WHERE status = 'completed'),
        updated_at = datetime('now')
    WHERE key = 'completed_phases';

    UPDATE metadata
    SET value = datetime('now'),
        updated_at = datetime('now')
    WHERE key = 'last_updated';
END;

CREATE TRIGGER IF NOT EXISTS update_phases_timestamp
AFTER UPDATE ON phases
BEGIN
    UPDATE phases SET updated_at = datetime('now') WHERE id = NEW.id;
END;

CREATE TRIGGER IF NOT EXISTS update_decisions_timestamp
AFTER UPDATE ON decisions
BEGIN
    UPDATE decisions SET updated_at = datetime('now') WHERE id = NEW.id;
END;

CREATE TRIGGER IF NOT EXISTS update_features_timestamp
AFTER UPDATE ON features
BEGIN
    UPDATE features SET updated_at = datetime('now') WHERE id = NEW.id;
END;
`

const seedSQL = `
-- Seed categories
INSERT OR IGNORE INTO categories (id, name, display_name, total) VALUES
    (0, 'foundation', 'Foundation', 21),
    (1, 'stdlib', 'Standard Library', 21),
    (2, 'bytecode-vm', 'Bytecode & VM', 8),
    (3, 'frontend', 'Frontend', 5),
    (4, 'typing', 'Type System', 7),
    (5, 'interpreter', 'Interpreter', 2),
    (6, 'cli', 'CLI', 6),
    (7, 'lsp', 'LSP', 5),
    (8, 'polish', 'Polish', 5);

-- Seed metadata
INSERT OR IGNORE INTO metadata (key, value) VALUES
    ('schema_version', '1'),
    ('atlas_version', 'v0.2'),
    ('total_phases', '78'),
    ('completed_phases', '0'),
    ('last_updated', datetime('now'));
`

// InitSchema creates all tables, indexes, triggers, views and seeds initial data
func (db *DB) InitSchema() error {
	err := db.WithTransaction(func(tx *Transaction) error {
		// Create schema
		if _, err := tx.Exec(schemaSQL); err != nil {
			return err
		}

		// Seed data
		if _, err := tx.Exec(seedSQL); err != nil {
			return err
		}

		return nil
	})

	if err != nil {
		return err
	}

	// Prepare statements after schema is created
	return db.Prepare()
}
