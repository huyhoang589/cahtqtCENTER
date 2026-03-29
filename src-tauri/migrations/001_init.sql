CREATE TABLE IF NOT EXISTS settings (
    key   TEXT PRIMARY KEY NOT NULL,
    value TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS partners (
    id         TEXT PRIMARY KEY NOT NULL,
    name       TEXT NOT NULL UNIQUE,
    created_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS partner_members (
    id              TEXT    PRIMARY KEY NOT NULL,
    partner_id      TEXT    NOT NULL REFERENCES partners(id) ON DELETE CASCADE,
    name            TEXT    NOT NULL,
    email           TEXT,
    cert_cn         TEXT    NOT NULL,
    cert_serial     TEXT    NOT NULL,
    cert_valid_from INTEGER NOT NULL,
    cert_valid_to   INTEGER NOT NULL,
    cert_file_path  TEXT    NOT NULL,
    created_at      INTEGER NOT NULL,
    UNIQUE(partner_id, cert_serial)
);

CREATE TABLE IF NOT EXISTS enc_logs (
    id                TEXT    PRIMARY KEY NOT NULL,
    operation         TEXT    NOT NULL,
    src_file          TEXT    NOT NULL,
    dst_file          TEXT    NOT NULL,
    partner_member_id TEXT,
    status            TEXT    NOT NULL,
    error_msg         TEXT,
    created_at        INTEGER NOT NULL
);
