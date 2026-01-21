-- Only dealing with the citation ids for now.

CREATE TABLE IF NOT EXISTS citation (
    id INTEGER PRIMARY KEY NOT NULL,
    identifier TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS citation__citation_id ON citation(id);
CREATE UNIQUE INDEX IF NOT EXISTS citation__identifier ON citation(identifier);
