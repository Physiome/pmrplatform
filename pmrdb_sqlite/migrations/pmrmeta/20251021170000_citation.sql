-- Only dealing with the citation ids for now.

CREATE TABLE IF NOT EXISTS citation (
    id INTEGER PRIMARY KEY NOT NULL,
    identifier TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS citation__citation_id ON citation(id);

CREATE TABLE IF NOT EXISTS citation_link (
    citation_id INTEGER NOT NULL,
    resource_path TEXT NOT NULL,
    FOREIGN KEY(citation_id) REFERENCES citation(id)
);
