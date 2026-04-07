CREATE TABLE IF NOT EXISTS license_audit (
    id            TEXT PRIMARY KEY,
    server_serial TEXT NOT NULL,
    user_name     TEXT NOT NULL,
    unit_name     TEXT NOT NULL,
    token_serial  TEXT NOT NULL,
    machine_fp    TEXT NOT NULL,
    cpu_id        TEXT NOT NULL,
    board_serial  TEXT NOT NULL,
    product       TEXT NOT NULL,
    expires_at    INTEGER,
    output_path   TEXT NOT NULL,
    created_at    INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_license_audit_created_at ON license_audit(created_at DESC);
