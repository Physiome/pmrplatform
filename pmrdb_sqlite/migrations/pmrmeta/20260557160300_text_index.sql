-- The postgresql equivalent may be a GIN index on a table.
-- <https://www.postgresql.org/docs/current/textsearch-tables.html>
CREATE VIRTUAL TABLE idx_text USING fts5(
    title,
    content,
    resource_path UNINDEXED
);
