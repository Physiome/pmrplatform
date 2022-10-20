CREATE TABLE IF NOT EXISTS workspace_alias (
    id INTEGER PRIMARY KEY NOT NULL,
    workspace_id INTEGER NOT NULL,
    alias TEXT NOT NULL UNIQUE,
    created INTEGER NOT NULL,
    FOREIGN KEY(workspace_id) REFERENCES workspace(id)
);

CREATE UNIQUE INDEX workspace_alias_idx_workspace_id_alias ON workspace_alias(workspace_id, alias);
