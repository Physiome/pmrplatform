CREATE TABLE IF NOT EXISTS idx_kind (
    id INTEGER PRIMARY KEY NOT NULL,
    description TEXT NOT NULL
);
CREATE UNIQUE INDEX IF NOT EXISTS idx_kind__description ON idx_kind(description);

CREATE TABLE IF NOT EXISTS idx_entry (
    id INTEGER PRIMARY KEY NOT NULL,
    idx_kind_id INTEGER NOT NULL,
    term TEXT NOT NULL,
    FOREIGN KEY(idx_kind_id) REFERENCES idx_kind(id)
);
CREATE UNIQUE INDEX IF NOT EXISTS idx_entry__idx_kind_id_term ON idx_entry(idx_kind_id, term);

CREATE TABLE IF NOT EXISTS idx_entry_link (
    idx_entry_id INTEGER NOT NULL,
    resource_path TEXT NOT NULL,
    FOREIGN KEY(idx_entry_id) REFERENCES idx_entry(id)
);
CREATE UNIQUE INDEX IF NOT EXISTS idx_entry_link__idx_entry_id_resource_path ON idx_entry_link(idx_entry_id, resource_path);
