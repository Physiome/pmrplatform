CREATE TABLE IF NOT EXISTS alias (
    kind TEXT NOT NULL,
    kind_id INTEGER NOT NULL,
    alias TEXT NOT NULL,
    created_ts INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS alias__kind_kind_id ON alias(kind, kind_id);
CREATE UNIQUE INDEX IF NOT EXISTS alias__kind_alias ON alias(kind, alias);

CREATE TABLE IF NOT EXISTS alias_request (
    kind TEXT NOT NULL,
    kind_id INTEGER NOT NULL,
    alias TEXT NOT NULL,
    created_ts INTEGER NOT NULL,
    -- references the pmrac model
    user_id INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS alias_request__kind_kind_id ON alias_request(kind, kind_id);
CREATE INDEX IF NOT EXISTS alias_request__kind_alias ON alias_request(kind, alias);
CREATE INDEX IF NOT EXISTS alias_request__user_id_kind_alias ON alias_request(user_id, kind, alias);
CREATE INDEX IF NOT EXISTS alias_request__user_id_kind_kind_id ON alias_request(user_id, kind, kind_id);
