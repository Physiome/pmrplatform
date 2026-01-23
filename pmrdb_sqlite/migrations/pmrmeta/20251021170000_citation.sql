CREATE TABLE IF NOT EXISTS citation (
    id TEXT PRIMARY KEY NOT NULL,
    title TEXT NOT NULL,
    journal TEXT,
    volume TEXT,
    first_page TEXT,
    last_page TEXT,
    issued TEXT
);
CREATE INDEX IF NOT EXISTS citation__title ON citation(title);

-- Rather than a common registry of authors, just duplicate whatever duplicates
-- for every citation.  Rationale for this is that these citation entries are
-- from possibly incomplete, user provided metadata rather than from official
-- sources or registries.
CREATE TABLE IF NOT EXISTS citation_author (
    citation_id TEXT NOT NULL,
    family TEXT NOT NULL,
    given TEXT,
    -- concatenation of what's provided?
    other TEXT,
    ordering INTEGER,
    FOREIGN KEY(citation_id) REFERENCES citation(id)
);

CREATE INDEX IF NOT EXISTS citation_author__citation_id ON citation_author(citation_id);
CREATE INDEX IF NOT EXISTS citation_author__family_given ON citation_author(family, given);
